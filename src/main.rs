mod approval;
mod config;
mod events;
mod fetch;
mod model;

use anyhow::{Context, Result, bail};
use config::Config;
use events::GerritEventType;
use fetch::{FetchRequest, FetchWorker};
use ssh2::Session;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(20);
const READ_TIMEOUT: Duration = Duration::from_secs(5);

fn main() -> Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let cfg = Arc::new(Config::load(&config_path)?);
    let fetch_worker = FetchWorker::spawn();

    let tcp = TcpStream::connect((cfg.host.as_str(), cfg.port))
        .with_context(|| format!("connecting to {}:{}", cfg.host, cfg.port))?;
    tcp.set_read_timeout(Some(READ_TIMEOUT))
        .context("setting tcp read timeout")?;

    let mut session = Session::new().context("creating ssh session")?;
    session.set_tcp_stream(tcp);
    session.handshake().context("ssh handshake failed")?;

    session
        .userauth_pubkey_file(
            &cfg.username,
            cfg.public_key.as_deref(),
            &cfg.private_key,
            cfg.passphrase.as_deref(),
        )
        .context("ssh public key authentication failed")?;

    if !session.authenticated() {
        bail!("ssh authentication did not succeed");
    }

    session.set_keepalive(true, KEEPALIVE_INTERVAL.as_secs() as u32);

    let mut channel = session.channel_session().context("opening ssh channel")?;
    let command = cfg.stream_events_command();
    channel
        .exec(&command)
        .with_context(|| format!("executing '{command}'"))?;

    let mut stream = channel.stream(0);
    let mut buf = [0u8; 4096];
    let mut pending = String::new();
    let mut last_keepalive = Instant::now();

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                pending.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(pos) = pending.find('\n') {
                    let line = pending[..pos].trim().to_string();
                    pending.drain(..=pos);
                    if !line.is_empty() {
                        handle_line(&line, &cfg, &fetch_worker);
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("error reading from ssh stream: {e}");
                break;
            }
        }

        if last_keepalive.elapsed() >= KEEPALIVE_INTERVAL {
            match session.keepalive_send() {
                Ok(_) => last_keepalive = Instant::now(),
                Err(e) => {
                    eprintln!("warning: keepalive failed, connection may be dead: {e}");
                    break;
                }
            }
        }
    }

    let line = pending.trim();
    if !line.is_empty() {
        handle_line(line, &cfg, &fetch_worker);
    }

    let mut exit_status = 0;
    if channel.eof() {
        exit_status = channel.exit_status().unwrap_or(-1);
    }
    eprintln!("gerrit stream-events channel closed, exit status: {exit_status}");

    // Drain any queued fetches before exiting rather than abandoning them.
    fetch_worker.shutdown();

    Ok(())
}

fn handle_line(line: &str, cfg: &Config, fetch_worker: &FetchWorker) {
    match serde_json::from_str::<GerritEventType>(line) {
        Ok(event) => {
            // println!("{event:#?}");
            maybe_fetch(event, cfg, fetch_worker);
        }
        Err(e) => {
            eprintln!("warning: failed to deserialize event: {e}");
            println!("{line}");
        }
    }
}

fn maybe_fetch(event: GerritEventType, cfg: &Config, fetch_worker: &FetchWorker) {
    let (change, patch_set) = match event {
        GerritEventType::PatchsetCreated {
            change, patch_set, ..
        } => (change, patch_set),
        GerritEventType::ChangeMerged {
            change, patch_set, ..
        } => (change, patch_set),
        _ => return,
    };

    if change.project == cfg.project {
        fetch_worker.enqueue(FetchRequest {
            repo_dir: cfg.git_repo.clone(),
            remote: cfg.git_remote.clone(),
            ref_name: patch_set.ref_name,
        })
    }
}
