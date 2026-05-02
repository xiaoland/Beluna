use std::{
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
