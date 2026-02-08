use anyhow::Context;
use beluna::{cli::config_path_from_args, config::Config, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    server::run(config).await
}
