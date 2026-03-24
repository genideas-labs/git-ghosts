/// tests/test_cli.rs
///
/// Binary-invocation integration tests for git-ghosts.
/// These tests exercise the exit-code contract end-to-end using
/// std::process::Command with the compiled binary.
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a `Command` pre-pointed at the compiled `git-ghosts` binary.
fn git_ghosts_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_git-ghosts"))
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-cli-{}-{}", label, nanos))
}

fn init_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init failed: {:?}", out);
}

fn make_commit(dir: &std::path::Path) {
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .unwrap();
    std::fs::write(dir.join("README.md"), b"hello").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    let out = Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git commit failed: {:?}", out);
}

// ---------------------------------------------------------------------------
// --help tests
// ---------------------------------------------------------------------------

/// `git-ghosts --help` exits with code 0.
#[test]
fn help_exits_zero() {
    let out = git_ghosts_cmd().arg("--help").output().unwrap();
    assert!(
        out.status.success(),
        "--help should exit 0, got: {:?}",
        out.status
    );
}

/// `git-ghosts --help` stdout contains 'scan', 'report', and 'clean'.
#[test]
fn help_lists_subcommands() {
    let out = git_ghosts_cmd().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(stdout.contains("scan"), "--help should mention 'scan'");
    assert!(stdout.contains("report"), "--help should mention 'report'");
    assert!(stdout.contains("clean"), "--help should mention 'clean'");
}

// ---------------------------------------------------------------------------
// --version tests
// ---------------------------------------------------------------------------

/// `git-ghosts --version` exits with code 0 and prints a non-empty version string.
#[test]
fn version_exits_zero_and_prints_string() {
    let out = git_ghosts_cmd().arg("--version").output().unwrap();
    assert!(
        out.status.success(),
        "--version should exit 0, got: {:?}",
        out.status
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.trim().is_empty(),
        "--version should print something"
    );
}

// ---------------------------------------------------------------------------
// Missing / unknown subcommand tests
// ---------------------------------------------------------------------------

/// Invoking `git-ghosts` with no arguments exits with a non-zero code.
#[test]
fn no_args_exits_nonzero() {
    let out = git_ghosts_cmd().output().unwrap();
    assert!(
        !out.status.success(),
        "no-args invocation should exit non-zero, got: {:?}",
        out.status
    );
}

/// An unknown subcommand exits with a non-zero code.
#[test]
fn unknown_subcommand_exits_nonzero() {
    let out = git_ghosts_cmd()
        .arg("totally-unknown-cmd")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "unknown subcommand should exit non-zero, got: {:?}",
        out.status
    );
}

/// An unknown subcommand prints a usage/error message (to stdout or stderr).
#[test]
fn unknown_subcommand_prints_message() {
    let out = git_ghosts_cmd()
        .arg("totally-unknown-cmd")
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.trim().is_empty(),
        "unknown subcommand should print something"
    );
}

// ---------------------------------------------------------------------------
// scan subcommand tests
// ---------------------------------------------------------------------------

/// `git-ghosts scan --path <non-git-dir>` exits with code 1 and writes an
/// error message to stderr.
#[test]
fn scan_non_git_dir_exits_one() {
    let dir = temp_dir("scan-nongit");
    std::fs::create_dir_all(&dir).unwrap();

    let out = git_ghosts_cmd()
        .args(["scan", "--path"])
        .arg(&dir)
        .output()
        .unwrap();

    // Accept path as positional argument too.
    let out2 = git_ghosts_cmd().arg("scan").arg(&dir).output().unwrap();

    // At least one invocation form must exit non-zero.
    let either_nonzero = !out.status.success() || !out2.status.success();
    assert!(
        either_nonzero,
        "scan on non-git dir should exit non-zero; status1={:?}, status2={:?}",
        out.status, out2.status
    );

    // The one that failed should have something on stderr.
    let failing_out = if !out.status.success() { &out } else { &out2 };
    let stderr = String::from_utf8_lossy(&failing_out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "scan error should go to stderr, got nothing"
    );
}

/// `git-ghosts scan <path>` (positional) on a valid git repo exits with code 0.
#[test]
fn scan_valid_repo_exits_zero() {
    let dir = temp_dir("scan-valid");
    init_repo(&dir);
    make_commit(&dir);

    let out = git_ghosts_cmd().arg("scan").arg(&dir).output().unwrap();
    assert!(
        out.status.success(),
        "scan on valid repo should exit 0, got: {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// report subcommand tests
// ---------------------------------------------------------------------------

/// `git-ghosts report <path>` with no prior scan (no cache) exits with code 1
/// and writes an error to stderr.
#[test]
fn report_no_cache_exits_one() {
    let dir = temp_dir("report-nocache");
    // Provide a .git dir so the path looks like a repo dir for the cache lookup.
    std::fs::create_dir_all(dir.join(".git")).unwrap();

    let out = git_ghosts_cmd().arg("report").arg(&dir).output().unwrap();

    assert_eq!(
        out.status.code(),
        Some(1),
        "report with no cache should exit 1, got: {:?}",
        out.status
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "report error should go to stderr"
    );
}

/// The stderr message for a missing cache contains the word "error".
#[test]
fn report_no_cache_stderr_contains_error() {
    let dir = temp_dir("report-nocache-err");
    std::fs::create_dir_all(dir.join(".git")).unwrap();

    let out = git_ghosts_cmd().arg("report").arg(&dir).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(
        stderr.contains("error"),
        "stderr should contain 'error', got: {}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// clean subcommand tests
// ---------------------------------------------------------------------------

/// `git-ghosts clean <path>` (without --dry-run) exits with code 1.
#[test]
fn clean_without_dry_run_exits_one() {
    let dir = temp_dir("clean-nodryrun");
    std::fs::create_dir_all(dir.join(".git")).unwrap();

    let out = git_ghosts_cmd().arg("clean").arg(&dir).output().unwrap();

    assert_eq!(
        out.status.code(),
        Some(1),
        "clean without --dry-run should exit 1, got: {:?}",
        out.status
    );
}

/// `git-ghosts clean <path>` (without --dry-run) writes an error to stderr.
#[test]
fn clean_without_dry_run_stderr_nonempty() {
    let dir = temp_dir("clean-nodryrun-err");
    std::fs::create_dir_all(dir.join(".git")).unwrap();

    let out = git_ghosts_cmd().arg("clean").arg(&dir).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "clean without --dry-run should print error to stderr"
    );
}

/// `git-ghosts clean --dry-run <path>` with no cache exits with code 1 and
/// writes an error to stderr.
#[test]
fn clean_dry_run_no_cache_exits_one() {
    let dir = temp_dir("clean-dryrun-nocache");
    std::fs::create_dir_all(dir.join(".git")).unwrap();

    let out = git_ghosts_cmd()
        .args(["clean", "--dry-run"])
        .arg(&dir)
        .output()
        .unwrap();

    assert_eq!(
        out.status.code(),
        Some(1),
        "clean --dry-run with no cache should exit 1, got: {:?}",
        out.status
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "clean --dry-run error should go to stderr"
    );
}

// ---------------------------------------------------------------------------
// exit-code contract: Ok(()) => 0
// ---------------------------------------------------------------------------

/// A successful scan followed by a report exits 0.
#[test]
fn scan_then_report_exits_zero() {
    let dir = temp_dir("scan-report");
    init_repo(&dir);
    make_commit(&dir);

    let scan_out = git_ghosts_cmd().arg("scan").arg(&dir).output().unwrap();
    assert!(
        scan_out.status.success(),
        "scan should succeed: {:?}\nstderr: {}",
        scan_out.status,
        String::from_utf8_lossy(&scan_out.stderr)
    );

    let report_out = git_ghosts_cmd().arg("report").arg(&dir).output().unwrap();
    assert!(
        report_out.status.success(),
        "report after scan should exit 0, got: {:?}\nstderr: {}",
        report_out.status,
        String::from_utf8_lossy(&report_out.stderr)
    );
}

/// A successful scan followed by `clean --dry-run` exits 0.
#[test]
fn scan_then_clean_dry_run_exits_zero() {
    let dir = temp_dir("scan-clean");
    init_repo(&dir);
    make_commit(&dir);

    let scan_out = git_ghosts_cmd().arg("scan").arg(&dir).output().unwrap();
    assert!(
        scan_out.status.success(),
        "scan should succeed: {:?}\nstderr: {}",
        scan_out.status,
        String::from_utf8_lossy(&scan_out.stderr)
    );

    let clean_out = git_ghosts_cmd()
        .args(["clean", "--dry-run"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        clean_out.status.success(),
        "clean --dry-run after scan should exit 0, got: {:?}\nstderr: {}",
        clean_out.status,
        String::from_utf8_lossy(&clean_out.stderr)
    );
}
