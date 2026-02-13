use anyhow::{Context, Result};

use beluna::{cli::config_path_from_args, config::Config, core_loop};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    core_loop::run(config).await
}
