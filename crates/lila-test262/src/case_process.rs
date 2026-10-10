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
            #[cfg(target_os = "linux")]
            {
                let supervisor = unsafe { libc::getpid() };
                // A worker owns its process group for case-deadline cleanup.
                // Also bind it to the spawning supervisor thread: killing the
                // publisher must not leave its isolated compiler running.
                unsafe {
                    command.pre_exec(move || {
                        if libc::prctl(
                            libc::PR_SET_PDEATHSIG,
                            libc::SIGKILL as libc::c_ulong,
                            0 as libc::c_ulong,
                            0 as libc::c_ulong,
                            0 as libc::c_ulong,
                        ) != 0
                        {
                            return Err(std::io::Error::last_os_error());
                        }
                        // The supervisor can die between fork and arming the
                        // signal. Reject that race before executing the worker.
                        if libc::getppid() != supervisor {
                            return Err(std::io::Error::from_raw_os_error(libc::ECHILD));
                        }
                        Ok(())
                    });
                }
            }
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

    #[test]
    #[cfg(target_os = "linux")]
    fn isolated_worker_terminates_when_its_supervisor_is_killed() {
        const HELPER_ROOT: &str = "LILA_CASE_PARENT_DEATH_TEST_ROOT";
        if let Some(root) = std::env::var_os(HELPER_ROOT) {
            // This branch runs in the disposable supervisor process. Its
            // worker has a separate group, so killing only the supervisor's
            // group cannot account for the worker's subsequent termination.
            let mut command = Command::new("sleep");
            command.arg("60");
            let deadline = CaseDeadline::new(60_000, ExecutionBackend::SpecExec).unwrap();
            let mut worker = CaseProcess::spawn(&mut command, deadline).unwrap();
            let root = std::path::Path::new(&root);
            std::fs::write(root.join("worker-pid.tmp"), worker.child.id().to_string()).unwrap();
            std::fs::rename(root.join("worker-pid.tmp"), root.join("worker-pid")).unwrap();
            let _ = worker.wait_until_deadline().unwrap();
            panic!("the disposable supervisor should have been killed");
        }

        let root = std::env::temp_dir().join(format!(
            "lila-case-parent-death-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "case_process::tests::isolated_worker_terminates_when_its_supervisor_is_killed",
                "--nocapture",
            ])
            .env(HELPER_ROOT, &root)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let deadline = CaseDeadline::new(5_000, ExecutionBackend::SpecExec).unwrap();
        let mut supervisor = CaseProcess::spawn(&mut command, deadline).unwrap();
        let pid_path = root.join("worker-pid");
        while !pid_path.exists() {
            assert!(!deadline.expired(), "supervisor did not start its worker");
            assert!(supervisor.child.try_wait().unwrap().is_none());
            std::thread::sleep(Duration::from_millis(2));
        }
        let pid: i32 = std::fs::read_to_string(pid_path).unwrap().parse().unwrap();
        assert_eq!(unsafe { libc::getpgid(pid) }, pid);
        assert_ne!(pid, supervisor.group);
        supervisor.retire().unwrap();

        let retired_by = Instant::now() + Duration::from_secs(2);
        let terminated = loop {
            match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break true,
                Ok(stat)
                    if stat.rsplit_once(')').unwrap().1.split_whitespace().next() == Some("Z") =>
                {
                    break true;
                }
                _ if Instant::now() >= retired_by => break false,
                _ => std::thread::sleep(Duration::from_millis(2)),
            }
        };
        // Keep a failing regression from leaving its own long-lived worker.
        if !terminated {
            unsafe { libc::kill(-pid, libc::SIGKILL) };
        }
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            terminated,
            "isolated worker survived supervisor termination"
        );
    }
}
