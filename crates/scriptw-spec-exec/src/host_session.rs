//! Thread-local execution, agent lifetime and embedded Realm ownership.
use super::*;

thread_local! {
    static HOST_REALMS: RefCell<HostRealmStore> = RefCell::new(HostRealmStore::default());
    static CURRENT_HOST_SESSION: RefCell<Option<Arc<HostSession>>> = const { RefCell::new(None) };
    static CURRENT_AGENT_STATE: RefCell<Option<AgentThreadState>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct HostRealmStore {
    next_id: u64,
    realms: BTreeMap<u64, HostRealmContext>,
}

/// Embedded Realms share their creating Context's interner and job domain.
/// Ambient policies retain their existing independent Context ownership.
#[derive(Clone)]
pub(super) enum HostRealmContext {
    SeparateContext(Rc<RefCell<Context>>),
    EmbeddedRealm(boa_engine::realm::Realm),
}

impl HostRealmContext {
    pub(super) fn with_context<T>(
        &self,
        context: &mut Context,
        action: impl FnOnce(&mut Context) -> JsResult<T>,
    ) -> JsResult<T> {
        match self {
            Self::SeparateContext(realm) => {
                let mut realm = realm.try_borrow_mut().map_err(|_| {
                    oracle_exception::host_failure(
                        JsNativeError::typ()
                            .with_message("host realm context is already active")
                            .into(),
                        context,
                    )
                })?;
                action(&mut realm)
            }
            Self::EmbeddedRealm(realm) => {
                let old_realm = context.enter_realm(realm.clone());
                // Catch only to restore the creator Realm; preserve the panic
                // payload and resume unwinding after restoration.
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(context)));
                context.enter_realm(old_realm);
                match result {
                    Ok(result) => result,
                    Err(payload) => std::panic::resume_unwind(payload),
                }
            }
        }
    }

    pub(super) fn eval(&self, source: &str, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Self::SeparateContext(_) => self.with_context(context, |realm| {
                let result = realm.eval(Source::from_bytes(source.as_bytes()))?;
                realm.run_jobs()?;
                Ok(result)
            }),
            Self::EmbeddedRealm(_) => {
                let result = self.with_context(context, |realm| {
                    realm.eval(Source::from_bytes(source.as_bytes()))
                })?;
                // The creating Context is restored before jobs run. Boa jobs
                // retain their actual execution Realm and this same interner.
                context.run_jobs()?;
                Ok(result)
            }
        }
    }
}

pub(super) struct HostRealmScope {
    host_session: Arc<HostSession>,
}

pub(super) struct HostSession {
    can_block: bool,
    module_loading_policy: ModuleLoadingPolicy,
    pub(super) started_at: StdInstant,
    host_hooks: Arc<dyn HostHooks>,
    output_events: HostOutputEvents,
    pub(super) reports_tx: Sender<String>,
    reports_rx: Mutex<Receiver<String>>,
    agents: Mutex<Vec<AgentHandle>>,
}

enum HostOutputEvents {
    DelegateOnly,
    Capture(Mutex<Vec<HostOutputEvent>>),
}

struct AgentHandle {
    command_tx: Sender<AgentCommand>,
    join: JoinHandle<()>,
}

#[derive(Clone)]
enum AgentCommand {
    Broadcast(RawSharedArrayBuffer),
    Shutdown,
}

pub(super) struct AgentThreadState {
    pub(super) callback: Option<boa_engine::JsObject>,
    pub(super) report_tx: Sender<String>,
    pub(super) started_at: StdInstant,
    pub(super) leaving: bool,
}

impl HostRealmScope {
    pub(super) fn legacy(
        can_block: bool,
        module_loading_policy: ModuleLoadingPolicy,
        host_hooks: Arc<dyn HostHooks>,
    ) -> Self {
        Self::new(
            can_block,
            module_loading_policy,
            host_hooks,
            HostOutputEvents::DelegateOnly,
        )
    }

    pub(super) fn observed(
        can_block: bool,
        module_loading_policy: ModuleLoadingPolicy,
        host_hooks: Arc<dyn HostHooks>,
    ) -> Self {
        Self::new(
            can_block,
            module_loading_policy,
            host_hooks,
            HostOutputEvents::Capture(Mutex::new(Vec::new())),
        )
    }

    fn new(
        can_block: bool,
        module_loading_policy: ModuleLoadingPolicy,
        host_hooks: Arc<dyn HostHooks>,
        output_events: HostOutputEvents,
    ) -> Self {
        reset_host_realms();
        let (reports_tx, reports_rx) = mpsc::channel();
        let host_session = Arc::new(HostSession {
            can_block,
            module_loading_policy,
            started_at: StdInstant::now(),
            host_hooks,
            output_events,
            reports_tx,
            reports_rx: Mutex::new(reports_rx),
            agents: Mutex::new(Vec::new()),
        });
        CURRENT_HOST_SESSION.with(|session| {
            *session.borrow_mut() = Some(host_session.clone());
        });
        Self { host_session }
    }

    pub(super) fn finish_output_events(&self) -> Vec<HostOutputEvent> {
        // Agents share this execution's sink. Join them before taking the
        // transcript so a line emitted while shutdown is in flight is not
        // silently left behind in the session.
        self.host_session.shutdown();
        self.host_session.output_events.take()
    }
}

impl Drop for HostRealmScope {
    fn drop(&mut self) {
        self.host_session.shutdown();
        CURRENT_HOST_SESSION.with(|session| {
            *session.borrow_mut() = None;
        });
        CURRENT_AGENT_STATE.with(|state| {
            *state.borrow_mut() = None;
        });
        reset_host_realms();
    }
}

impl HostSession {
    pub(super) fn print_line(&self, text: String) {
        self.output_events.record(&text);
        self.host_hooks.print_line(&text);
    }

    pub(super) fn start_agent(self: &Arc<Self>, source: String) -> Result<(), ExecutionError> {
        let (command_tx, command_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let session = self.clone();
        let join = thread::Builder::new()
            .name("scriptw-test262-agent".to_string())
            .spawn(move || run_agent_thread(session, source, command_rx, ready_tx))
            .map_err(|err| ExecutionError::new(format!("failed to spawn agent thread: {err}")))?;

        match ready_rx.recv() {
            Ok(Ok(())) => {
                self.agents
                    .lock()
                    .expect("agent list mutex poisoned")
                    .push(AgentHandle { command_tx, join });
                Ok(())
            }
            Ok(Err(message)) => {
                let _ = join.join();
                self.reports_tx.send(message.clone()).map_err(|err| {
                    ExecutionError::new(format!("failed to queue agent error: {err}"))
                })?;
                Ok(())
            }
            Err(err) => {
                let _ = join.join();
                Err(ExecutionError::new(format!(
                    "failed to initialize agent thread: {err}"
                )))
            }
        }
    }

    pub(super) fn broadcast(&self, buffer: RawSharedArrayBuffer) -> usize {
        let mut sent = 0;
        let mut agents = self.agents.lock().expect("agent list mutex poisoned");
        let mut survivors = Vec::with_capacity(agents.len());

        for handle in agents.drain(..) {
            if handle
                .command_tx
                .send(AgentCommand::Broadcast(buffer.clone()))
                .is_ok()
            {
                sent += 1;
                survivors.push(handle);
            } else {
                let _ = handle.join.join();
            }
        }

        *agents = survivors;
        sent
    }

    pub(super) fn next_report(&self) -> Result<Option<String>, ExecutionError> {
        let receiver = self
            .reports_rx
            .lock()
            .map_err(|_| ExecutionError::new("report queue mutex poisoned"))?;
        match receiver.try_recv() {
            Ok(report) => Ok(Some(report)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Ok(None),
        }
    }

    fn shutdown(&self) {
        let mut agents = self.agents.lock().expect("agent list mutex poisoned");
        let handles = std::mem::take(&mut *agents);
        drop(agents);

        for handle in &handles {
            let _ = handle.command_tx.send(AgentCommand::Shutdown);
        }
        for handle in handles {
            let _ = handle.join.join();
        }
    }
}

impl HostOutputEvents {
    fn record(&self, text: &str) {
        if let Self::Capture(events) = self {
            events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(HostOutputEvent::PrintLine(text.to_string()));
        }
    }

    fn take(&self) -> Vec<HostOutputEvent> {
        let Self::Capture(events) = self else {
            return Vec::new();
        };
        let mut events = events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *events)
    }
}

fn reset_host_realms() {
    oracle_exception::reset_host_failures();
    HOST_REALMS.with(|store| {
        let mut store = store.borrow_mut();
        store.next_id = 0;
        store.realms.clear();
    });
}

pub(super) fn current_host_session(context: &mut Context) -> JsResult<Arc<HostSession>> {
    CURRENT_HOST_SESSION.with(|session| {
        session.borrow().clone().ok_or_else(|| {
            oracle_exception::host_failure(
                JsNativeError::typ()
                    .with_message("host session is not initialized")
                    .into(),
                context,
            )
        })
    })
}

pub(super) fn with_agent_state_mut<R>(
    action: impl FnOnce(&mut AgentThreadState) -> JsResult<R>,
) -> JsResult<R> {
    CURRENT_AGENT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let state = state.as_mut().ok_or_else(|| {
            JsError::from(
                JsNativeError::typ().with_message("agent thread state is not initialized"),
            )
        })?;
        action(state)
    })
}

pub(super) fn with_agent_state<R>(action: impl FnOnce(&AgentThreadState) -> R) -> Option<R> {
    CURRENT_AGENT_STATE.with(|state| state.borrow().as_ref().map(action))
}

// The owner cannot move across threads: it restores this thread's TLS.
struct AgentThreadScope {
    _thread: std::marker::PhantomData<Rc<()>>,
}
impl AgentThreadScope {
    fn enter(session: &Arc<HostSession>) -> Self {
        reset_host_realms();
        CURRENT_HOST_SESSION.with(|host| {
            *host.borrow_mut() = Some(session.clone());
        });
        CURRENT_AGENT_STATE.with(|state| {
            *state.borrow_mut() = Some(AgentThreadState {
                callback: None,
                report_tx: session.reports_tx.clone(),
                started_at: StdInstant::now(),
                leaving: false,
            });
        });

        Self {
            _thread: std::marker::PhantomData,
        }
    }
}
impl Drop for AgentThreadScope {
    fn drop(&mut self) {
        reset_host_realms();
        CURRENT_AGENT_STATE.with(|state| {
            *state.borrow_mut() = None;
        });
        CURRENT_HOST_SESSION.with(|host| {
            *host.borrow_mut() = None;
        });
    }
}

fn run_agent_thread(
    session: Arc<HostSession>,
    source: String,
    command_rx: Receiver<AgentCommand>,
    ready_tx: Sender<Result<(), String>>,
) {
    let _scope = AgentThreadScope::enter(&session);

    let mut context = match build_host_context(true, session.module_loading_policy.clone()) {
        Ok(context) => context,
        Err(err) => {
            let _ = ready_tx.send(Err(err.to_string()));
            return;
        }
    };

    let init_result = (|| -> Result<(), String> {
        install_host_globals(&mut context, &[]).map_err(|err| err.to_string())?;
        context
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(|err| format_js_error(err, &mut context).to_string())?;
        context
            .run_jobs()
            .map_err(|err| format_js_error(err, &mut context).to_string())?;
        Ok(())
    })();

    let _ = ready_tx.send(init_result.clone());

    if init_result.is_ok() {
        loop {
            if with_agent_state(|state| state.leaving).unwrap_or(true) {
                break;
            }

            match command_rx.recv() {
                Ok(AgentCommand::Broadcast(buffer)) => {
                    if let Err(message) = invoke_agent_callback(buffer, &mut context) {
                        let _ = session.reports_tx.send(message);
                        break;
                    }
                }
                Ok(AgentCommand::Shutdown) | Err(_) => break,
            }
        }
    }
}

fn invoke_agent_callback(
    buffer: RawSharedArrayBuffer,
    context: &mut Context,
) -> Result<(), String> {
    let callback = with_agent_state(|state| state.callback.clone());
    let Some(callback) = callback.flatten() else {
        return Ok(());
    };

    let shared = JsSharedArrayBuffer::from_buffer(buffer, context);
    let result = callback
        .call(&JsValue::undefined(), &[shared.into()], context)
        .map_err(|err| format_js_error(err, context).to_string())?;

    if let Some(object) = result.as_object() {
        if let Ok(promise) = JsPromise::from_object(object.clone()) {
            promise
                .await_blocking(context)
                .map_err(|err| format_js_error(err, context).to_string())?;
        } else {
            context
                .run_jobs()
                .map_err(|err| format_js_error(err, context).to_string())?;
        }
    } else {
        context
            .run_jobs()
            .map_err(|err| format_js_error(err, context).to_string())?;
    }

    Ok(())
}

pub(super) fn create_host_realm(
    context: &mut Context,
) -> Result<(u64, boa_engine::JsObject), boa_engine::JsError> {
    let session = current_host_session(context)?;
    let realm = match &session.module_loading_policy {
        ModuleLoadingPolicy::Embedded(graph) => HostRealmContext::EmbeddedRealm(
            EmbeddedGraphModuleLoader::create_realm(graph, context)?,
        ),
        ModuleLoadingPolicy::Filesystem | ModuleLoadingPolicy::RejectAll => {
            let context =
                build_host_context(session.can_block, session.module_loading_policy.clone())?;
            HostRealmContext::SeparateContext(Rc::new(RefCell::new(context)))
        }
    };
    let global = realm.with_context(context, |realm| {
        install_host_globals(realm, &[])
            .map_err(|err| JsNativeError::error().with_message(err.to_string()))?;
        Ok(realm.global_object())
    })?;

    let id = HOST_REALMS.with(|store| {
        let mut store = store.borrow_mut();
        let id = store.next_id;
        store.next_id += 1;
        store.realms.insert(id, realm);
        id
    });
    install_host_realm_eval(id, context)?;
    Ok((id, global))
}

pub(super) fn host_realm(realm_id: u64) -> JsResult<HostRealmContext> {
    // User source can create or enter another Realm. Keep the store borrow
    // shorter than that user call while retaining the actual owner.
    HOST_REALMS
        .with(|store| store.borrow().realms.get(&realm_id).cloned())
        .ok_or_else(|| {
            JsNativeError::reference()
                .with_message(format!("unknown host realm id {realm_id}"))
                .into()
        })
}

/// Snapshot the registered handles before inspecting exception provenance;
/// callers must not keep the thread-local store borrowed while entering a Realm.
pub(super) fn with_host_realms<T>(action: impl FnOnce(&BTreeMap<u64, HostRealmContext>) -> T) -> T {
    HOST_REALMS.with(|store| action(&store.borrow().realms))
}

pub(super) fn host_realms() -> Vec<HostRealmContext> {
    with_host_realms(|realms| realms.values().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_output_delegates_without_retaining_a_transcript() {
        let events = HostOutputEvents::DelegateOnly;
        events.record("streamed only");
        assert!(events.take().is_empty());

        let events = HostOutputEvents::Capture(Mutex::new(Vec::new()));
        events.record("captured");
        assert_eq!(
            events.take(),
            vec![HostOutputEvent::PrintLine("captured".to_string())]
        );
    }

    struct Silent;
    impl HostHooks for Silent {}

    #[test]
    fn agent_scope_clears_tls_and_realms_on_return_and_unwind() {
        for unwind in [false, true] {
            let parent =
                HostRealmScope::legacy(false, ModuleLoadingPolicy::RejectAll, Arc::new(Silent));
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _scope = AgentThreadScope::enter(&parent.host_session);
                CURRENT_HOST_SESSION.with(|value| assert!(value.borrow().is_some()));
                CURRENT_AGENT_STATE.with(|value| assert!(value.borrow().is_some()));
                HOST_REALMS.with(|value| value.borrow_mut().next_id = 9);
                if unwind {
                    panic!("agent initializer failed");
                }
            }));
            assert_eq!(result.is_err(), unwind);
            CURRENT_HOST_SESSION.with(|value| assert!(value.borrow().is_none()));
            CURRENT_AGENT_STATE.with(|value| assert!(value.borrow().is_none()));
            HOST_REALMS.with(|value| {
                assert_eq!(value.borrow().next_id, 0);
                assert!(value.borrow().realms.is_empty());
            });
        }
    }
}
