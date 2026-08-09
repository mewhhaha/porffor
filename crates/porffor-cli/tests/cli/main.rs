//! Shared helpers and module list for the `cli` integration target.
//!
//! The test bodies live in the sibling modules so that feature lanes
//! working on different areas do not all append to one file. They stay
//! child modules of a single target rather than separate `tests/*.rs`
//! files because each extra integration target statically links
//! `porffor_cli` and relinks a 143 MB binary.

mod array;
mod binary_data;
mod data_view;
mod date;
mod dynamic;
mod frontend;
mod functions;
mod heap;
mod iterator;
mod known_failures;
mod language;
mod object;
mod regexp;
mod string;
mod typed_array;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command as ProcessCommand;

/// Wall-clock bound on a single guarded `porf` subprocess.
///
/// Only the guarded path pays this; the in-process fast path is unchanged, so
/// the suite's cost is unchanged except for this one budget. Raising it is the
/// wrong response to a red run: a test that needs more than two minutes to
/// print a fixture's output is the defect.
const HANG_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Poll interval for the guarded path's `try_wait` loop.
const HANG_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// Stand-in name when libtest gives the current thread none at all.
const UNNAMED_THREAD: &str = "<unnamed libtest thread>";

struct Command {
    /// The `porf` binary every call site passes as `env!("CARGO_BIN_EXE_porf")`.
    ///
    /// It used to be discarded, because `output()` only ever ran the CLI in
    /// process. The guarded path needs a real program to spawn, and a test that
    /// blocks in process cannot be interrupted at all.
    program: std::ffi::OsString,
    args: Vec<String>,
}

struct CommandOutput {
    status: CommandStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct CommandStatus {
    success: bool,
}

impl CommandStatus {
    fn success(&self) -> bool {
        self.success
    }
}

impl Command {
    fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: program.as_ref().to_os_string(),
            args: Vec::new(),
        }
    }

    fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args.push(arg.as_ref().to_string_lossy().into_owned());
        self
    }

    /// Run `porf` and collect its output.
    ///
    /// Almost every test takes the in-process fast path: no spawn, no timeout,
    /// and the 143 MB binary is already linked into this process. The exception
    /// is a test the ledger declares a hang. `Atomics.wait` blocks the calling
    /// thread outright, so in process it consumes a libtest worker forever and
    /// the whole suite spins at 583 of 584 — which is why the documented
    /// invocation carried a `--skip` for three batches and rung 1c was never a
    /// gate. Running that one test as a real child, and killing the child after
    /// [`HANG_TIMEOUT`], turns the hang into an ordinary bounded failure that
    /// libtest can report and `should_panic` can pin.
    fn output(&mut self) -> io::Result<CommandOutput> {
        let thread = std::thread::current();
        let thread_name = thread.name().map(str::to_owned);
        match known_failures::execution_path(thread_name.as_deref()) {
            known_failures::ExecutionPath::InProcess => {
                let output = porffor_cli::run_cli_capture(self.args.clone());
                Ok(CommandOutput {
                    status: CommandStatus {
                        success: output.exit_code == 0,
                    },
                    stdout: output.stdout,
                    stderr: output.stderr,
                })
            }
            known_failures::ExecutionPath::GuardedSubprocess => {
                self.guarded_output(thread_name.as_deref().unwrap_or(UNNAMED_THREAD))
            }
        }
    }

    /// Spawn the real `porf` binary and bound it by wall clock.
    ///
    /// Panics rather than returning a synthetic failed [`CommandOutput`], and
    /// that is deliberate: the guarded test bodies assert with a bare
    /// `assert!(output.status.success())`, so a synthetic failure would panic
    /// with `assertion failed: output.status.success()` and the timeout message
    /// would never reach the `expected = ...` substring that makes the outcome
    /// checkable.
    fn guarded_output(&mut self, test_name: &str) -> io::Result<CommandOutput> {
        let mut child = ProcessCommand::new(&self.program)
            .args(&self.args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Drain both pipes on their own threads. A `try_wait` poll loop over
        // piped output deadlocks the moment the child fills a pipe buffer, and
        // some fixtures print more than that — which would look exactly like
        // the hang this path exists to bound.
        let mut child_stdout = child.stdout.take().expect("stdout was piped");
        let mut child_stderr = child.stderr.take().expect("stderr was piped");
        let stdout_reader = std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = io::Read::read_to_end(&mut child_stdout, &mut buffer);
            buffer
        });
        let stderr_reader = std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = io::Read::read_to_end(&mut child_stderr, &mut buffer);
            buffer
        });

        let started = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(CommandOutput {
                    status: CommandStatus {
                        success: status.success(),
                    },
                    stdout: stdout_reader.join().unwrap_or_default(),
                    stderr: stderr_reader.join().unwrap_or_default(),
                });
            }
            if started.elapsed() >= HANG_TIMEOUT {
                let _ = child.kill();
                let _ = child.wait();
                panic!(
                    "porf run exceeded {:?}: {} - the child was killed. If this test has no row in \
                     crates/porffor-cli/tests/known-failures.tsv it is a NEW hang: add a row with \
                     an owner, or fix it. Do not raise the timeout.",
                    HANG_TIMEOUT, test_name
                );
            }
            std::thread::sleep(HANG_POLL_INTERVAL);
        }
    }
}

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn suite_root() -> String {
    format!(
        "{}/../porffor-test262/tests/fixtures/fake_test262/vendor/test262",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn snapshot_dir() -> String {
    std::env::temp_dir()
        .join(format!("porffor-cli-test262-{}", std::process::id()))
        .display()
        .to_string()
}

fn unique_snapshot_dir(name: &str) -> String {
    std::env::temp_dir()
        .join(format!(
            "porffor-cli-test262-{}-{}-{}",
            name,
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
        .display()
        .to_string()
}

fn unique_project_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "porffor-cli-{}-{}-{}",
        name,
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp project should be created");
    root
}

#[cfg(feature = "spec-exec-oracle")]
fn copy_dir_recursive(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("copy destination should be created");
    for entry in fs::read_dir(from).expect("copy source should read") {
        let entry = entry.expect("copy source entry should read");
        let source = entry.path();
        let destination = to.join(entry.file_name());
        if source.is_dir() {
            copy_dir_recursive(&source, &destination);
        } else {
            fs::copy(&source, &destination).expect("copy file should succeed");
        }
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn copied_suite_root(name: &str) -> String {
    let root = unique_project_dir(name);
    let suite = root.join("vendor").join("test262");
    copy_dir_recursive(Path::new(&suite_root()), &suite);
    suite.display().to_string()
}

fn write_project_file(root: &Path, relative_path: &str, source: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test file parent should be created");
    fs::write(path, source).expect("test file should write");
}

fn tiny_wasm_suite_root(name: &str) -> String {
    let root = std::env::temp_dir().join(format!(
        "porffor-cli-tiny-test262-{}-{}-{}",
        name,
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let test_dir = root.join("test/language/wasm/pass");
    std::fs::create_dir_all(&test_dir).expect("tiny test262 wasm dir should be created");
    std::fs::write(
        test_dir.join("publish-status-wasm.js"),
        "/*---\nflags: [raw]\n---*/\n\n1 + 2;\n",
    )
    .expect("tiny test262 wasm case should write");
    root.display().to_string()
}

fn temp_readme_path(name: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "porffor-cli-readme-{}-{}-{}.md",
        std::process::id(),
        name,
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(
        &path,
        "# Porffor\n\n## Current Status\nold status\n\n## Design\nstill here\n",
    )
    .expect("temp readme should write");
    path.display().to_string()
}
