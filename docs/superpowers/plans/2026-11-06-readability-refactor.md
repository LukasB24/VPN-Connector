# Readability Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the VPN connector readable by splitting it by responsibility, fixing misleading names, and replacing `bool`/`Option` failures with `Result`, without changing runtime behavior.

**Architecture:** `main` parses a `Config`, waits for startup, and calls `monitor::run`. `monitor` asks `policy::decide` (pure) what to do, applies it through a `NetworkBackend` trait (implemented by `Nmcli`), and reports through `ui`. Each file has one job.

**Tech Stack:** Rust 2021, `log` 0.4 (standard logging facade) + `flexi_logger` 0.29 (file backend with rotation), `nmcli` at runtime.

**Spec:** Readability review from this conversation (no separate doc): split `connector.rs`, rename to connection names / `NetworkBackend`, return `Result`, document the debounce, keep behavior identical. Addition: all output goes through the standard `log` crate and is written to a `logs/` directory, and also shown on the terminal.

## Global Constraints

- Only new dependencies: `log = "0.4"`, `flexi_logger = "0.29"`.
- Log directory: `logs/` inside the project directory, resolved at compile time via `env!("CARGO_MANIFEST_DIR")` (independent of the working directory); file name `r<timestamp>.log`; rotate at 1 MB, keep 5 files; default level `info`, overridable with `RUST_LOG`.
- Every log line also goes to the terminal (stderr) with colored level, replacing the hand-written ANSI codes.
- Behavior unchanged: 60 s startup wait, 2 s settle, same decision rules, same CLI args (`<vpn> <home> [trusted...]`).
- A failed VPN connect still ends the program with a failure exit code; a failed disconnect is logged and monitoring continues.
- `cargo build`, `cargo test`, `cargo clippy -- -D warnings` pass after every task.
- Names refer to NetworkManager **connection names**, not SSIDs.

## Review Focus

1. `nmcli` not installed → program prints the spawn error and exits non-zero (no panic). Covered in Task 4 (`run_fails_when_events_unavailable`).
2. Event stream ends (`nmcli monitor` dies) → exits non-zero with "Network monitor stopped". Task 4 (`run_fails_when_events_end`).
3. Burst of events → only one extra check per burst. Task 4 (`burst_of_events_triggers_one_check`).
4. Disconnect fails at home → keeps monitoring. Task 4 (`disconnect_failure_keeps_running`).
5. `active_connections` fails → treated as "no connections" → tries VPN (current behavior). Task 4 (`listing_failure_tries_vpn`).

---

## File Structure

| File | Responsibility |
|---|---|
| `src/config.rs` | `Config` + parsing CLI args |
| `src/policy.rs` | Pure decision: active connections → `Action` |
| `src/backend.rs` | `NetworkBackend` trait + `BackendError` |
| `src/nmcli.rs` | `Nmcli` backend (replaces `linux_connector.rs`) |
| `src/monitor.rs` | Check → apply → wait-for-change loop |
| `src/logging.rs` | `log` + `flexi_logger` setup (file in `logs/` + terminal) |
| `src/main.rs` | Wiring + exit code |

`src/connector.rs` and `src/linux_connector.rs` are deleted by the end.

---

### Task 1: Config

**Files:** Create `src/config.rs`; Modify `src/main.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct Config { pub vpn: String, pub home_network: String, pub trusted_networks: Vec<String> }
  impl Config { pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Config> } // args WITHOUT program name
  ```

- [ ] **Step 1: Tests in `src/config.rs`:** `parses_vpn_home_and_trusted` (`["vpn","home","a","b"]` → fields match, `trusted_networks == ["a","b"]`), `trusted_is_optional` (`["vpn","home"]` → empty vec), `rejects_missing_home` (`["vpn"]` → `None`).
- [ ] **Step 2:** `cargo test config` → FAIL (not defined).
- [ ] **Step 3:** Implement `Config::from_args`. In `main.rs`, use it (`Config::from_args(env::args().skip(1))`) and keep passing a value the old `run_vpn_connector` accepts (temporarily convert or keep `VpnConfig` until Task 2 removes it).
- [ ] **Step 4:** `cargo test && cargo clippy -- -D warnings` → PASS.
- [ ] **Step 5:** Commit `refactor: extract Config with arg parsing`.

### Task 2: Policy

**Files:** Create `src/policy.rs`; Modify `src/connector.rs` (remove `decide`, `Action`, tests, `VpnConfig`; use `Config`)

**Interfaces:**
- Consumes: `Config` (Task 1).
- Produces:
  ```rust
  pub enum Action { Connect, Disconnect, Stay(Reason) }
  pub enum Reason { AtHome(String), OnVpn, OnTrusted(String) }   // derive Debug, PartialEq
  impl std::fmt::Display for Reason   // "Connected to home network: {0}" / "Connected to VPN" / "Connected to secure network: {0}"
  pub fn decide(active: &[String], config: &Config) -> Action
  ```
  `Reason` replaces the old message strings.

- [ ] **Step 1:** Port the six existing tests with exact assertions: `["home","vpn"]`→`Disconnect`; `["home"]`→`Stay(AtHome("home"))`; `["cafe","vpn"]`→`Stay(OnVpn)`; `["office"]`→`Stay(OnTrusted("office"))`; `["cafe"]`→`Connect`; `[]`→`Connect`. Use `assert_eq!`. Add `reason_display` asserting the three exact strings above.
- [ ] **Step 2:** `cargo test policy` → FAIL.
- [ ] **Step 3:** Implement `decide` with the same rule order as today (home first, then VPN, then trusted).
- [ ] **Step 4:** `cargo test && cargo clippy -- -D warnings` → PASS.
- [ ] **Step 5:** Commit `refactor: move decision logic to pure policy module`.

### Task 3: Backend trait and nmcli

**Files:** Create `src/backend.rs`, `src/nmcli.rs`; Delete `src/linux_connector.rs`; Modify `src/connector.rs`, `src/main.rs`

**Interfaces:**
- Produces:
  ```rust
  pub enum BackendError { Spawn(std::io::Error), Failed { command: String, stderr: String }, EventsEnded }
  impl std::fmt::Display for BackendError
  pub trait NetworkBackend {
      fn active_connections(&self) -> Result<Vec<String>, BackendError>;
      fn network_events(&self) -> Result<Receiver<()>, BackendError>;
      fn connect(&self, name: &str) -> Result<(), BackendError>;
      fn disconnect(&self, name: &str) -> Result<(), BackendError>;
  }
  pub struct Nmcli;   // impl NetworkBackend
  ```

- [ ] **Step 1:** Implement `Nmcli` with one private helper `fn run(args: &[&str]) -> Result<String, BackendError>` returning stdout on success, `Failed` (with lossy stderr) on non-zero exit. Put the list args in `const ACTIVE_NAMES_ARGS: &[&str] = &["-t","-e","no","-f","NAME","connection","show","--active"];`. No unit tests here (shells out); old `connector.rs` call sites adapt (`.is_ok()` / `.unwrap_or_default()`) so behavior is unchanged.
- [ ] **Step 2:** `cargo build && cargo test && cargo clippy -- -D warnings` → PASS.
- [ ] **Step 3:** Commit `refactor: NetworkBackend trait with Result errors, nmcli backend`.

### Task 4: Monitor loop

**Files:** Create `src/monitor.rs`; Delete `src/connector.rs`; Modify `src/main.rs`

**Interfaces:**
- Consumes: `Config`, `decide`/`Action`, `NetworkBackend`/`BackendError`.
- Produces:
  ```rust
  pub fn run(backend: &impl NetworkBackend, config: &Config) -> Result<(), BackendError> // only returns on error
  ```
  Internals: `fn check_once(..) -> Result<(), BackendError>` (connect error is returned; disconnect error is printed and swallowed; listing error → empty list) and `fn wait_for_change(events: &Receiver<()>) -> Result<(), BackendError>` (block for one event, sleep `SETTLE`, then drain queued events). Comment on the drain: *events arrive in bursts; we check once per burst*. Make `SETTLE: Duration` a parameter of an inner `run_with_settle` so tests pass `Duration::ZERO`.
  Loop shape: `let events = backend.network_events()?; loop { check_once(..)?; wait_for_change(&events)?; }`.

- [ ] **Step 1:** Tests with a `FakeBackend` (`RefCell<Vec<String>>` active list, scripted connect/disconnect results, call log, and a pre-filled channel whose sender is dropped so the stream ends):
  - `run_fails_when_events_unavailable` → `network_events` returns `Err(Spawn)` → `run` returns that error, no connect calls.
  - `run_fails_when_events_end` → zero events → exactly one check, then `Err(EventsEnded)`.
  - `burst_of_events_triggers_one_check` → 3 events pre-queued → 2 checks total, then `EventsEnded`.
  - `connect_failure_stops` → active `["cafe"]`, connect fails → `Err(Failed{..})` after 1 check.
  - `disconnect_failure_keeps_running` → active `["home","vpn"]`, disconnect fails, 1 event → 2 checks, ends with `EventsEnded`.
  - `listing_failure_tries_vpn` → `active_connections` errs → `connect("vpn")` called.
- [ ] **Step 2:** `cargo test monitor` → FAIL.
- [ ] **Step 3:** Implement `monitor`, reporting with `log` macros (`info!` for checks/status using `Reason`'s `Display`, `warn!` for disconnect failure, `error!` for connect failure). Add `log = "0.4"` to `Cargo.toml` here; without a logger installed the macros are no-ops, so tests stay silent. Point `main` at `monitor::run`; delete `connector.rs`.
- [ ] **Step 4:** `cargo test && cargo clippy -- -D warnings` → PASS.
- [ ] **Step 5:** Commit `refactor: monitor loop with explicit check/wait steps`.

### Task 5: Logging module and main wiring

**Files:** Create `src/logging.rs`; Modify `Cargo.toml`, `src/main.rs`, `README.md`, `.gitignore`

**Interfaces:**
- Consumes: `monitor::run`, `Config::from_args`, `Nmcli`.
- Produces:
  ```rust
  pub fn init(dir: &Path) -> Result<flexi_logger::LoggerHandle, flexi_logger::FlexiLoggerError>
  pub fn log_dir() -> PathBuf  // Path::new(env!("CARGO_MANIFEST_DIR")).join("logs")
  ```
  Use `Logger::try_with_env_or_str("info")`, `.log_to_file(FileSpec::default().directory(dir).basename("vpn_connector"))`, `.duplicate_to_stderr(Duplicate::All)`, `.format_for_files(detailed_format)`, `.format_for_stderr(colored_default_format)`, `.rotate(Criterion::Size(1_000_000), Naming::Timestamps, Cleanup::KeepLogFiles(5))`, `.start()`. `main` must hold the returned handle until exit (dropping it flushes/stops logging).

- [ ] **Step 1:** Test `init_creates_log_file`: init into a temp dir under `std::env::temp_dir()`, `log::info!("hello")`, `handle.flush()`, assert a file in that dir starts with `vpn_connector` and contains `hello`. (Only one logger per process: keep this the only test that calls `init`.)
- [ ] **Step 2:** `cargo test logging` → FAIL; add `flexi_logger = "0.29"`; implement; PASS.
- [ ] **Step 3:** `main`: parse or print usage to stderr and return `FAILURE` (usage before logger so it is not logged); `let _log = logging::init(&logging::log_dir())` (on error print it and continue without file logging); `info!` startup wait; sleep `STARTUP_WAIT`; `if let Err(e) = monitor::run(&Nmcli, &config) { error!("{e}") }`; return `FAILURE`. Rewrite usage text as plain lines (no `\` continuations). Remove the old ANSI color constants. Add `logs/` to `.gitignore`. README: "connection name" instead of "SSID/wifi name", and a note that logs go to `logs/` in the project directory (the binary must not be moved to another machine without rebuilding) and `RUST_LOG=debug` increases verbosity.
- [ ] **Step 4:** `cargo test && cargo clippy -- -D warnings`; `cargo run` (no args) → usage, exit 1; `cargo run -- x y` → colored lines on terminal and `logs/vpn_connector*.log` created.
- [ ] **Step 5:** Commit `feat: log to logs/ via log + flexi_logger; simplify main`.
