use std::io::{BufRead, BufReader};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::connector::Connector;

pub struct LinuxConnector;

impl Connector for LinuxConnector {
    fn active_connections(&self) -> Vec<String> {
        let Some(output) = run_nmcli(&[
            "-t",
            "-e",
            "no",
            "-f",
            "NAME",
            "connection",
            "show",
            "--active",
        ]) else {
            return Vec::new();
        };

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(String::from)
            .collect()
    }

    fn watch_network_changes(&self) -> Option<Receiver<()>> {
        let mut monitor = Command::new("nmcli")
            .arg("monitor")
            .stdout(Stdio::piped())
            .spawn()
            .inspect_err(|error| eprintln!("Failed to start nmcli monitor: {error}"))
            .ok()?;
        let stdout = monitor.stdout.take()?;

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for _ in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(()).is_err() {
                    break;
                }
            }
        });

        Some(receiver)
    }

    fn connect(&self, name: &str) -> bool {
        run_nmcli(&["connection", "up", name]).is_some_and(|output| output.status.success())
    }

    fn disconnect(&self, name: &str) -> bool {
        run_nmcli(&["connection", "down", name]).is_some_and(|output| output.status.success())
    }
}

fn run_nmcli(args: &[&str]) -> Option<Output> {
    Command::new("nmcli")
        .args(args)
        .output()
        .inspect_err(|error| eprintln!("Failed to execute nmcli: {error}"))
        .ok()
}
