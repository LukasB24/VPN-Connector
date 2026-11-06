use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::backend::{BackendError, NetworkBackend};

#[rustfmt::skip]
const ACTIVE_NAMES_ARGS: &[&str] = &["-t", "-e", "no", "-f", "NAME", "connection", "show", "--active"];

pub struct Nmcli;

impl NetworkBackend for Nmcli {
    fn active_connections(&self) -> Result<Vec<String>, BackendError> {
        let stdout = run(ACTIVE_NAMES_ARGS)?;
        Ok(stdout.lines().map(String::from).collect())
    }

    fn network_events(&self) -> Result<Receiver<()>, BackendError> {
        let mut monitor = Command::new("nmcli")
            .arg("monitor")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(BackendError::Spawn)?;
        let stdout = monitor.stdout.take().ok_or(BackendError::EventsEnded)?;

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for _line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(()).is_err() {
                    break;
                }
            }
        });

        Ok(receiver)
    }

    fn connect(&self, name: &str) -> Result<(), BackendError> {
        run(&["connection", "up", name]).map(drop)
    }

    fn disconnect(&self, name: &str) -> Result<(), BackendError> {
        run(&["connection", "down", name]).map(drop)
    }
}

fn run(args: &[&str]) -> Result<String, BackendError> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(BackendError::Spawn)?;

    if !output.status.success() {
        return Err(BackendError::Failed {
            command: format!("nmcli {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
