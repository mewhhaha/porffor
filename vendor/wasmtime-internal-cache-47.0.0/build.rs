use std::path::Path;
use std::process::Command;
use std::str;

pub(crate) fn compiler_identity(source_dir: &Path, package_version: &str) -> (String, bool) {
    // Cargo adds this file when packaging a crate. A registry or vendored
    // package can live beneath an unrelated Git checkout; that checkout is
    // not the compiler's development source identity.
    if source_dir.join(".cargo_vcs_info.json").is_file() {
        return (package_version.to_string(), false);
    }

    match Command::new("git")
        .current_dir(source_dir)
        .args(["rev-parse", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => (
            str::from_utf8(&output.stdout).unwrap().trim().to_string(),
            true,
        ),
        _ => (package_version.to_string(), false),
    }
}

fn main() {
    let (compiler_version, use_mtime) = compiler_identity(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        env!("CARGO_PKG_VERSION"),
    );
    println!("cargo:rustc-env=COMPILER_VERSION={compiler_version}");
    println!("cargo:rustc-env=USE_MTIME={use_mtime}");
}
