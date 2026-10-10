//! The CLI exports actual selected data; inspect reuses the same SDK admission.
use lila_engine::{IntlCompilationProfile, IntlDataSelection, SelectedIntlDataBundle};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub(super) fn command(args: Vec<String>, profile: Option<IntlCompilationProfile>) -> Result<(), String> {
    let mut args = args.into_iter();
    let operation = args.next().ok_or_else(|| "intl needs export or inspect".to_owned())?;
    let flag = match operation.as_str() {
        "export" => "--output", "inspect" => "--input",
        _ => return Err(format!("unknown intl command {operation}; expected export or inspect")),
    };
    if operation == "inspect" && profile.is_some() {
        return Err("intl inspect admits the input bundle's identity; it does not accept profile or locale selection flags".into());
    }
    let mut path = None;
    while let Some(argument) = args.next() {
        if argument != flag { return Err(format!("unknown intl {operation} argument {argument}; expected {flag} PATH")); }
        if path.is_some() { return Err(format!("{flag} may only be specified once")); }
        path = Some(PathBuf::from(args.next().ok_or_else(|| format!("{flag} needs a path"))?));
    }
    let path = path.ok_or_else(|| format!("intl {operation} needs {flag} PATH"))?;
    if operation == "export" {
        let selection = IntlDataSelection::new(profile.unwrap_or_default());
        let selected = selection.selected().map_err(|error| format!("Intl export selection failed: {error}"))?;
        let bytes = selected.export_bytes().map_err(|error| error.to_string())?;
        publish(&path, &bytes)?;
        println!("Intl bundle exported: {} ({} bytes)", path.display(), bytes.len());
    } else {
        let input = fs::File::open(&path).map_err(|error| format!("cannot read Intl bundle {}: {error}", path.display()))?;
        let mut bytes = Vec::new();
        input.take(SelectedIntlDataBundle::MAX_EXPORT_BYTES as u64 + 1).read_to_end(&mut bytes)
            .map_err(|error| format!("cannot read Intl bundle {}: {error}", path.display()))?;
        let admitted = SelectedIntlDataBundle::from_export_bytes(&bytes).map_err(|error| format!("cannot admit Intl bundle {}: {error}", path.display()))?;
        let identity = admitted.identity().artifact_identity();
        let identity = std::str::from_utf8(identity.as_bytes()).map_err(|error| format!("invalid admitted Intl identity UTF-8: {error}"))?;
        print!("{identity}");
    }
    Ok(())
}

/// Publish a fully synced sibling file without replacing an existing output.
/// Both names share a filesystem; hard-link publication is atomic and fails
/// clearly if the destination already exists or the filesystem cannot support it.
fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let name = path.file_name().ok_or_else(|| "Intl export output needs a file name".to_owned())?;
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary_name = std::ffi::OsString::from(".");
    temporary_name.push(name); temporary_name.push(".lila-intl-export.tmp");
    let temporary = parent.join(temporary_name);
    let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)
        .map_err(|error| format!("cannot create Intl export transaction {}: {error}", temporary.display()))?;
    let result: Result<(), String> = (|| {
        output.write_all(bytes).map_err(|error| format!("cannot write Intl export {}: {error}", temporary.display()))?;
        output.sync_all().map_err(|error| format!("cannot sync Intl export {}: {error}", temporary.display()))?;
        drop(output);
        fs::hard_link(&temporary, path).map_err(|error| format!("cannot publish Intl export {} without replacing it: {error}", path.display()))?;
        Ok(())
    })();
    let cleanup = fs::remove_file(&temporary);
    result?;
    cleanup.map_err(|error| format!("Intl export was published but transaction cleanup failed: {error}"))?;
    #[cfg(unix)]
    fs::File::open(parent).and_then(|directory| directory.sync_all())
        .map_err(|error| format!("Intl export was published but directory sync failed: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_and_inspect_reject_foreign_cli_arguments_before_data_or_output() {
        for args in [vec!["export"], vec!["export", "--input", "x"], vec!["inspect", "--output", "x"],
            vec!["export", "--output", "x", "--output", "y"], vec!["export", "--output"]] {
            assert!(command(args.into_iter().map(str::to_owned).collect(), None).is_err());
        }
        assert!(command(vec!["inspect".into(), "--input".into(), "missing".into()], Some(IntlCompilationProfile::Minimal)).unwrap_err().contains("does not accept"));
    }
}
