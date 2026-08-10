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
mod throw_propagation;
mod typed_array;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command as ProcessCommand;

/// Wall-clock bound on a single `porf` invocation, on **both** execution paths.
///
/// # Why 900 s and not two minutes
///
/// The previous value was 120 s, justified as "a test that needs more than two
/// minutes to print a fixture's output is the defect". That justification is
/// contradicted by this repository's own calibration two files over:
/// `README.md` and `docs/rust-rewrite/batch-workflow.md` both record that on a
/// 4-CPU box with a sweep holding two of them **a single cold Wasm-AOT compile
/// can exceed the 300 s** `run-watched.sh` default, which is exactly why the
/// documented rung-1c invocation passes `--stall 900`. The guarded child is a
/// cold `porf` spawned from scratch and the clock starts at spawn, so the whole
/// parse → lower → emit → Cranelift path is inside this budget before the
/// fixture's first `Atomics.wait` is reached. At 120 s a loaded box times out on
/// a *correct* run, the declared-hang row goes green either way, and the "test
/// did not panic as expected" stale-baseline signal — the entire payoff of
/// declaring the hang — never fires.
///
/// 900 s is the same headroom `--stall 900` already buys, chosen from the same
/// measurement rather than invented. It is a **termination** bound, not a
/// performance assertion: the suite pays it only when something is actually
/// stuck, and there is exactly one declared hang.
///
/// # What a timeout here does and does not prove
///
/// Be precise, because `#[should_panic(expected = "porf run exceeded")]` asserts
/// on *this* message and not on anything about blocking.
/// `wasm_atomics_wait_core.js` prints nothing before it blocks, so "no output
/// before the deadline" cannot separate "blocked in `Atomics.wait`" from "still
/// compiling". The calibration above is the only thing separating them. Read a
/// timeout on a test with **no** ledger row as "hung *or* pathologically slow",
/// investigate before adding a row, and do not raise this constant as the
/// response to a red run — recalibrate it against a measured cold compile on the
/// box in question and record that measurement in the ledger's evidence column.
const HANG_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(900);

/// Poll interval for the guarded path's `try_wait` loop.
const HANG_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// Stack for the in-process worker thread introduced by [`Command::output`]'s
/// bounded in-process path.
///
/// Generous on purpose. libtest gives its worker threads `RUST_MIN_STACK` or
/// std's default, and the in-process CLI call used to run directly on that
/// thread; moving it onto a thread of our own must not shrink the stack it gets,
/// so this is set well above either. Linux commits thread stacks lazily, so the
/// cost is address space, not memory.
const IN_PROCESS_STACK_SIZE: usize = 64 * 1024 * 1024;

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
    /// Almost every test takes the in-process path: no process spawn, and the
    /// 143 MB binary is already linked into this process. The exception is a
    /// test the ledger declares a hang. `Atomics.wait` blocks the calling thread
    /// outright, so run directly it consumes a libtest worker forever and the
    /// whole suite spins at 587 of 588 — which is why the documented invocation
    /// carried a `--skip` for three batches and rung 1c was never a gate.
    /// Running that one test as a real child, and killing the child after
    /// [`HANG_TIMEOUT`], turns the hang into an ordinary bounded failure that
    /// libtest can report and `should_panic` can pin.
    ///
    /// **Both paths are bounded, and that is the point.** Routing by ledger row
    /// means the guarded path is only ever reached by a test the ledger already
    /// knows about; a *new* hang, in a test with no row, is by construction on
    /// the other path. If that path were unbounded — as it was — then under the
    /// documented `--test-threads=2` invocation every one of the 587 undeclared
    /// tests could still spin rung 1c forever, which is precisely the state the
    /// ledger exists to end, and `guarded_output`'s "this is a NEW hang" message
    /// could never actually be produced by the documented command. So the
    /// in-process call runs on a worker thread and is bounded by the same
    /// [`HANG_TIMEOUT`]. The blocked thread is leaked rather than killed —
    /// threads cannot be killed safely — but a leaked thread does not stop the
    /// test binary exiting, so termination becomes universal at the cost of one
    /// thread spawn per invocation.
    ///
    /// A panic inside the worker is resumed on this thread rather than being
    /// swallowed, so `#[should_panic]` and ordinary assertion failures behave
    /// exactly as they did when the call ran inline.
    fn output(&mut self) -> io::Result<CommandOutput> {
        let thread = std::thread::current();
        let thread_name = thread.name().map(str::to_owned);
        match known_failures::execution_path(thread_name.as_deref()) {
            known_failures::ExecutionPath::InProcess => {
                self.bounded_in_process_output(thread_name.as_deref().unwrap_or(UNNAMED_THREAD))
            }
            known_failures::ExecutionPath::GuardedSubprocess => {
                self.guarded_output(thread_name.as_deref().unwrap_or(UNNAMED_THREAD))
            }
        }
    }

    /// Run the CLI in process on a worker thread, bounded by [`HANG_TIMEOUT`].
    ///
    /// See [`Self::output`] for why this is not simply a direct call.
    fn bounded_in_process_output(&mut self, test_name: &str) -> io::Result<CommandOutput> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let args = self.args.clone();
        std::thread::Builder::new()
            .stack_size(IN_PROCESS_STACK_SIZE)
            .name(format!("porf-cli:{test_name}"))
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    porffor_cli::run_cli_capture(args)
                }));
                // A send failure means this test already gave up waiting; the
                // result is simply dropped.
                let _ = sender.send(result);
            })
            .expect("in-process CLI worker should spawn");

        match receiver.recv_timeout(HANG_TIMEOUT) {
            Ok(Ok(output)) => Ok(CommandOutput {
                status: CommandStatus {
                    success: output.exit_code == 0,
                },
                stdout: output.stdout,
                stderr: output.stderr,
            }),
            Ok(Err(panic)) => std::panic::resume_unwind(panic),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => panic!(
                "porf run exceeded {:?} in process: {} - the worker thread is leaked, not killed. \
                 If this test has no row in crates/porffor-cli/tests/known-failures.tsv it is a \
                 NEW hang (or a pathologically slow case; this bound cannot tell them apart): \
                 investigate, then add a row with an owner, or fix it.",
                HANG_TIMEOUT, test_name
            ),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!(
                "the in-process CLI worker for {test_name} disconnected without sending a result"
            ),
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
                     crates/porffor-cli/tests/known-failures.tsv it is a NEW hang (or a \
                     pathologically slow case; this bound cannot tell them apart, since the \
                     fixture prints nothing before it blocks): investigate, then add a row with \
                     an owner, or fix it. Recalibrate the timeout only against a measured cold \
                     compile on this box, never as a response to a red run.",
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
