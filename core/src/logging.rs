use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result, anyhow};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{self, RollingFileAppender},
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    EnvFilter, Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
use uuid::Uuid;

use crate::config::{LoggingConfig, LoggingRotation};

const LOG_FILE_PREFIX: &str = "core.log";

pub struct LoggingGuard {
    _worker_guard: WorkerGuard,
    run_id: String,
}

impl LoggingGuard {
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}

pub fn init_tracing(logging_config: &LoggingConfig) -> Result<LoggingGuard> {
    if logging_config.filter.trim().is_empty() {
        return Err(anyhow!("logging.filter cannot be empty"));
    }
    if logging_config.dir.as_os_str().is_empty() {
        return Err(anyhow!("logging.dir cannot be empty"));
    }

    let log_dir = resolve_log_dir(&logging_config.dir)?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create logging directory {}", log_dir.display()))?;

    let retention_warnings =
        purge_old_log_files(&log_dir, LOG_FILE_PREFIX, logging_config.retention_days);
    let appender = build_rolling_appender(&log_dir, logging_config.rotation.clone());
    let (non_blocking_writer, worker_guard) = tracing_appender::non_blocking(appender);
    let env_filter = build_env_filter(&logging_config.filter)?;

    let file_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_target(true)
        .with_current_span(true)
        .with_span_list(true)
        .with_ansi(false)
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    let stderr_layer = logging_config.stderr_warn_enabled.then(|| {
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_filter(LevelFilter::WARN)
    });

    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(file_layer)
        .with(stderr_layer)
        .try_init()
        .context("failed to initialize tracing subscriber")?;

    let run_id = Uuid::now_v7().to_string();
    tracing::info!(
        target: "logging",
        run_id = %run_id,
        dir = %log_dir.display(),
        filter = %logging_config.filter,
        rotation = ?logging_config.rotation,
        retention_days = logging_config.retention_days,
        stderr_warn_enabled = logging_config.stderr_warn_enabled,
        "logging_initialized"
    );
    for warning in retention_warnings {
        tracing::warn!(target: "logging", warning = %warning, "logging_retention_warning");
    }

    Ok(LoggingGuard {
        _worker_guard: worker_guard,
        run_id,
    })
}

fn build_env_filter(filter: &str) -> Result<EnvFilter> {
    EnvFilter::try_new(filter)
        .with_context(|| format!("failed to parse logging.filter '{}'", filter))
}

fn build_rolling_appender(log_dir: &Path, rotation: LoggingRotation) -> RollingFileAppender {
    match rotation {
        LoggingRotation::Daily => rolling::daily(log_dir, LOG_FILE_PREFIX),
        LoggingRotation::Hourly => rolling::hourly(log_dir, LOG_FILE_PREFIX),
    }
}

fn resolve_log_dir(dir: &Path) -> Result<PathBuf> {
    if dir.is_absolute() {
        return Ok(dir.to_path_buf());
    }

    Ok(std::env::current_dir()
        .context("failed to read current working directory for logging.dir resolution")?
        .join(dir))
}

fn purge_old_log_files(log_dir: &Path, prefix: &str, retention_days: usize) -> Vec<String> {
    purge_old_log_files_at(log_dir, prefix, retention_days, SystemTime::now())
}

fn purge_old_log_files_at(
    log_dir: &Path,
    prefix: &str,
    retention_days: usize,
    now: SystemTime,
) -> Vec<String> {
    let retention_seconds = retention_days.saturating_mul(24 * 60 * 60) as u64;
    let cutoff = now
        .checked_sub(Duration::from_secs(retention_seconds))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut warnings = Vec::new();

    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(err) => {
            warnings.push(format!(
                "failed to scan logging directory {}: {}",
                log_dir.display(),
                err
            ));
            return warnings;
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                warnings.push(format!(
                    "failed to iterate logging directory entries: {}",
                    err
                ));
                continue;
            }
        };

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(prefix) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                warnings.push(format!(
                    "failed to stat {}: {}",
                    entry.path().display(),
                    err
                ));
                continue;
            }
        };

        if !metadata.is_file() {
            continue;
        }

        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(err) => {
                warnings.push(format!(
                    "failed to read mtime for {}: {}",
                    entry.path().display(),
                    err
                ));
                continue;
            }
        };

        if modified <= cutoff
            && let Err(err) = fs::remove_file(entry.path())
        {
            warnings.push(format!(
                "failed to remove expired log file {}: {}",
                entry.path().display(),
                err
            ));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use uuid::Uuid;

    use super::{build_env_filter, purge_old_log_files_at};

    #[test]
    fn invalid_filter_is_rejected() {
        let err = build_env_filter("info,core==debug").expect_err("filter must fail");
        assert!(err.to_string().contains("logging.filter"));
    }

    #[test]
    fn retention_cleanup_only_removes_prefixed_files() {
        let dir = std::env::temp_dir().join(format!("beluna-logging-test-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let expired_log = dir.join("core.log.2026-02-01");
        let keep_file = dir.join("keep.txt");

        fs::write(&expired_log, "old").expect("log file should be created");
        fs::write(&keep_file, "keep").expect("non-log file should be created");

        let now = std::time::SystemTime::now() + Duration::from_secs(1);
        let warnings = purge_old_log_files_at(&dir, "core.log", 0, now);
        assert!(
            warnings.is_empty(),
            "cleanup should not generate warnings: {warnings:?}",
        );
        assert!(!expired_log.exists(), "prefixed file should be removed");
        assert!(keep_file.exists(), "non-prefixed file should remain");

        let _ = fs::remove_file(&keep_file);
        let _ = fs::remove_dir(&dir);
    }
}
