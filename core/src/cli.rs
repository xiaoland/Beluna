use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};

pub fn config_path_from_args() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    let first = args.next();

    if args.next().is_some() {
        return Err(anyhow!(
            "expected at most one argument: <config-path>. Example: cargo run -- ./beluna.jsonc"
        ));
    }

    match first {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            let mut path = env::current_dir()?;
            path.push("beluna.jsonc");
            Ok(path)
        }
    }
}
