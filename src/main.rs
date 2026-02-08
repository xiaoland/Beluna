mod ai_gateway;
mod cli;
mod config;
mod protocol;
mod server;

use anyhow::Context;
use cli::config_path_from_args;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    server::run(config).await
}
