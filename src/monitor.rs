use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::{error, info, warn};

use crate::backend::{BackendError, NetworkBackend};
use crate::config::Config;
use crate::policy::{Action, decide};

const SETTLE: Duration = if cfg!(test) {
    Duration::ZERO
} else {
    Duration::from_secs(2)
};

pub fn run(backend: &impl NetworkBackend, config: &Config) -> Result<(), BackendError> {
    let events = backend.network_events()?;
    loop {
        check_once(backend, config)?;
        wait_for_change(&events, SETTLE)?;
    }
}

fn check_once(backend: &impl NetworkBackend, config: &Config) -> Result<(), BackendError> {
    info!("Checking connection status...");

    let active = backend.active_connections().unwrap_or_else(|error| {
        warn!("{error}");
        Vec::new()
    });

    match decide(&active, config) {
        Action::Stay(reason) => info!("Status OK: {reason}"),
        Action::Disconnect => {
            info!("Connected to VPN and home network. VPN will be disabled");
            if let Err(error) = backend.disconnect(&config.vpn) {
                warn!("Failed to disconnect from VPN: {error}");
            }
        }
        Action::Connect => {
            warn!("Status NOT CONNECTED. Trying to connect...");
            backend.connect(&config.vpn).inspect_err(|error| {
                error!("Failed to connect to VPN (check VPN name): {error}");
            })?;
            info!("Successfully connected to VPN");
        }
    }
    Ok(())
}

fn wait_for_change(events: &Receiver<()>, settle: Duration) -> Result<(), BackendError> {
    events.recv().map_err(|_| BackendError::EventsEnded)?;
    while events.recv_timeout(settle).is_ok() {}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::sync::mpsc;

    struct FakeBackend {
        active: Option<Vec<String>>,
        events: RefCell<Option<Receiver<()>>>,
        connect_ok: bool,
        disconnect_ok: bool,
        checks: Cell<usize>,
        connected: RefCell<Vec<String>>,
    }

    impl FakeBackend {
        fn new(active: &[&str], event_count: usize) -> Self {
            let (sender, receiver) = mpsc::channel();
            for _ in 0..event_count {
                sender.send(()).unwrap();
            }
            FakeBackend {
                active: Some(active.iter().map(|name| name.to_string()).collect()),
                events: RefCell::new(Some(receiver)),
                connect_ok: true,
                disconnect_ok: true,
                checks: Cell::new(0),
                connected: RefCell::new(Vec::new()),
            }
        }
    }

    fn failed() -> BackendError {
        BackendError::Failed {
            command: "fake".into(),
            stderr: String::new(),
        }
    }

    impl NetworkBackend for FakeBackend {
        fn active_connections(&self) -> Result<Vec<String>, BackendError> {
            self.checks.set(self.checks.get() + 1);
            self.active.clone().ok_or_else(failed)
        }

        fn network_events(&self) -> Result<Receiver<()>, BackendError> {
            self.events.borrow_mut().take().ok_or_else(|| {
                BackendError::Spawn(std::io::Error::from(std::io::ErrorKind::NotFound))
            })
        }

        fn connect(&self, name: &str) -> Result<(), BackendError> {
            self.connected.borrow_mut().push(name.into());
            if self.connect_ok {
                Ok(())
            } else {
                Err(failed())
            }
        }

        fn disconnect(&self, _name: &str) -> Result<(), BackendError> {
            if self.disconnect_ok {
                Ok(())
            } else {
                Err(failed())
            }
        }
    }

    fn config() -> Config {
        Config {
            vpn: "vpn".into(),
            home_network: "home".into(),
            trusted_networks: vec![],
        }
    }

    fn run_fake(backend: &FakeBackend) -> Result<(), BackendError> {
        run(backend, &config())
    }

    #[test]
    fn run_fails_when_events_unavailable() {
        let backend = FakeBackend::new(&["cafe"], 0);
        backend.events.replace(None);
        assert!(matches!(run_fake(&backend), Err(BackendError::Spawn(_))));
        assert!(backend.connected.borrow().is_empty());
    }

    #[test]
    fn run_fails_when_events_end() {
        let backend = FakeBackend::new(&["home"], 0);
        assert!(matches!(run_fake(&backend), Err(BackendError::EventsEnded)));
        assert_eq!(backend.checks.get(), 1);
    }

    #[test]
    fn burst_of_events_triggers_one_check() {
        let backend = FakeBackend::new(&["home"], 3);
        assert!(matches!(run_fake(&backend), Err(BackendError::EventsEnded)));
        assert_eq!(backend.checks.get(), 2);
    }

    #[test]
    fn connect_failure_stops() {
        let mut backend = FakeBackend::new(&["cafe"], 5);
        backend.connect_ok = false;
        assert!(matches!(
            run_fake(&backend),
            Err(BackendError::Failed { .. })
        ));
        assert_eq!(backend.checks.get(), 1);
    }

    #[test]
    fn disconnect_failure_keeps_running() {
        let mut backend = FakeBackend::new(&["home", "vpn"], 1);
        backend.disconnect_ok = false;
        assert!(matches!(run_fake(&backend), Err(BackendError::EventsEnded)));
        assert_eq!(backend.checks.get(), 2);
    }

    #[test]
    fn listing_failure_tries_vpn() {
        let mut backend = FakeBackend::new(&[], 0);
        backend.active = None;
        let _ = run_fake(&backend);
        assert_eq!(*backend.connected.borrow(), ["vpn"]);
    }
}
