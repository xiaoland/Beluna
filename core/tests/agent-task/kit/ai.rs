use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use tokio::{process::Child, process::Command, time::sleep};

pub struct AimockBoundary {
    child: Child,
    base_url: String,
    origin_url: String,
    fixtures_path: PathBuf,
}

pub fn render_fixture_tree(
    source: &Path,
    target: &Path,
    replacements: &[(&str, String)],
) -> Result<PathBuf> {
    if target.exists() {
        fs::remove_dir_all(target).with_context(|| {
            format!("failed to clear rendered fixture dir {}", target.display())
        })?;
    }
    fs::create_dir_all(target)
        .with_context(|| format!("failed to create rendered fixture dir {}", target.display()))?;
    render_fixture_tree_inner(source, target, replacements)?;
    Ok(target.to_path_buf())
}

impl AimockBoundary {
    pub async fn start(fixtures_path: &Path) -> Result<Self> {
        if !fixtures_path.exists() {
            bail!(
                "AIMock fixture path does not exist: {}",
                fixtures_path.display()
            );
        }

        let port = reserve_local_port()?;
        let origin_url = format!("http://127.0.0.1:{port}");
        let base_url = format!("{origin_url}/v1");

        let mut child = Command::new("npx")
            .arg("-p")
            .arg("@copilotkit/aimock")
            .arg("llmock")
            .arg("-f")
            .arg(fixtures_path)
            .arg("-p")
            .arg(port.to_string())
            .arg("-h")
            .arg("127.0.0.1")
            .arg("--strict")
            .arg("--validate-on-load")
            .arg("--log-level")
            .arg("silent")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to spawn AIMock through npx")?;

        wait_until_ready(&mut child, &origin_url).await?;

        Ok(Self {
            child,
            base_url,
            origin_url,
            fixtures_path: fixtures_path.to_path_buf(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn origin_url(&self) -> &str {
        &self.origin_url
    }

    pub fn fixtures_path(&self) -> &Path {
        &self.fixtures_path
    }
}

impl Drop for AimockBoundary {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn reserve_local_port() -> Result<u16> {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).context("failed to reserve a local AIMock port")?;
    let port = listener
        .local_addr()
        .context("failed to read reserved local AIMock port")?
        .port();
    drop(listener);
    Ok(port)
}

async fn wait_until_ready(child: &mut Child, origin_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let health_url = format!("{origin_url}/health");

    for _ in 0..100 {
        if let Some(status) = child
            .try_wait()
            .context("failed to inspect AIMock process status")?
        {
            bail!("AIMock exited before readiness check: {status}");
        }

        if let Ok(response) = client.get(&health_url).send().await
            && response.status().is_success()
        {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    bail!("AIMock did not become ready at {health_url}");
}

fn render_fixture_tree_inner(
    source: &Path,
    target: &Path,
    replacements: &[(&str, String)],
) -> Result<()> {
    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read fixture dir {}", source.display()))?
    {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            fs::create_dir_all(&target_path).with_context(|| {
                format!(
                    "failed to create rendered fixture subdir {}",
                    target_path.display()
                )
            })?;
            render_fixture_tree_inner(&source_path, &target_path, replacements)?;
            continue;
        }
        if metadata.is_file() {
            match fs::read_to_string(&source_path) {
                Ok(mut content) => {
                    for (from, to) in replacements {
                        content = content.replace(from, to);
                    }
                    fs::write(&target_path, content).with_context(|| {
                        format!("failed to write rendered fixture {}", target_path.display())
                    })?;
                }
                Err(_) => {
                    fs::copy(&source_path, &target_path).with_context(|| {
                        format!("failed to copy rendered fixture {}", source_path.display())
                    })?;
                }
            }
        }
    }
    Ok(())
}
