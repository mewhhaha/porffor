//! A suite's explicitly owned output directory is separate from its inputs.

use std::path::{Component, Path, PathBuf};

use super::{
    git_capture, hash_directory_content_excluding, suite_git_context, LocalHarnessSource,
    SuiteConfig,
};

// Resolve existing symlink ancestors before normalizing a nonexistent suffix.
// This works before the supervisor has created its snapshot directory.
pub(super) fn normalize(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    let mut result = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::RootDir | Component::Prefix(_) => result.push(part.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(value) => {
                result.push(value);
                match result.canonicalize() {
                    Ok(canonical) => result = canonical,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(format!(
                            "cannot resolve suite output path {}: {error}",
                            result.display()
                        ))
                    }
                }
            }
        }
    }
    Ok(result)
}

pub(super) fn owned_output_root(config: &SuiteConfig) -> Result<PathBuf, String> {
    let root = normalize(&config.suite_root)?;
    let output = normalize(config.source_output_root())?;
    if root.starts_with(&output) {
        return Err("Test262 output directory must not contain the suite root".into());
    }
    for input in [root.join("test"), root.join("harness")] {
        let input = normalize(&input)?;
        if input.starts_with(&output) || output.starts_with(&input) {
            return Err(
                "Test262 output directory must be disjoint from test and harness inputs".into(),
            );
        }
    }
    if let LocalHarnessSource::File(harness) = &config.local_harness {
        if normalize(harness)?.starts_with(&output) {
            return Err(
                "Test262 output directory must not contain the selected harness file".into(),
            );
        }
    }
    check_destination(&root, &output, &config.snapshot_dir)?;
    let supplied =
        std::path::absolute(config.source_output_root()).map_err(|error| error.to_string())?;
    if output.starts_with(&root) || supplied.starts_with(&root) {
        let Some(context) = suite_git_context(&root) else {
            return Ok(output);
        };
        // A declared output role must never mask a tracked input, even outside
        // test/ and harness/. Literal pathspecs also cover names containing *[].
        // Check both the supplied path and its resolved namespace. A tracked
        // symlink must not turn into an untracked output exclusion.
        for path in [&output, &supplied] {
            if !path.starts_with(&root) {
                continue;
            }
            let Ok(relative) = path.strip_prefix(&context.toplevel) else {
                continue;
            };
            let spec = format!(":(top,literal){}", relative.to_string_lossy());
            let tracked = git_capture(&context.toplevel, &["ls-files", "--", &spec])
                .ok_or("cannot verify Test262 output directory has no tracked inputs")?;
            if !tracked.is_empty() {
                return Err("Test262 output directory must not contain tracked inputs".into());
            }
        }
    }
    Ok(output)
}

fn check_destination(root: &Path, output: &Path, destination: &Path) -> Result<(), String> {
    let destination = normalize(destination)?;
    if destination.starts_with(root) && !destination.starts_with(output) {
        return Err(
            "Test262 worker snapshots inside the suite must stay within its owned output directory"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn validate_case_destination(
    config: &SuiteConfig,
    destination: &Path,
) -> Result<(), String> {
    let root = normalize(&config.suite_root)?;
    let output = owned_output_root(config)?;
    check_destination(&root, &output, destination)
}

pub(super) fn compute_pin(root: &Path, output: &Path) -> String {
    let Some(context) = suite_git_context(root) else {
        return "missing-vendored-suite".into();
    };
    let mut args = vec!["status", "--porcelain", "--untracked-files=all", "--"];
    let input = format!(
        ":(top,literal){}",
        if context.prefix.is_empty() {
            "."
        } else {
            &context.prefix
        }
    );
    args.push(&input);
    let exclusion = output
        .strip_prefix(&context.toplevel)
        .ok()
        .map(|path| format!(":(top,exclude,literal){}", path.to_string_lossy()));
    if let Some(exclusion) = &exclusion {
        args.push(exclusion);
    }
    let clean = git_capture(&context.toplevel, &args).is_some_and(|status| status.is_empty());
    if clean {
        let spec = if context.prefix.is_empty() {
            "HEAD".into()
        } else {
            format!("HEAD:{}", context.prefix)
        };
        if let Some(pin) = git_capture(&context.toplevel, &["rev-parse", "--verify", &spec]) {
            return pin;
        }
    }
    format!(
        "worktree-{:016x}",
        hash_directory_content_excluding(root, Some(output))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    struct Fixture {
        root: PathBuf,
        config: SuiteConfig,
    }
    impl Fixture {
        fn new(label: &str) -> Self {
            let root = crate::tests::unique_temp_path(label);
            fs::create_dir_all(root.join("suite/test/language")).unwrap();
            fs::create_dir_all(root.join("suite/harness")).unwrap();
            assert!(Command::new("git")
                .args(["init", "-q"])
                .arg(&root)
                .status()
                .unwrap()
                .success());
            fs::write(root.join(".gitignore"), "suite/\n").unwrap();
            fs::write(root.join("suite/test/language/case.js"), "true;\n").unwrap();
            fs::write(root.join("suite/harness/assert.js"), "assert;\n").unwrap();
            fs::write(root.join("suite/metadata.txt"), "metadata\n").unwrap();
            let mut config = SuiteConfig::default();
            config.suite_root = root.join("suite");
            config.snapshot_dir = config.suite_root.join("snapshots[*]");
            config.local_harness = LocalHarnessSource::None;
            Self { root, config }
        }
        fn pin(&self) -> String {
            compute_pin(
                &self.config.suite_root,
                &owned_output_root(&self.config).unwrap(),
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn generated_outputs_preserve_suite_identity_but_all_other_inputs_change_it() {
        let fixture = Fixture::new("suite-output-source-closure");
        let initial = fixture.pin();
        let raw = crate::compute_suite_test262_pin(&fixture.config.suite_root);
        fs::create_dir_all(&fixture.config.snapshot_dir).unwrap();
        fs::write(
            fixture.config.snapshot_dir.join("attempt.journal"),
            "pending",
        )
        .unwrap();
        fs::write(fixture.config.snapshot_dir.join("result.json"), "{}").unwrap();
        assert_eq!(fixture.pin(), initial);
        assert_ne!(
            crate::compute_suite_test262_pin(&fixture.config.suite_root),
            raw
        );
        // A fresh worker's result destination is separate from the inherited
        // supervisor namespace, including when that namespace already exists.
        let mut child = fixture.config.clone();
        child.snapshot_dir = fixture.root.join("child-results");
        child
            .inherit_case_output_root(fixture.config.snapshot_dir.clone())
            .unwrap();
        assert_eq!(
            compute_pin(&child.suite_root, &owned_output_root(&child).unwrap()),
            initial
        );
        for bad in ["test/new-child", "harness/new-child", "sibling-child"] {
            let destination = fixture.config.suite_root.join(bad);
            assert!(validate_case_destination(&fixture.config, &destination).is_err());
            assert!(!destination.exists());
            child.snapshot_dir = destination;
            assert!(owned_output_root(&child).is_err());
        }
        let safe = fixture.config.snapshot_dir.join("child-results");
        assert!(validate_case_destination(&fixture.config, &safe).is_ok());
        for file in ["test/language/case.js", "harness/assert.js", "metadata.txt"] {
            let path = fixture.config.suite_root.join(file);
            let bytes = fs::read(&path).unwrap();
            fs::write(&path, [bytes.as_slice(), b"changed"].concat()).unwrap();
            assert_ne!(
                fixture.pin(),
                initial,
                "input mutation must change pin: {file}"
            );
            fs::write(path, bytes).unwrap();
            assert_eq!(fixture.pin(), initial);
        }
    }

    #[test]
    fn output_namespace_cannot_hide_source_inputs_or_tracked_files() {
        let fixture = Fixture::new("suite-output-admission");
        for path in [
            ".",
            "..",
            "test",
            "test/language",
            "harness",
            "harness/../test/new",
        ] {
            let mut config = fixture.config.clone();
            config.snapshot_dir = config.suite_root.join(path);
            assert!(
                owned_output_root(&config).is_err(),
                "invalid output: {path}"
            );
        }
        let mut config = fixture.config.clone();
        config.local_harness = LocalHarnessSource::File(config.snapshot_dir.join("custom.js"));
        assert!(owned_output_root(&config).is_err());
        // Revalidate after a public field changes; a checked inherited path
        // never authorizes hiding a newly selected harness.
        config = fixture.config.clone();
        config
            .inherit_case_output_root(config.snapshot_dir.clone())
            .unwrap();
        config.local_harness = LocalHarnessSource::File(config.snapshot_dir.join("custom.js"));
        assert!(crate::discover_suite(&config, None).is_err());
        fs::create_dir_all(&fixture.config.snapshot_dir).unwrap();
        fs::write(fixture.config.snapshot_dir.join("tracked.txt"), "source").unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&fixture.root)
            .args(["add", "-f", "--"])
            .arg(fixture.config.snapshot_dir.join("tracked.txt"))
            .status()
            .unwrap()
            .success());
        assert!(owned_output_root(&fixture.config).is_err());
        // Existing tracked snapshots outside the suite are not source inputs
        // and are not excluded from the source hash in the first place.
        config = fixture.config.clone();
        config.snapshot_dir = fixture.root.join("external-snapshots");
        fs::create_dir_all(&config.snapshot_dir).unwrap();
        fs::write(config.snapshot_dir.join("tracked.json"), "{}").unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&fixture.root)
            .args(["add", "--", "external-snapshots/tracked.json"])
            .status()
            .unwrap()
            .success());
        assert!(owned_output_root(&config).is_ok());
        #[cfg(unix)]
        {
            let alias = fixture.config.suite_root.join("output-alias");
            std::os::unix::fs::symlink(fixture.config.suite_root.join("test"), &alias).unwrap();
            config = fixture.config.clone();
            config.snapshot_dir = alias.join("new");
            assert!(owned_output_root(&config).is_err());
        }
    }

    #[test]
    fn cached_suite_identity_is_scoped_to_normalized_output_namespace() {
        let fixture = Fixture::new("suite-output-cache-key");
        let mut other = fixture.config.clone();
        other.snapshot_dir = other.suite_root.join("other-results");
        for (path, value) in [
            (&fixture.config.snapshot_dir, "first"),
            (&other.snapshot_dir, "second"),
        ] {
            fs::create_dir_all(path).unwrap();
            fs::write(path.join("result.json"), value).unwrap();
        }
        let first = crate::pinned_revisions(&fixture.config);
        let second = crate::pinned_revisions(&other);
        assert_ne!(first, second);
        assert_eq!(first.test262, fixture.pin());
        assert_eq!(
            second.test262,
            compute_pin(&other.suite_root, &owned_output_root(&other).unwrap())
        );
        let mut alias = fixture.config.clone();
        alias.snapshot_dir = alias.suite_root.join("unused/../snapshots[*]");
        assert_eq!(crate::pinned_revisions(&alias), first);
    }
}
