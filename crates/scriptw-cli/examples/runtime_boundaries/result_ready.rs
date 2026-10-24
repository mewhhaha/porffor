//! Opt-in Linux checked-result signal. Opening /proc duplicates the inherited
//! pipe without taking ownership of an arbitrary raw descriptor.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Write;

fn configuration(fd: Option<&str>, nonce: Option<&str>) -> Result<Option<(u32, String)>, String> {
    match (fd, nonce) {
        (None, None) => Ok(None),
        (Some(fd), Some(nonce))
            if !fd.is_empty()
                && fd.bytes().all(|b| b.is_ascii_digit())
                && nonce.len() == 32
                && nonce
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
        {
            let fd: u32 = fd.parse().map_err(|_| "invalid result-ready descriptor")?;
            if fd < 3 {
                return Err("result-ready descriptor must be a dedicated inherited pipe".into());
            }
            Ok(Some((fd, nonce.into())))
        }
        _ => Err("result-ready pipe descriptor and 32-hex nonce must be supplied together".into()),
    }
}

pub(super) fn emit(checked_stdout: &str) -> Result<Option<Value>, String> {
    let fd = std::env::var("SCRIPTW_BOUNDARY_READY_FD").ok();
    let nonce = std::env::var("SCRIPTW_BOUNDARY_READY_NONCE").ok();
    let Some((fd, nonce)) = configuration(fd.as_deref(), nonce.as_deref())? else {
        return Ok(None);
    };
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::FileTypeExt;
        let mut pipe = std::fs::OpenOptions::new()
            .write(true)
            .open(format!("/proc/self/fd/{fd}"))
            .map_err(|error| format!("result-ready pipe: {error}"))?;
        if !pipe
            .metadata()
            .map_err(|error| error.to_string())?
            .file_type()
            .is_fifo()
        {
            return Err("result-ready descriptor is not a pipe".into());
        }
        let stdout_sha256 = format!("{:x}", Sha256::digest(checked_stdout.as_bytes()));
        let marker = format!("SWREADY1 {nonce} {stdout_sha256}\n");
        pipe.write_all(marker.as_bytes())
            .map_err(|error| format!("result-ready marker: {error}"))?;
        Ok(Some(
            json!({"nonce": nonce, "stdout_sha256": stdout_sha256}),
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (fd, nonce, checked_stdout);
        Err("result-ready pipes require Linux /proc".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_pipe_and_nonce_are_an_explicit_pair() {
        assert!(configuration(None, None).unwrap().is_none());
        assert!(
            configuration(Some("9"), Some("0123456789abcdef0123456789abcdef"))
                .unwrap()
                .is_some()
        );
        for (fd, nonce) in [
            (Some("1"), Some("0123456789abcdef0123456789abcdef")),
            (Some("9"), None),
            (None, Some("0123456789abcdef0123456789abcdef")),
            (Some("9"), Some("short")),
            (Some("-3"), Some("0123456789abcdef0123456789abcdef")),
            (Some("9"), Some("0123456789abcdef0123456789abcdeF")),
        ] {
            assert!(configuration(fd, nonce).is_err());
        }
    }
}
