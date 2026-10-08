//! Invocation-owned Linux RSS observations. This is a sampled process maximum,
//! including native caches and the sampler itself, rather than a Store counter.
use std::io::Read;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(super) const SAMPLE_INTERVAL: Duration = Duration::from_millis(10);
const MAX_STATUS_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmProcessMemoryScope {
    /// Artifact admission through the final endpoint read after completion
    /// decoding and root release. Other threads and retained caches contribute.
    RuntimeInvocationProcessRss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmProcessMemoryUnavailable {
    UnsupportedPlatform,
    ProcStatusReadFailed,
    ProcStatusMalformed,
    SamplingThreadStartFailed,
    SamplingThreadPanicked,
    SampleCountOverflow,
}

/// Only the invocation sampler constructs successful observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmProcessMemorySample {
    initial_rss_bytes: u64,
    final_rss_bytes: u64,
    maximum_rss_bytes: u64,
    sample_count: u64,
    elapsed: Duration,
}
impl WasmProcessMemorySample {
    pub const SAMPLE_INTERVAL: Duration = self::SAMPLE_INTERVAL;
    pub const fn scope(&self) -> WasmProcessMemoryScope {
        WasmProcessMemoryScope::RuntimeInvocationProcessRss
    }
    pub const fn initial_rss_bytes(&self) -> u64 {
        self.initial_rss_bytes
    }
    pub const fn final_rss_bytes(&self) -> u64 {
        self.final_rss_bytes
    }
    pub const fn maximum_rss_bytes(&self) -> u64 {
        self.maximum_rss_bytes
    }
    pub const fn sample_count(&self) -> u64 {
        self.sample_count
    }
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmProcessMemoryObservation {
    Sampled(WasmProcessMemorySample),
    Unavailable(WasmProcessMemoryUnavailable),
}

enum SamplerState {
    Running {
        stop: Sender<()>,
        thread: JoinHandle<Result<WasmProcessMemorySample, WasmProcessMemoryUnavailable>>,
    },
    Unavailable(WasmProcessMemoryUnavailable),
    Finished,
}

/// Every exit, including a product error or unwind, retires the one sampling
/// thread. The channel wakes it immediately rather than waiting one interval.
pub(crate) struct ProcessMemorySampler {
    state: SamplerState,
}
impl ProcessMemorySampler {
    pub(crate) fn start() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self::start_with_reader(read_process_rss)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self {
                state: SamplerState::Unavailable(WasmProcessMemoryUnavailable::UnsupportedPlatform),
            }
        }
    }

    fn start_with_reader(
        mut read: impl FnMut() -> Result<u64, WasmProcessMemoryUnavailable> + Send + 'static,
    ) -> Self {
        let started = Instant::now();
        let initial = match read() {
            Ok(value) => value,
            Err(reason) => {
                return Self {
                    state: SamplerState::Unavailable(reason),
                }
            }
        };
        let (stop, stopped) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("lila-rss-profile".into())
            .stack_size(256 * 1024)
            .spawn(move || {
                let mut maximum = initial;
                let mut count = 1u64;
                loop {
                    let finished = match stopped.recv_timeout(SAMPLE_INTERVAL) {
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => true,
                        Err(RecvTimeoutError::Timeout) => false,
                    };
                    let current = read()?;
                    maximum = maximum.max(current);
                    count = count
                        .checked_add(1)
                        .ok_or(WasmProcessMemoryUnavailable::SampleCountOverflow)?;
                    if finished {
                        return Ok(WasmProcessMemorySample {
                            initial_rss_bytes: initial,
                            final_rss_bytes: current,
                            maximum_rss_bytes: maximum,
                            sample_count: count,
                            elapsed: started.elapsed(),
                        });
                    }
                }
            });
        Self {
            state: match thread {
                Ok(thread) => SamplerState::Running { stop, thread },
                Err(_) => SamplerState::Unavailable(
                    WasmProcessMemoryUnavailable::SamplingThreadStartFailed,
                ),
            },
        }
    }

    pub(crate) fn finish(mut self) -> WasmProcessMemoryObservation {
        self.retire()
    }

    fn retire(&mut self) -> WasmProcessMemoryObservation {
        match std::mem::replace(&mut self.state, SamplerState::Finished) {
            SamplerState::Running { stop, thread } => {
                // A read failure may already have ended the thread.
                let _ = stop.send(());
                match thread.join() {
                    Ok(Ok(sample)) => WasmProcessMemoryObservation::Sampled(sample),
                    Ok(Err(reason)) => WasmProcessMemoryObservation::Unavailable(reason),
                    Err(_) => WasmProcessMemoryObservation::Unavailable(
                        WasmProcessMemoryUnavailable::SamplingThreadPanicked,
                    ),
                }
            }
            SamplerState::Unavailable(reason) => WasmProcessMemoryObservation::Unavailable(reason),
            SamplerState::Finished => unreachable!("the private sampler retires once"),
        }
    }
}
impl Drop for ProcessMemorySampler {
    fn drop(&mut self) {
        if !matches!(self.state, SamplerState::Finished) {
            let _ = self.retire();
        }
    }
}

#[cfg(target_os = "linux")]
fn read_process_rss() -> Result<u64, WasmProcessMemoryUnavailable> {
    let file = std::fs::File::open("/proc/self/status")
        .map_err(|_| WasmProcessMemoryUnavailable::ProcStatusReadFailed)?;
    let mut bytes = Vec::with_capacity(4096);
    file.take(MAX_STATUS_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| WasmProcessMemoryUnavailable::ProcStatusReadFailed)?;
    if bytes.len() as u64 > MAX_STATUS_BYTES {
        return Err(WasmProcessMemoryUnavailable::ProcStatusMalformed);
    }
    parse_process_rss(&bytes)
}

fn parse_process_rss(bytes: &[u8]) -> Result<u64, WasmProcessMemoryUnavailable> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| WasmProcessMemoryUnavailable::ProcStatusMalformed)?;
    let mut rss = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let mut fields = value.split_ascii_whitespace();
            let digits = fields
                .next()
                .ok_or(WasmProcessMemoryUnavailable::ProcStatusMalformed)?;
            if digits.is_empty()
                || !digits.bytes().all(|byte| byte.is_ascii_digit())
                || fields.next() != Some("kB")
                || fields.next().is_some()
                || rss.is_some()
            {
                return Err(WasmProcessMemoryUnavailable::ProcStatusMalformed);
            }
            rss = Some(
                digits
                    .parse::<u64>()
                    .ok()
                    .and_then(|value| value.checked_mul(1024))
                    .ok_or(WasmProcessMemoryUnavailable::ProcStatusMalformed)?,
            );
        }
    }
    rss.ok_or(WasmProcessMemoryUnavailable::ProcStatusMalformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[test]
    fn rss_parser_uses_current_residency_and_rejects_missing_or_damaged_units() {
        assert_eq!(
            parse_process_rss(b"VmHWM:\t999999 kB\nVmRSS:\t12 kB\n").unwrap(),
            12 * 1024
        );
        for bytes in [
            b"VmHWM: 12 kB\n".as_slice(),
            b"VmRSS: 12 bytes\n",
            b"VmRSS: 12 kB\nVmRSS: 13 kB\n",
            b"VmRSS: -1 kB\n",
            b"VmRSS: 18446744073709551615 kB\n",
            b"VmRSS: 1 kB extra\n",
        ] {
            assert_eq!(
                parse_process_rss(bytes),
                Err(WasmProcessMemoryUnavailable::ProcStatusMalformed)
            );
        }
    }

    #[test]
    fn invocation_endpoints_are_samples_and_read_failure_never_becomes_a_zero_peak() {
        let mut reads = 0;
        let observed = ProcessMemorySampler::start_with_reader(move || {
            reads += 1;
            Ok(if reads == 1 { 11 } else { 45 })
        })
        .finish();
        let WasmProcessMemoryObservation::Sampled(sample) = observed else {
            panic!("sampler unavailable")
        };
        assert_eq!(sample.initial_rss_bytes(), 11);
        assert_eq!(sample.final_rss_bytes(), 45);
        assert_eq!(sample.maximum_rss_bytes(), 45);
        assert!(sample.sample_count() >= 2);
        let mut first = true;
        assert_eq!(
            ProcessMemorySampler::start_with_reader(move || {
                if std::mem::take(&mut first) {
                    Ok(10)
                } else {
                    Err(WasmProcessMemoryUnavailable::ProcStatusReadFailed)
                }
            })
            .finish(),
            WasmProcessMemoryObservation::Unavailable(
                WasmProcessMemoryUnavailable::ProcStatusReadFailed
            )
        );
    }

    #[test]
    fn dropping_an_invocation_joins_its_sampler_before_returning() {
        struct ReaderLifetime(Arc<AtomicBool>);
        impl Drop for ReaderLifetime {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let retired = Arc::new(AtomicBool::new(false));
        let reader = ReaderLifetime(Arc::clone(&retired));
        let sampler = ProcessMemorySampler::start_with_reader(move || {
            let _ = &reader;
            Ok(10)
        });
        drop(sampler);
        assert!(retired.load(Ordering::SeqCst));
    }
}
