use anyhow::{Context, Result};

use beluna::{brainstem, cli::config_path_from_args, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    brainstem::run(config).await
}
