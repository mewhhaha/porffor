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

struct Command {
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
    fn new(_program: impl AsRef<OsStr>) -> Self {
        Self { args: Vec::new() }
    }

    fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args.push(arg.as_ref().to_string_lossy().into_owned());
        self
    }

    fn output(&mut self) -> io::Result<CommandOutput> {
        let output = porffor_cli::run_cli_capture(self.args.clone());
        Ok(CommandOutput {
            status: CommandStatus {
                success: output.exit_code == 0,
            },
            stdout: output.stdout,
            stderr: output.stderr,
        })
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
