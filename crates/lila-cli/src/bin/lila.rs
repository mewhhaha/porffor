use std::process::ExitCode;

fn main() -> ExitCode {
    // Do not hold the process stderr lock while the engine runs: optional
    // Wasm trace events are emitted by a sized worker thread.
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    ExitCode::from(lila_cli::run_cli(
        std::env::args().skip(1),
        &mut stdout,
        &mut stderr,
    ))
}
