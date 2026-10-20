//! Explicit case workers share the inherited affinity and whole-CPU quota.
use super::*;

#[derive(Debug, Serialize)]
pub(super) struct ClosureScheduling {
    pub(super) configured_workers: NonZeroUsize,
    admitted_worker_ceiling: NonZeroUsize,
    compilation_jobs: usize,
}

impl ClosureScheduling {
    fn admit(workers: NonZeroUsize, capacity: NonZeroUsize, jobs: usize) -> Result<Self, String> {
        if workers > capacity {
            return Err(format!("closure requested {workers} workers but the visible affinity/CPU quota permits {capacity}"));
        }
        if workers.get() > 1 && jobs != 1 {
            return Err(
                "parallel closure requires --jobs 1; --threads only schedules cases".into(),
            );
        }
        Ok(Self {
            configured_workers: workers,
            admitted_worker_ceiling: capacity,
            compilation_jobs: jobs,
        })
    }

    pub(super) fn current(workers: NonZeroUsize) -> Result<Self, String> {
        // Preserve the serial entry's existing one-worker policy even when a
        // local memory-only scope does not delegate the CPU controller.
        let capacity = if workers == NonZeroUsize::MIN {
            NonZeroUsize::MIN
        } else {
            visible_capacity()?
        };
        Self::admit(workers, capacity, compilation_jobs())
    }

    pub(super) fn require_current_capacity(&self) -> Result<(), String> {
        Self::current(self.configured_workers).map(|_| ())
    }
}

// Match scripts/limited_verification.py: whole CPUs, minimum one, and the
// tightest visible ancestor quota. available_parallelism also bounds affinity.
fn quota_capacity(current: NonZeroUsize, raw: &str) -> Result<NonZeroUsize, String> {
    let fields = raw.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err("invalid closure cpu.max".into());
    }
    let decimal = |value: &str| -> Result<usize, String> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("invalid closure cpu.max integer".into());
        }
        value
            .parse::<NonZeroUsize>()
            .map(NonZeroUsize::get)
            .map_err(|_| "invalid closure cpu.max positive integer".into())
    };
    let period = decimal(fields[1])?;
    if fields[0] == "max" {
        return Ok(current);
    }
    let quota = decimal(fields[0])?;
    Ok(current.min(NonZeroUsize::new((quota / period).max(1)).unwrap()))
}

fn visible_capacity() -> Result<NonZeroUsize, String> {
    let mut capacity = thread::available_parallelism()
        .map_err(|error| format!("cannot determine closure CPU affinity: {error}"))?;
    #[cfg(target_os = "linux")]
    {
        let membership = fs::read_to_string("/proc/self/cgroup")
            .map_err(|error| format!("cannot read closure cgroup membership: {error}"))?;
        let unified = membership
            .lines()
            .filter_map(|line| line.strip_prefix("0::"))
            .collect::<Vec<_>>();
        if unified.len() != 1
            || !unified[0].starts_with('/')
            || Path::new(unified[0])
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err("closure requires one safe unified cgroup-v2 membership".into());
        }
        let root = Path::new("/sys/fs/cgroup");
        let mut group = root.join(unified[0].trim_start_matches('/'));
        loop {
            let raw = fs::read_to_string(group.join("cpu.max"))
                .map_err(|error| format!("cannot read closure CPU quota: {error}"))?;
            capacity = quota_capacity(capacity, &raw)?;
            if group == root {
                break;
            }
            if !group.pop() || !group.starts_with(root) {
                return Err("closure CPU quota ancestry escaped its cgroup root".into());
            }
        }
    }
    Ok(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(value: usize) -> NonZeroUsize {
        NonZeroUsize::new(value).unwrap()
    }

    #[test]
    fn explicit_workers_obey_affinity_and_tightest_whole_cpu_quota() {
        let mut capacity = count(8);
        for raw in ["max 100000", "450000 100000", "200000 100000"] {
            capacity = quota_capacity(capacity, raw).unwrap();
        }
        assert_eq!(capacity.get(), 2);
        assert_eq!(quota_capacity(count(8), "150000 100000").unwrap().get(), 1);
        assert_eq!(quota_capacity(count(8), "50000 100000").unwrap().get(), 1);
        assert_eq!(quota_capacity(count(1), "400000 100000").unwrap().get(), 1);
        assert!(ClosureScheduling::admit(count(4), count(4), 1).is_ok());
        assert!(ClosureScheduling::admit(count(5), count(4), 1).is_err());
        assert!(ClosureScheduling::admit(count(4), count(4), 2).is_err());
        assert!(ClosureScheduling::admit(count(1), count(4), 2).is_ok());
        assert!(NonZeroUsize::new(0).is_none());
    }

    #[test]
    fn malformed_quota_never_authorizes_parallel_workers() {
        for raw in [
            "",
            "max",
            "max 0",
            "0 100000",
            "1 -1",
            "+4 1",
            "4 1 extra",
            "999999999999999999999999999999 1",
        ] {
            assert!(quota_capacity(count(4), raw).is_err(), "{raw}");
        }
    }
}
