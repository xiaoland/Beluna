use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};

pub fn config_path_from_args() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    let mut config_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --config"))?;
                config_path = Some(PathBuf::from(value));
            }
            other => {
                return Err(anyhow!(
                    "unknown argument: {other}. usage: beluna [--config <path>]"
                ));
            }
        }
    }

    Ok(config_path.unwrap_or_else(|| PathBuf::from("./beluna.jsonc")))
}
