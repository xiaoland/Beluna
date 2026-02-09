use std::{fs, io::ErrorKind, os::unix::fs::FileTypeExt, path::Path};

use anyhow::{Context, Result, bail};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{UnixListener, UnixStream},
    signal::unix::{SignalKind, signal},
    sync::mpsc,
};

use crate::{
    config::Config,
    protocol::{ClientMessage, parse_client_message},
};

enum ExitReason {
    SocketMessage,
    Signal(&'static str),
}

pub async fn run(config: Config) -> Result<()> {
    prepare_socket_path(&config.socket_path)?;
    let listener = UnixListener::bind(&config.socket_path)
        .with_context(|| format!("unable to bind socket {}", config.socket_path.display()))?;

    let mut sigint =
        signal(SignalKind::interrupt()).context("unable to listen for SIGINT (Ctrl+C)")?;
    let mut sigterm = signal(SignalKind::terminate()).context("unable to listen for SIGTERM")?;
    let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<()>();

    eprintln!(
        "Beluna listening on unix socket (NDJSON): {}",
        config.socket_path.display()
    );

    let exit_reason = loop {
        tokio::select! {
            _ = sigint.recv() => break ExitReason::Signal("SIGINT"),
            _ = sigterm.recv() => break ExitReason::Signal("SIGTERM"),
            Some(()) = exit_rx.recv() => break ExitReason::SocketMessage,
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _)) => {
                        let sender = exit_tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) = handle_client(stream, sender).await {
                                eprintln!("client handling failed: {err:#}");
                            }
                        });
                    }
                    Err(err) => {
                        eprintln!("accept failed: {err}");
                    }
                }
            }
        }
    };

    cleanup_socket_path(&config.socket_path)?;
    match exit_reason {
        ExitReason::SocketMessage => eprintln!("Beluna stopped: received exit message"),
        ExitReason::Signal(signal_name) => eprintln!("Beluna stopped: received {signal_name}"),
    }

    Ok(())
}

async fn handle_client(stream: UnixStream, exit_tx: mpsc::UnboundedSender<()>) -> Result<()> {
    let mut lines = BufReader::new(stream).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_client_message(line) {
            Ok(ClientMessage::Exit) => {
                let _ = exit_tx.send(());
                break;
            }
            Err(err) => eprintln!("ignoring invalid protocol message: {err}"),
        }
    }

    Ok(())
}

fn prepare_socket_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create {}", parent.display()))?;
    }

    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_socket() || metadata.is_file() {
                fs::remove_file(path)
                    .with_context(|| format!("unable to remove stale socket {}", path.display()))?;
            } else {
                bail!(
                    "socket path exists but is not removable as file/socket: {}",
                    path.display()
                );
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("unable to inspect {}", path.display()));
        }
    }

    Ok(())
}

fn cleanup_socket_path(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("unable to remove {}", path.display())),
    }
}
