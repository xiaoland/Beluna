use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};

pub enum CliCommand {
    Run { config_path: PathBuf },
    ConfigSchema { output: Option<PathBuf> },
}

pub fn command_from_args() -> Result<CliCommand> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Ok(CliCommand::Run {
            config_path: PathBuf::from("./beluna.jsonc"),
        });
    }

    match args[0].as_str() {
        "--config" => parse_run_command(&args),
        "config" => parse_config_command(&args),
        other => Err(anyhow!(
            "unknown argument: {other}. usage: beluna [--config <path>] | beluna config schema [--output <path>]"
        )),
    }
}

fn parse_run_command(args: &[String]) -> Result<CliCommand> {
    if args.len() != 2 {
        return Err(anyhow!(
            "invalid run arguments. usage: beluna [--config <path>]"
        ));
    }

    Ok(CliCommand::Run {
        config_path: PathBuf::from(&args[1]),
    })
}

fn parse_config_command(args: &[String]) -> Result<CliCommand> {
    if args.get(1).map(String::as_str) != Some("schema") {
        return Err(anyhow!(
            "unknown config subcommand. usage: beluna config schema [--output <path>]"
        ));
    }

    let mut output: Option<PathBuf> = None;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--output" => {
                let value = args.get(index + 1).ok_or_else(|| {
                    anyhow!(
                        "missing value for --output. usage: beluna config schema --output <path>"
                    )
                })?;
                output = Some(PathBuf::from(value));
                index += 2;
            }
            other => {
                return Err(anyhow!(
                    "unknown argument for schema command: {other}. usage: beluna config schema [--output <path>]"
                ));
            }
        }
    }

    Ok(CliCommand::ConfigSchema { output })
}
