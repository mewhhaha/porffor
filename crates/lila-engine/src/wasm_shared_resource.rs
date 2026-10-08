//! Native shared byte ownership. JavaScript identity stays in Wasm GC.

use super::*;
use std::collections::BTreeMap;
use std::num::NonZeroI64;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::Condvar;
use std::time::{Duration, Instant};

pub(super) struct WasmSharedMemoryBacking {
    pub(super) memory: WasmtimeSharedMemory,
    allocations: Mutex<SharedByteAllocations>,
    async_waiters: Mutex<SharedWaiters>,
}

struct SharedByteAllocations {
    end: u64,
    free: BTreeMap<u64, u64>,
}

/// Only the backing allocator can construct a disjoint, bounded resource.
pub(super) struct WasmSharedBufferResource {
    backing: Arc<WasmSharedMemoryBacking>,
    base: u64,
    reserved: u64,
    maximum: u64,
    length: AtomicU64,
    growable: bool,
    growth: Mutex<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SharedWaiterId(NonZeroI64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SharedWaiterAddress(u64);

#[derive(Debug, Clone, Copy)]
enum SharedWaiterWidth {
    Int32,
    BigInt64,
}

impl SharedWaiterWidth {
    fn from_wire(width: i32) -> wasmtime::Result<Self> {
        match width {
            4 => Ok(Self::Int32),
            8 => Ok(Self::BigInt64),
            _ => Err(wasmtime::Error::msg(
                "shared waiter width must be four or eight",
            )),
        }
    }

    const fn bytes(self) -> u64 {
        match self {
            Self::Int32 => 4,
            Self::BigInt64 => 8,
        }
    }
}

/// Only registration publishes a native waiter identity; zero is an immediate
/// comparison failure and can never be retained as an active waiter.
enum AsyncWaitRegistrationKind {
    NotEqual,
    Registered(SharedWaiterId),
}
pub(super) struct AsyncWaitRegistration(AsyncWaitRegistrationKind);
impl AsyncWaitRegistration {
    pub(super) const fn wire(self) -> i64 {
        match self.0 {
            AsyncWaitRegistrationKind::NotEqual => 0,
            AsyncWaitRegistrationKind::Registered(id) => id.0.get(),
        }
    }
}

struct SharedWaiter {
    id: SharedWaiterId,
    address: SharedWaiterAddress,
    // A pending wait retains its native resource until settlement/cancellation.
    _resource: Arc<WasmSharedBufferResource>,
    notified: bool,
    notification: WaitNotification,
}

enum WaitNotification {
    Async,
    Sync(Arc<SyncWaitSignal>),
}
#[derive(Clone, Copy)]
enum SyncWaitState {
    Pending,
    Notified,
    Cancelled,
}
struct SyncWaitSignal {
    state: Mutex<SyncWaitState>,
    changed: Condvar,
}

/// A result from the common critical-section owner, never a raw Wasm wait code.
pub(super) enum NativeSyncWaitResult {
    Notified,
    NotEqual,
    TimedOut,
}
impl NativeSyncWaitResult {
    pub(super) const fn wire(self) -> i32 {
        match self {
            Self::Notified => 0,
            Self::NotEqual => 1,
            Self::TimedOut => 2,
        }
    }
}

struct SharedWaiters {
    next_id: Option<NonZeroI64>,
    waiters: VecDeque<SharedWaiter>,
}

/// Last Store owner cancels every remaining wait, including trap/timeout exits.
/// This breaks the backing → waiter → resource → backing retention cycle.
pub(super) struct WasmStoreAsyncWaiters {
    backing: Option<Arc<WasmSharedMemoryBacking>>,
    ids: Mutex<Vec<SharedWaiterId>>,
}

impl WasmSharedMemoryBacking {
    pub(super) fn new(memory: WasmtimeSharedMemory) -> Arc<Self> {
        Arc::new(Self {
            memory,
            allocations: Mutex::new(SharedByteAllocations {
                end: 8,
                free: BTreeMap::new(),
            }),
            async_waiters: Mutex::new(SharedWaiters {
                next_id: NonZeroI64::new(1),
                waiters: VecDeque::new(),
            }),
        })
    }

    pub(super) fn allocate(
        self: &Arc<Self>,
        length: u64,
        maximum: u64,
        growable: bool,
    ) -> wasmtime::Result<Option<Arc<WasmSharedBufferResource>>> {
        if length > maximum || (!growable && length != maximum) {
            return Err(wasmtime::Error::msg(
                "invalid compiled shared-buffer size contract",
            ));
        }
        let Some(reserved) = maximum.max(1).checked_add(7).map(|size| size & !7) else {
            return Ok(None);
        };
        let mut allocations = self
            .allocations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let reusable = allocations
            .free
            .iter()
            .find_map(|(&base, &size)| (size >= reserved).then_some((base, size)));
        let base = reusable.map_or(allocations.end, |(base, _)| base);
        let Some(end) = base.checked_add(reserved) else {
            return Ok(None);
        };
        if end > WASM_STORE_MEMORY_CAP_BYTES as u64 {
            return Ok(None);
        }
        let required_pages = end.div_ceil(64 * 1024);
        let current_pages = self.memory.size();
        if required_pages > current_pages
            && self.memory.grow(required_pages - current_pages).is_err()
        {
            return Ok(None);
        }
        match reusable {
            Some((base, size)) => {
                allocations.free.remove(&base);
                if size > reserved {
                    allocations.free.insert(end, size - reserved);
                }
            }
            None => allocations.end = end,
        }
        drop(allocations);
        Ok(Some(Arc::new(WasmSharedBufferResource {
            backing: Arc::clone(self),
            base,
            reserved,
            maximum,
            length: AtomicU64::new(length),
            growable,
            growth: Mutex::new(()),
        })))
    }

    pub(super) fn check_resource(
        &self,
        resource: &WasmSharedBufferResource,
    ) -> wasmtime::Result<()> {
        if !std::ptr::eq(self, Arc::as_ptr(&resource.backing)) {
            return Err(wasmtime::Error::msg(
                "shared byte resource belongs to another backing",
            ));
        }
        Ok(())
    }

    pub(super) fn notify_async_waiters(
        &self,
        resource: &WasmSharedBufferResource,
        relative_offset: i64,
        width: i32,
        count: i64,
    ) -> wasmtime::Result<i64> {
        self.check_resource(resource)?;
        let address =
            resource.waiter_address(relative_offset, SharedWaiterWidth::from_wire(width)?)?;
        if count < 0 {
            return Err(wasmtime::Error::msg("compiled notify count is negative"));
        }
        let mut registry = self
            .async_waiters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut notified = 0;
        for waiter in &mut registry.waiters {
            if notified >= count {
                break;
            }
            if waiter.address != address || waiter.notified {
                continue;
            }
            waiter.notified = true;
            if let WaitNotification::Sync(signal) = &waiter.notification {
                *signal
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = SyncWaitState::Notified;
                signal.changed.notify_one();
            }
            notified += 1;
        }
        Ok(notified)
    }

    fn poll_async_waiter(&self, id: SharedWaiterId) -> i64 {
        let mut registry = self
            .async_waiters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(position) = registry.waiters.iter().position(|waiter| waiter.id == id) else {
            return -1;
        };
        if !registry.waiters[position].notified {
            return 0;
        }
        registry.waiters.remove(position);
        1
    }

    fn cancel_async_waiter(&self, id: SharedWaiterId) -> i64 {
        let mut registry = self
            .async_waiters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(position) = registry.waiters.iter().position(|waiter| waiter.id == id) else {
            return -1;
        };
        let notified = registry.waiters[position].notified;
        registry.waiters.remove(position);
        i64::from(notified)
    }

    pub(super) fn cancel_all_waiters(&self) {
        let mut registry = self
            .async_waiters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for waiter in &registry.waiters {
            if let WaitNotification::Sync(signal) = &waiter.notification {
                *signal
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = SyncWaitState::Cancelled;
                signal.changed.notify_one();
            }
        }
        registry.waiters.clear();
    }

    /// Comparison runs while the same registry lock excludes notify/removal.
    fn wait_word_equals(
        &self,
        address: SharedWaiterAddress,
        width: SharedWaiterWidth,
        expected: i64,
    ) -> bool {
        let pointer = self.memory.data()[address.0 as usize].get();
        match width {
            SharedWaiterWidth::Int32 => {
                unsafe { AtomicU32::from_ptr(pointer.cast()) }.load(Ordering::SeqCst)
                    == expected as u32
            }
            SharedWaiterWidth::BigInt64 => {
                unsafe { AtomicU64::from_ptr(pointer.cast()) }.load(Ordering::SeqCst)
                    == expected as u64
            }
        }
    }

    fn release(&self, base: u64, reserved: u64) {
        let mut allocations = self
            .allocations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.zero(base, reserved);
        let mut start = base;
        let mut size = reserved;
        if let Some((&previous, &previous_size)) = allocations.free.range(..base).next_back() {
            if previous + previous_size == base {
                allocations.free.remove(&previous);
                start = previous;
                size += previous_size;
            }
        }
        if let Some((&next, &next_size)) = allocations.free.range(start..).next() {
            if start + size == next {
                allocations.free.remove(&next);
                size += next_size;
            }
        }
        allocations.free.insert(start, size);
    }

    fn zero(&self, base: u64, length: u64) {
        // Private allocation/growth proofs guarantee this entire range exists.
        // Shared-memory host accesses use atomics even during exclusive reuse.
        for byte in &self.memory.data()[base as usize..(base + length) as usize] {
            unsafe { AtomicU8::from_ptr(byte.get()) }.store(0, Ordering::Relaxed);
        }
    }
}

impl WasmSharedBufferResource {
    pub(super) const fn base(&self) -> u64 {
        self.base
    }
    pub(super) const fn maximum(&self) -> u64 {
        self.maximum
    }
    pub(super) const fn growable(&self) -> bool {
        self.growable
    }
    pub(super) fn length(&self) -> u64 {
        self.length.load(Ordering::SeqCst)
    }

    pub(super) fn grow(&self, length: u64) -> bool {
        let _growth = self
            .growth
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let previous = self.length();
        if !self.growable || length < previous || length > self.maximum {
            return false;
        }
        self.backing.zero(self.base + previous, length - previous);
        // All agents observe the same active byte length and initialized bytes.
        self.length.store(length, Ordering::SeqCst);
        true
    }

    fn waiter_address(
        &self,
        relative: i64,
        width: SharedWaiterWidth,
    ) -> wasmtime::Result<SharedWaiterAddress> {
        let relative = u64::try_from(relative)
            .map_err(|_| wasmtime::Error::msg("shared waiter offset is negative"))?;
        let bytes = width.bytes();
        if relative % bytes != 0
            || relative
                .checked_add(bytes)
                .is_none_or(|end| end > self.length())
        {
            return Err(wasmtime::Error::msg(
                "shared waiter range is unaligned or out of bounds",
            ));
        }
        Ok(SharedWaiterAddress(self.base + relative))
    }
}

impl Drop for WasmSharedBufferResource {
    fn drop(&mut self) {
        self.backing.release(self.base, self.reserved);
    }
}

impl WasmStoreAsyncWaiters {
    pub(super) fn new(backing: Option<Arc<WasmSharedMemoryBacking>>) -> Arc<Self> {
        Arc::new(Self {
            backing,
            ids: Mutex::new(Vec::new()),
        })
    }

    pub(super) fn register(
        &self,
        resource: Arc<WasmSharedBufferResource>,
        relative_offset: i64,
        width: i32,
        expected_word: i64,
    ) -> wasmtime::Result<AsyncWaitRegistration> {
        let backing = self
            .backing
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("async wait requires shared memory"))?;
        backing.check_resource(&resource)?;
        let address =
            resource.waiter_address(relative_offset, SharedWaiterWidth::from_wire(width)?)?;
        let mut ids = self.ids.lock().unwrap_or_else(|error| error.into_inner());
        let mut registry = backing
            .async_waiters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // This is the same critical section used by notify. The shared typed
        // width/range proof precedes a no-tear load, so no notification can fall
        // between comparison and publication of the waiting record.
        let equal =
            backing.wait_word_equals(address, SharedWaiterWidth::from_wire(width)?, expected_word);
        if !equal {
            return Ok(AsyncWaitRegistration(AsyncWaitRegistrationKind::NotEqual));
        }
        let id = SharedWaiterId(
            registry
                .next_id
                .ok_or_else(|| wasmtime::Error::msg("shared waiter identity exhausted"))?,
        );
        registry.next_id = id.0.get().checked_add(1).and_then(NonZeroI64::new);
        registry.waiters.push_back(SharedWaiter {
            id,
            address,
            _resource: resource,
            notified: false,
            notification: WaitNotification::Async,
        });
        ids.push(id);
        Ok(AsyncWaitRegistration(
            AsyncWaitRegistrationKind::Registered(id),
        ))
    }

    pub(super) fn wait(
        &self,
        resource: Arc<WasmSharedBufferResource>,
        relative_offset: i64,
        width: i32,
        expected: i64,
        timeout_nanos: i64,
    ) -> wasmtime::Result<NativeSyncWaitResult> {
        let backing = self
            .backing
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("sync wait requires shared memory"))?;
        backing.check_resource(&resource)?;
        let width = SharedWaiterWidth::from_wire(width)?;
        let address = resource.waiter_address(relative_offset, width)?;
        let signal = Arc::new(SyncWaitSignal {
            state: Mutex::new(SyncWaitState::Pending),
            changed: Condvar::new(),
        });
        let id = {
            let mut registry = backing
                .async_waiters
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if !backing.wait_word_equals(address, width, expected) {
                return Ok(NativeSyncWaitResult::NotEqual);
            }
            if timeout_nanos == 0 {
                return Ok(NativeSyncWaitResult::TimedOut);
            }
            let id = SharedWaiterId(
                registry
                    .next_id
                    .ok_or_else(|| wasmtime::Error::msg("shared waiter identity exhausted"))?,
            );
            registry.next_id = id.0.get().checked_add(1).and_then(NonZeroI64::new);
            registry.waiters.push_back(SharedWaiter {
                id,
                address,
                _resource: resource,
                notified: false,
                notification: WaitNotification::Sync(Arc::clone(&signal)),
            });
            id
        };
        // Never hold the signal lock while acquiring the registry: notify and
        // cancellation use registry -> signal. Elapsed time avoids Instant overflow.
        let started = Instant::now();
        let timeout = (timeout_nanos >= 0).then(|| Duration::from_nanos(timeout_nanos as u64));
        let mut state = signal
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        loop {
            match *state {
                SyncWaitState::Notified => {
                    drop(state);
                    backing.cancel_async_waiter(id);
                    return Ok(NativeSyncWaitResult::Notified);
                }
                SyncWaitState::Cancelled => {
                    return Err(wasmtime::Error::msg(
                        "shared wait cancelled during agent retirement",
                    ));
                }
                SyncWaitState::Pending => {}
            }
            if let Some(timeout) = timeout {
                let Some(remaining) = timeout
                    .checked_sub(started.elapsed())
                    .filter(|remaining| !remaining.is_zero())
                else {
                    drop(state);
                    return match backing.cancel_async_waiter(id) {
                        1 => Ok(NativeSyncWaitResult::Notified),
                        0 => Ok(NativeSyncWaitResult::TimedOut),
                        _ => Err(wasmtime::Error::msg("shared wait retired during timeout")),
                    };
                };
                state = signal
                    .changed
                    .wait_timeout(state, remaining)
                    .unwrap_or_else(|error| error.into_inner())
                    .0;
            } else {
                state = signal
                    .changed
                    .wait(state)
                    .unwrap_or_else(|error| error.into_inner());
            }
        }
    }

    pub(super) fn poll(&self, wire: i64) -> i64 {
        self.settle(wire, false)
    }

    pub(super) fn cancel(&self, wire: i64) -> i64 {
        self.settle(wire, true)
    }

    fn settle(&self, wire: i64, cancel: bool) -> i64 {
        let Some(id) = NonZeroI64::new(wire)
            .filter(|id| id.get() > 0)
            .map(SharedWaiterId)
        else {
            return -1;
        };
        let Some(backing) = &self.backing else {
            return -1;
        };
        let mut ids = self.ids.lock().unwrap_or_else(|error| error.into_inner());
        let Some(position) = ids.iter().position(|owned| *owned == id) else {
            return -1;
        };
        let result = if cancel {
            backing.cancel_async_waiter(id)
        } else {
            backing.poll_async_waiter(id)
        };
        if result != 0 || cancel {
            ids.swap_remove(position);
        }
        result
    }
}

impl Drop for WasmStoreAsyncWaiters {
    fn drop(&mut self) {
        if let Some(backing) = &self.backing {
            for &id in self
                .ids
                .get_mut()
                .unwrap_or_else(|error| error.into_inner())
                .iter()
            {
                backing.cancel_async_waiter(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_backing() -> Arc<WasmSharedMemoryBacking> {
        let engine = shared_wasm_engine().expect("required shared-memory runtime");
        WasmSharedMemoryBacking::new(
            WasmtimeSharedMemory::new(&engine, wasmtime::MemoryType::shared(1, 4))
                .expect("bounded native shared byte backing"),
        )
    }

    #[test]
    fn released_ranges_are_cleared_coalesced_and_reused_without_overlapping_live_bytes() {
        let backing = new_backing();
        let first = backing.allocate(16, 16, false).unwrap().unwrap();
        let second = backing.allocate(24, 24, false).unwrap().unwrap();
        let live = backing.allocate(8, 8, false).unwrap().unwrap();
        let base = first.base();
        for byte in &backing.memory.data()[base as usize..live.base() as usize] {
            unsafe { AtomicU8::from_ptr(byte.get()) }.store(127, Ordering::Relaxed);
        }
        drop(second);
        drop(first);
        let replacement = backing.allocate(40, 40, false).unwrap().unwrap();
        assert_eq!(replacement.base(), base);
        assert_eq!(replacement.base() + replacement.maximum(), live.base());
        for byte in &backing.memory.data()[base as usize..live.base() as usize] {
            assert_eq!(
                unsafe { AtomicU8::from_ptr(byte.get()) }.load(Ordering::Relaxed),
                0
            );
        }
        assert!(backing.allocate(0, u64::MAX, true).unwrap().is_none());
        assert!(backing.allocate(9, 8, true).is_err());
        assert!(backing.allocate(4, 8, false).is_err());
        assert_eq!(live.length(), 8);
    }

    #[test]
    fn resource_growth_is_shared_and_exposes_zero_bytes_before_publishing_length() {
        let backing = new_backing();
        let resource = backing.allocate(8, 64, true).unwrap().unwrap();
        let agent_resource = Arc::clone(&resource);
        let byte = &backing.memory.data()[(resource.base() + 8) as usize];
        unsafe { AtomicU8::from_ptr(byte.get()) }.store(99, Ordering::Relaxed);
        assert!(agent_resource.grow(16));
        assert_eq!(resource.length(), 16);
        assert_eq!(
            unsafe { AtomicU8::from_ptr(byte.get()) }.load(Ordering::Relaxed),
            0
        );
        assert!(!resource.grow(15));
        assert!(!resource.grow(65));
        assert!(resource.grow(16));
        let fixed = backing.allocate(8, 8, false).unwrap().unwrap();
        assert!(!fixed.grow(8));
        let foreign = new_backing();
        assert!(foreign.notify_async_waiters(&resource, 0, 4, 1).is_err());
    }

    #[test]
    fn pending_waits_retain_resources_and_only_the_registering_store_can_settle_them() {
        let backing = new_backing();
        let resource = backing.allocate(8, 8, false).unwrap().unwrap();
        let weak = Arc::downgrade(&resource);
        let owner = WasmStoreAsyncWaiters::new(Some(Arc::clone(&backing)));
        let foreign_store = WasmStoreAsyncWaiters::new(Some(Arc::clone(&backing)));
        assert!(owner.register(Arc::clone(&resource), 4, 8, 0).is_err());
        assert!(owner.register(Arc::clone(&resource), 8, 4, 0).is_err());
        assert!(owner.register(Arc::clone(&resource), 0, 2, 0).is_err());
        let id = owner
            .register(Arc::clone(&resource), 0, 4, 0)
            .unwrap()
            .wire();
        assert_eq!(foreign_store.cancel(id), -1);
        assert_eq!(owner.poll(id), 0);
        drop(resource);
        let held = weak.upgrade().expect("pending wait retains resource");
        assert_eq!(backing.notify_async_waiters(&held, 0, 4, 1).unwrap(), 1);
        drop(held);
        assert_eq!(owner.poll(id), 1);
        assert!(weak.upgrade().is_none());
        assert_eq!(owner.poll(id), -1);
        let retained = backing.allocate(8, 8, false).unwrap().unwrap();
        let retained_weak = Arc::downgrade(&retained);
        owner.register(Arc::clone(&retained), 0, 8, 0).unwrap();
        drop(retained);
        drop(owner);
        assert!(
            retained_weak.upgrade().is_none(),
            "Store exit cancels retained waits"
        );
    }
}
