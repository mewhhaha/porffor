//! A whole-case deadline begins before spawning the actual compiler process.

use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

use lila_engine::ExecutionBackend;

#[derive(Clone, Copy)]
pub(super) struct CaseDeadline {
    started: Instant,
    expires: Instant,
}

impl CaseDeadline {
    pub(super) fn new(timeout_ms: u64, backend: ExecutionBackend) -> Result<Self, String> {
        let allowance = match backend {
            ExecutionBackend::WasmAot => super::WASM_AOT_CHILD_COMPILE_ALLOWANCE_MS,
            ExecutionBackend::SpecExec => 0,
        };
        let budget_ms = timeout_ms
            .checked_add(allowance)
            .ok_or("case compile/execution deadline overflows milliseconds")?;
        let started = Instant::now();
        let expires = started
            .checked_add(Duration::from_millis(budget_ms))
            .ok_or("case compile/execution deadline exceeds the monotonic clock domain")?;
        Ok(Self { started, expires })
    }

    pub(super) fn elapsed_ms(self) -> u128 {
        self.started.elapsed().as_millis()
    }
    pub(super) fn expired(self) -> bool {
        Instant::now() >= self.expires
    }
}

pub(super) struct CaseProcess {
    child: Child,
    #[cfg(unix)]
    group: i32,
    deadline: CaseDeadline,
    retired: bool,
}

impl CaseProcess {
    pub(super) fn spawn(command: &mut Command, deadline: CaseDeadline) -> Result<Self, String> {
        #[cfg(not(unix))]
        {
            let _ = (command, deadline);
            Err("bounded Test262 case supervision requires Unix process-group retirement on this host".into())
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
            if deadline.expired() {
                return Err("case deadline expired before spawning its worker".into());
            }
            let child = command.spawn().map_err(|error| error.to_string())?;
            // On supported Unix hosts pid_t is a positive signed integer. This
            // projection cannot fail for a live Child and adds no wait path.
            let group = i32::try_from(child.id()).expect("Unix child pid is a positive pid_t");
            Ok(Self {
                child,
                group,
                deadline,
                retired: false,
            })
        }
    }

    pub(super) fn wait_until_deadline(&mut self) -> Result<Option<ExitStatus>, String> {
        loop {
            // An exited child observed after the deadline cannot publish success.
            if self.deadline.expired() {
                return Ok(None);
            }
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| format!("failed to poll case worker: {error}"))?
            {
                return Ok((!self.deadline.expired()).then_some(status));
            }
            let remaining = self
                .deadline
                .expires
                .saturating_duration_since(Instant::now());
            std::thread::sleep(Duration::from_millis(10).min(remaining));
        }
    }

    pub(super) fn retire(&mut self) -> Result<(), String> {
        if self.retired {
            return Ok(());
        }
        let mut errors = Vec::new();
        #[cfg(unix)]
        {
            // Includes descendants even when the direct worker already exited.
            let status = unsafe { libc::kill(-self.group, libc::SIGKILL) };
            if status != 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    errors.push(format!("case process group termination failed: {error}"));
                }
            }
        }
        let cleanup_deadline = Instant::now() + Duration::from_secs(2);
        let mut signalled_direct = false;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if !signalled_direct {
                        if let Err(error) = self.child.kill() {
                            #[cfg(unix)]
                            if error.raw_os_error() != Some(libc::ESRCH) {
                                errors.push(format!(
                                    "case direct worker termination failed: {error}"
                                ));
                            }
                            #[cfg(not(unix))]
                            errors.push(format!("case direct worker termination failed: {error}"));
                        }
                        signalled_direct = true;
                    }
                    if Instant::now() >= cleanup_deadline {
                        errors
                            .push("case worker was not reaped within its cleanup deadline".into());
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => {
                    errors.push(format!("case worker reaping failed: {error}"));
                    break;
                }
            }
        }
        self.retired = errors.is_empty();
        if self.retired {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

impl Drop for CaseProcess {
    fn drop(&mut self) {
        let _ = self.retire();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_rejects_overflow_and_counts_time_before_spawn() {
        assert!(CaseDeadline::new(u64::MAX, ExecutionBackend::WasmAot).is_err());
        let deadline = CaseDeadline::new(1, ExecutionBackend::SpecExec).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        assert!(deadline.expired());
        #[cfg(unix)]
        assert!(CaseProcess::spawn(&mut Command::new("sh"), deadline).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn timeout_reaps_the_worker_and_terminates_its_descendants() {
        let root = std::env::temp_dir().join(format!(
            "lila-case-deadline-group-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let descendant = root.join("descendant");
        let script = root.join("worker.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nsleep 60 &\nprintf '%s' \"$!\" > \"$1\"\nwait\n",
        )
        .unwrap();
        let mut command = Command::new("sh");
        command.arg(&script).arg(&descendant);
        let deadline = CaseDeadline::new(500, ExecutionBackend::SpecExec).unwrap();
        let mut process = CaseProcess::spawn(&mut command, deadline).unwrap();
        assert!(process.wait_until_deadline().unwrap().is_none());
        process.retire().unwrap();
        assert!(process.child.try_wait().unwrap().is_some());
        let pid: i32 = std::fs::read_to_string(descendant)
            .unwrap()
            .parse()
            .unwrap();
        // kill(0) can still see a reparented zombie. Linux /proc additionally
        // proves it is no longer executing; the parent only reaps its own child.
        #[cfg(target_os = "linux")]
        {
            let retired_by = Instant::now() + Duration::from_secs(2);
            loop {
                match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                    Ok(stat)
                        if stat.rsplit_once(')').unwrap().1.split_whitespace().next()
                            == Some("Z") =>
                    {
                        break
                    }
                    result => {
                        assert!(
                            Instant::now() < retired_by,
                            "descendant remained live: {result:?}"
                        );
                        std::thread::sleep(Duration::from_millis(2));
                    }
                }
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
