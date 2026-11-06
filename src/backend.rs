use std::fmt;
use std::io;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum BackendError {
    Spawn(io::Error),
    Failed { command: String, stderr: String },
    EventsEnded,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackendError::Spawn(error) => write!(f, "Failed to start nmcli: {error}"),
            BackendError::Failed { command, stderr } => {
                write!(f, "`{command}` failed: {}", stderr.trim())
            }
            BackendError::EventsEnded => write!(f, "Network monitor stopped"),
        }
    }
}

pub trait NetworkBackend {
    fn active_connections(&self) -> Result<Vec<String>, BackendError>;

    fn network_events(&self) -> Result<Receiver<()>, BackendError>;

    fn connect(&self, name: &str) -> Result<(), BackendError>;

    fn disconnect(&self, name: &str) -> Result<(), BackendError>;
}
