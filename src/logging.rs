use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use flexi_logger::{
    Cleanup, Criterion, Duplicate, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Naming,
    colored_default_format, detailed_format,
};

const MAX_FILE_BYTES: u64 = 1_000_000;
const KEPT_FILES: usize = 5;

pub fn log_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("logs")
}

pub fn init(dir: &Path) -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str("info")?
        .log_to_file(FileSpec::default().directory(dir).suppress_basename())
        .duplicate_to_stderr(Duplicate::All)
        .format_for_files(detailed_format)
        .format_for_stderr(colored_default_format)
        .rotate(
            Criterion::Size(MAX_FILE_BYTES),
            Naming::TimestampsDirect,
            Cleanup::KeepLogFiles(KEPT_FILES),
        )
        .start()
}

fn check_writable(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let probe = dir.join(".write_test");
    fs::write(&probe, b"")?;
    fs::remove_file(probe)
}

pub fn init_or_terminal_only(dir: &Path) -> Option<LoggerHandle> {
    check_writable(dir)
        .map_err(|error| error.to_string())
        .and_then(|()| init(dir).map_err(|error| error.to_string()))
        .inspect_err(|error| eprintln!("File logging disabled: {error}"))
        .or_else(|_| Logger::try_with_env_or_str("info")?.log_to_stderr().start())
        .inspect_err(|error| eprintln!("Logging disabled: {error}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn unwritable_dir_is_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("vpn_connector_ro_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();

        let result = check_writable(&dir.join("logs"));

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn writable_dir_is_accepted() {
        let dir = std::env::temp_dir().join(format!("vpn_connector_rw_{}", std::process::id()));
        assert!(check_writable(&dir).is_ok());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn init_creates_log_file() {
        let dir = std::env::temp_dir().join(format!("vpn_connector_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let mut handle = init(&dir).unwrap();
        handle.adapt_duplication_to_stderr(Duplicate::None).unwrap();
        log::info!("hello");
        handle.flush();

        let file = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|ext| ext == "log"))
            .expect("log file created");
        assert!(fs::read_to_string(file).unwrap().contains("hello"));

        fs::remove_dir_all(&dir).unwrap();
    }
}
