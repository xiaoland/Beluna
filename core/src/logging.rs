use std::{
    fs::{self, OpenOptions},
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result, anyhow};
use time::{OffsetDateTime, UtcOffset};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, filter::LevelFilter, fmt, layer::SubscriberExt,
    util::SubscriberInitExt,
};
use uuid::Uuid;

use crate::config::LoggingConfig;

const LOG_FILE_PREFIX: &str = "core.log";
const LOG_FILE_DATE_SEPARATOR: char = '-';

pub struct LoggingGuard {
    _worker_guard: WorkerGuard,
    run_id: String,
}

impl LoggingGuard {
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}

pub fn init_tracing(
    logging_config: &LoggingConfig,
    otlp_log_layer: Option<Box<dyn Layer<Registry> + Send + Sync>>,
    otlp_trace_layer: Option<Box<dyn Layer<Registry> + Send + Sync>>,
) -> Result<LoggingGuard> {
    let log_dir = resolve_log_dir(&logging_config.dir)?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create logging directory {}", log_dir.display()))?;

    let retention_warnings =
        purge_old_log_files(&log_dir, LOG_FILE_PREFIX, logging_config.retention_days);
    let (log_file, log_file_path, awake_sequence) =
        open_log_file_for_awake(&log_dir, LOG_FILE_PREFIX)?;
    let (non_blocking_writer, worker_guard) = tracing_appender::non_blocking(log_file);
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

    let otlp_layer = compose_otlp_layers(otlp_trace_layer, otlp_log_layer);

    tracing_subscriber::registry()
        .with(otlp_layer)
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
        log_file = %log_file_path.display(),
        awake_sequence = awake_sequence,
        filter = %logging_config.filter,
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

fn compose_otlp_layers(
    otlp_trace_layer: Option<Box<dyn Layer<Registry> + Send + Sync>>,
    otlp_log_layer: Option<Box<dyn Layer<Registry> + Send + Sync>>,
) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
    match (otlp_trace_layer, otlp_log_layer) {
        (Some(trace_layer), Some(log_layer)) => Some(trace_layer.and_then(log_layer).boxed()),
        (Some(trace_layer), None) => Some(trace_layer),
        (None, Some(log_layer)) => Some(log_layer),
        (None, None) => None,
    }
}

fn build_env_filter(filter: &str) -> Result<EnvFilter> {
    EnvFilter::try_new(filter)
        .with_context(|| format!("failed to parse logging.filter '{}'", filter))
}

fn open_log_file_for_awake(prefix_dir: &Path, prefix: &str) -> Result<(fs::File, PathBuf, u64)> {
    let date = current_local_date_string(SystemTime::now());
    let mut sequence = next_awake_sequence(prefix_dir, prefix, &date)?;
    loop {
        let file_name = format!("{prefix}.{date}.{sequence}");
        let file_path = prefix_dir.join(&file_name);
        match OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&file_path)
        {
            Ok(file) => return Ok((file, file_path, sequence)),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                sequence = sequence.saturating_add(1);
            }
            Err(err) => {
                return Err(anyhow!(
                    "failed to create log file {}: {}",
                    file_path.display(),
                    err
                ));
            }
        }
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

fn current_local_date_string(now: SystemTime) -> String {
    let now = OffsetDateTime::from(now);
    let now_local = UtcOffset::current_local_offset()
        .map(|offset| now.to_offset(offset))
        .unwrap_or(now);
    let date = now_local.date();
    format!(
        "{:04}{sep}{:02}{sep}{:02}",
        date.year(),
        u8::from(date.month()),
        date.day(),
        sep = LOG_FILE_DATE_SEPARATOR
    )
}

fn next_awake_sequence(log_dir: &Path, prefix: &str, date: &str) -> Result<u64> {
    let mut max_sequence = 0_u64;
    let entries = fs::read_dir(log_dir)
        .with_context(|| format!("failed to scan logging directory {}", log_dir.display()))?;
    for entry_result in entries {
        let entry = entry_result.with_context(|| {
            format!("failed to iterate logging directory {}", log_dir.display())
        })?;
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to stat {}", entry.path().display()))?;
        if !metadata.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if let Some(sequence) = parse_awake_sequence_from_file_name(&file_name, prefix, date) {
            max_sequence = max_sequence.max(sequence);
        }
    }
    Ok(max_sequence.saturating_add(1))
}

fn parse_awake_sequence_from_file_name(file_name: &str, prefix: &str, date: &str) -> Option<u64> {
    let file_name_prefix = format!("{prefix}.{date}.");
    let sequence = file_name.strip_prefix(&file_name_prefix)?;
    if sequence.is_empty() || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let parsed = sequence.parse::<u64>().ok()?;
    (parsed > 0).then_some(parsed)
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

    use super::{
        build_env_filter, next_awake_sequence, parse_awake_sequence_from_file_name,
        purge_old_log_files_at,
    };

    #[test]
    fn invalid_filter_is_rejected() {
        let err = build_env_filter("info,core==debug").expect_err("filter must fail");
        assert!(err.to_string().contains("logging.filter"));
    }

    #[test]
    fn retention_cleanup_only_removes_prefixed_files() {
        let dir = std::env::temp_dir().join(format!("beluna-logging-test-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let expired_log = dir.join("core.log.2026-02-01.1");
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

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_awake_sequence_accepts_new_file_name_pattern() {
        assert_eq!(
            parse_awake_sequence_from_file_name("core.log.2026-02-20.1", "core.log", "2026-02-20"),
            Some(1)
        );
        assert_eq!(
            parse_awake_sequence_from_file_name("core.log.2026-02-20", "core.log", "2026-02-20"),
            None
        );
        assert_eq!(
            parse_awake_sequence_from_file_name("core.log.2026-02-20.x", "core.log", "2026-02-20"),
            None
        );
        assert_eq!(
            parse_awake_sequence_from_file_name("core.log.2026-02-21.3", "core.log", "2026-02-20"),
            None
        );
    }

    #[test]
    fn next_awake_sequence_uses_max_existing_counter_for_same_date() {
        let dir = std::env::temp_dir().join(format!("beluna-logging-test-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("temp dir should exist");

        let files = [
            "core.log.2026-02-20.1",
            "core.log.2026-02-20.3",
            "core.log.2026-02-20.invalid",
            "core.log.2026-02-20",
            "core.log.2026-02-19.9",
            "other.log.2026-02-20.99",
        ];
        for file in files {
            fs::write(dir.join(file), "x").expect("test file should be created");
        }

        let next = next_awake_sequence(&dir, "core.log", "2026-02-20")
            .expect("sequence scan should succeed");
        assert_eq!(next, 4);

        let _ = fs::remove_dir_all(&dir);
    }
}
