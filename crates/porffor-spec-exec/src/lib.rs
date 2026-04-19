use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration as StdDuration, Instant as StdInstant};

use boa_engine::builtins::array_buffer::SharedArrayBuffer as RawSharedArrayBuffer;
use boa_engine::builtins::promise::PromiseState;
use boa_engine::job::SimpleJobExecutor;
use boa_engine::module::{Module, ModuleLoader, ModuleRequest, Referrer};
use boa_engine::native_function::NativeFunction;
use boa_engine::object::builtins::{
    AlignedVec, JsArrayBuffer, JsPromise, JsSharedArrayBuffer, JsUint8Array,
};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::{Attribute, PropertyDescriptor};
use boa_engine::value::TryFromJs;
use boa_engine::{
    js_string, Context, JsArgs, JsError, JsNativeError, JsResult, JsString, JsValue, Source,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleHostConfig {
    pub module_root: Option<PathBuf>,
    pub test_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    message: String,
}

impl ExecutionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExecutionError {}

thread_local! {
    static HOST_REALMS: RefCell<HostRealmStore> = RefCell::new(HostRealmStore::default());
    static CURRENT_HOST_SESSION: RefCell<Option<Arc<HostSession>>> = const { RefCell::new(None) };
    static CURRENT_AGENT_STATE: RefCell<Option<AgentThreadState>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct HostRealmStore {
    next_id: u64,
    realms: BTreeMap<u64, Context>,
}

struct HostRealmScope {
    host_session: Arc<HostSession>,
}

struct HostSession {
    can_block: bool,
    started_at: StdInstant,
    reports_tx: Sender<String>,
    reports_rx: Mutex<Receiver<String>>,
    agents: Mutex<Vec<AgentHandle>>,
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

struct AgentThreadState {
    callback: Option<boa_engine::JsObject>,
    report_tx: Sender<String>,
    started_at: StdInstant,
    leaving: bool,
}

impl HostRealmScope {
    fn new(can_block: bool) -> Self {
        reset_host_realms();
        let (reports_tx, reports_rx) = mpsc::channel();
        let host_session = Arc::new(HostSession {
            can_block,
            started_at: StdInstant::now(),
            reports_tx,
            reports_rx: Mutex::new(reports_rx),
            agents: Mutex::new(Vec::new()),
        });
        CURRENT_HOST_SESSION.with(|session| {
            *session.borrow_mut() = Some(host_session.clone());
        });
        Self { host_session }
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
    fn start_agent(self: &Arc<Self>, source: String) -> Result<(), ExecutionError> {
        let (command_tx, command_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let session = self.clone();
        let join = thread::Builder::new()
            .name("porffor-test262-agent".to_string())
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

    fn broadcast(&self, buffer: RawSharedArrayBuffer) -> usize {
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

    fn next_report(&self) -> Result<Option<String>, ExecutionError> {
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

fn reset_host_realms() {
    HOST_REALMS.with(|store| {
        let mut store = store.borrow_mut();
        store.next_id = 0;
        store.realms.clear();
    });
}

fn current_host_session() -> JsResult<Arc<HostSession>> {
    CURRENT_HOST_SESSION.with(|session| {
        session.borrow().clone().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("host session is not initialized"))
        })
    })
}

fn with_agent_state_mut<R>(
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

fn with_agent_state<R>(action: impl FnOnce(&AgentThreadState) -> R) -> Option<R> {
    CURRENT_AGENT_STATE.with(|state| state.borrow().as_ref().map(action))
}

fn run_agent_thread(
    session: Arc<HostSession>,
    source: String,
    command_rx: Receiver<AgentCommand>,
    ready_tx: Sender<Result<(), String>>,
) {
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

    let mut context = match Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .can_block(true)
        .build()
    {
        Ok(context) => context,
        Err(err) => {
            let _ = ready_tx.send(Err(err.to_string()));
            CURRENT_AGENT_STATE.with(|state| {
                *state.borrow_mut() = None;
            });
            CURRENT_HOST_SESSION.with(|host| {
                *host.borrow_mut() = None;
            });
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

    CURRENT_AGENT_STATE.with(|state| {
        *state.borrow_mut() = None;
    });
    CURRENT_HOST_SESSION.with(|host| {
        *host.borrow_mut() = None;
    });
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

pub fn execute_script(
    source: &str,
    filename: Option<&str>,
    argv: &[String],
    can_block: bool,
) -> Result<ExecutionOutcome, ExecutionError> {
    let _host_realms = HostRealmScope::new(can_block);
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .can_block(can_block)
        .build()
        .map_err(|err| ExecutionError::new(err.to_string()))?;
    install_host_globals(&mut context, argv)?;
    context
        .eval(source_with_name(source, filename))
        .map_err(|err| format_js_error(err, &mut context))?;
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, &mut context))?;
    Ok(ExecutionOutcome {
        note: "spec-exec script completed in Rust host".to_string(),
    })
}

pub fn execute_module(
    source: &str,
    filename: Option<&str>,
    host: ModuleHostConfig,
    argv: &[String],
    can_block: bool,
) -> Result<ExecutionOutcome, ExecutionError> {
    let _host_realms = HostRealmScope::new(can_block);
    let module_path = normalize_module_path(filename).or_else(|| host.test_path.clone());
    let loader = Rc::new(Test262ModuleLoader::new(
        host.module_root.as_deref(),
        module_path.as_deref(),
    ));
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .module_loader(loader.clone())
        .can_block(can_block)
        .build()
        .map_err(|err| ExecutionError::new(err.to_string()))?;
    install_host_globals(&mut context, argv)?;

    let module = Module::parse(source_with_name(source, filename), None, &mut context)
        .map_err(|err| format_js_error(err, &mut context))?;
    loader.insert(
        module_path.clone().unwrap_or_else(|| loader.entry_path()),
        LoadedModuleKind::Source,
        module.clone(),
    );
    let promise = module.load_link_evaluate(&mut context);
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, &mut context))?;

    match promise.state() {
        PromiseState::Fulfilled(_) => Ok(ExecutionOutcome {
            note: format!(
                "spec-exec module completed in Rust host{}",
                module_path
                    .as_deref()
                    .map(|path| format!(" ({})", path.display()))
                    .unwrap_or_default()
            ),
        }),
        PromiseState::Rejected(err) => Err(format_opaque_error(err, &mut context)),
        PromiseState::Pending => Err(ExecutionError::new(
            "runtime module jobs are still pending after host job flush",
        )),
    }
}

#[derive(Debug, Default)]
struct Test262ModuleLoader {
    root: PathBuf,
    entry_path: PathBuf,
    module_map: RefCell<BTreeMap<(PathBuf, LoadedModuleKind), Module>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LoadedModuleKind {
    Source,
    Json,
    Text,
    Bytes,
}

impl Test262ModuleLoader {
    fn new(module_root: Option<&Path>, test_path: Option<&Path>) -> Self {
        let entry_path = test_path
            .map(normalize_absolute_path)
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from("main.mjs")));
        let root = module_root
            .map(normalize_absolute_path)
            .or_else(|| entry_path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from(".")));
        Self {
            root,
            entry_path,
            module_map: RefCell::new(BTreeMap::new()),
        }
    }

    fn entry_path(&self) -> PathBuf {
        self.entry_path.clone()
    }

    fn insert(&self, path: PathBuf, kind: LoadedModuleKind, module: Module) {
        self.module_map.borrow_mut().insert((path, kind), module);
    }

    fn get(&self, path: &Path, kind: LoadedModuleKind) -> Option<Module> {
        self.module_map
            .borrow()
            .get(&(path.to_path_buf(), kind))
            .cloned()
    }

    fn resolve_path(&self, referrer: Referrer, specifier: &JsString) -> JsResult<PathBuf> {
        let specifier = specifier.to_std_string_escaped();
        let specifier_path = Path::new(&specifier);
        if specifier_path.is_absolute() {
            return Ok(normalize_absolute_path(specifier_path));
        }

        let base_dir = if specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier == "."
            || specifier == ".."
        {
            referrer
                .path()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| {
                    self.entry_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.root.clone())
                })
        } else {
            self.root.clone()
        };

        Ok(normalize_absolute_path(&base_dir.join(specifier_path)))
    }

    fn request_kind(request: &ModuleRequest) -> JsResult<LoadedModuleKind> {
        let mut type_value = None;
        for attribute in request.attributes() {
            if attribute.key().to_std_string_escaped() == "type" {
                type_value = Some(attribute.value().to_std_string_escaped());
                break;
            }
        }

        match type_value.as_deref() {
            None | Some("") => Ok(LoadedModuleKind::Source),
            Some("json") => Ok(LoadedModuleKind::Json),
            Some("text") => Ok(LoadedModuleKind::Text),
            Some("bytes") => Ok(LoadedModuleKind::Bytes),
            Some(other) => Err(JsNativeError::syntax()
                .with_message(format!("unsupported import attribute type `{other}`"))
                .into()),
        }
    }
}

impl ModuleLoader for Test262ModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let specifier = request.specifier();
        let short_path = specifier.to_std_string_escaped();
        let path = self.resolve_path(referrer, &specifier)?;
        let kind = Self::request_kind(&request)?;
        if let Some(module) = self.get(&path, kind) {
            return Ok(module);
        }

        let module = match kind {
            LoadedModuleKind::Source => {
                let source = std::fs::read_to_string(&path).map_err(|err| {
                    JsNativeError::typ()
                        .with_message(format!("could not open file `{short_path}`"))
                        .with_cause(boa_engine::JsError::from_opaque(
                            js_string!(err.to_string()).into(),
                        ))
                })?;
                let should_consider_harness = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| !name.contains("_FIXTURE"))
                    .unwrap_or(true);
                let inject_harness = should_consider_harness
                    && (source.contains("assert.")
                        || source.contains("assert(")
                        || source.contains("Test262Error"));
                let source = if inject_harness {
                    format!("{TEST262_STA_GLOBAL}\n{TEST262_ASSERT_GLOBAL}\n{source}")
                } else {
                    source
                };
                let path_string = path.to_string_lossy().into_owned();
                Module::parse(
                    source_with_name(&source, Some(&path_string)),
                    None,
                    &mut context.borrow_mut(),
                )
                .map_err(|err| {
                    JsNativeError::syntax()
                        .with_message(format!("could not parse module `{short_path}`"))
                        .with_cause(err)
                })?
            }
            LoadedModuleKind::Json => {
                let source = std::fs::read_to_string(&path).map_err(|err| {
                    JsNativeError::typ()
                        .with_message(format!("could not open file `{short_path}`"))
                        .with_cause(boa_engine::JsError::from_opaque(
                            js_string!(err.to_string()).into(),
                        ))
                })?;
                Module::parse_json(js_string!(source), &mut context.borrow_mut()).map_err(
                    |err| {
                        JsNativeError::syntax()
                            .with_message(format!("could not parse module `{short_path}`"))
                            .with_cause(err)
                    },
                )?
            }
            LoadedModuleKind::Text => {
                let source = std::fs::read_to_string(&path).map_err(|err| {
                    JsNativeError::typ()
                        .with_message(format!("could not open file `{short_path}`"))
                        .with_cause(boa_engine::JsError::from_opaque(
                            js_string!(err.to_string()).into(),
                        ))
                })?;
                Module::from_value_as_default(
                    JsValue::from(js_string!(source)),
                    &mut context.borrow_mut(),
                )
            }
            LoadedModuleKind::Bytes => {
                let source = std::fs::read(&path).map_err(|err| {
                    JsNativeError::typ()
                        .with_message(format!("could not open file `{short_path}`"))
                        .with_cause(boa_engine::JsError::from_opaque(
                            js_string!(err.to_string()).into(),
                        ))
                })?;
                let bytes = AlignedVec::from_iter(0, source);
                let value = {
                    let mut context = context.borrow_mut();
                    let array_buffer =
                        JsArrayBuffer::from_byte_block_immutable(bytes, &mut context)?;
                    let array = JsUint8Array::from_array_buffer(array_buffer, &mut context)?;
                    JsValue::from(array)
                };
                Module::from_value_as_default(value, &mut context.borrow_mut())
            }
        };
        self.insert(path, kind, module.clone());
        Ok(module)
    }
}

fn source_with_name<'a>(
    source_text: &'a str,
    filename: Option<&'a str>,
) -> Source<'a, boa_engine::parser::source::UTF8Input<&'a [u8]>> {
    let source = Source::from_bytes(source_text);
    match filename {
        Some(path) => source.with_path(Path::new(path)),
        None => source,
    }
}

fn normalize_module_path(filename: Option<&str>) -> Option<PathBuf> {
    filename
        .map(PathBuf::from)
        .map(|path| normalize_absolute_path(&path))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[allow(dead_code)]
const DATE_TIME_FORMAT_SHIM: &str = r#"
(function () {
  if (typeof Intl !== "object" || typeof Intl.DateTimeFormat !== "function") {
    return;
  }

  const NativeDateTimeFormat = Intl.DateTimeFormat;
  const nativePrototype = NativeDateTimeFormat.prototype;
  if (typeof nativePrototype.resolvedOptions === "function") {
    return;
  }

  const stateKey = "__porfDateTimeFormatState";
  const boundFormatKey = "__porfBoundFormat";
  const legacyConstructedSymbol = Symbol("IntlLegacyConstructedSymbol");
  const validStyles = new Set(["full", "long", "medium", "short"]);
  const validDayPeriods = new Set(["narrow", "short", "long"]);
  const validTimeZoneNames = new Set([
    "short",
    "long",
    "shortOffset",
    "longOffset",
    "shortGeneric",
    "longGeneric"
  ]);
  const validCalendars = new Set([
    "buddhist",
    "chinese",
    "coptic",
    "dangi",
    "ethioaa",
    "ethiopic",
    "gregory",
    "hebrew",
    "indian",
    "islamic-civil",
    "islamic-tbla",
    "islamic-umalqura",
    "iso8601",
    "japanese",
    "persian",
    "roc"
  ]);
  const validNumberingSystems = new Set(["arab", "latn"]);
  const validHourCycles = new Set(["h11", "h12", "h23", "h24"]);
  const explicitComponents = [
    "weekday",
    "era",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
    "timeZoneName"
  ];
  const optionOrder = [
    "localeMatcher",
    "calendar",
    "numberingSystem",
    "hour12",
    "hourCycle",
    "timeZone",
    "weekday",
    "era",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
    "timeZoneName",
    "formatMatcher",
    "dateStyle",
    "timeStyle"
  ];

  function normalizeAsciiString(value, label) {
    const text = String(value);
    const lower = text.toLowerCase();
    if (!/^[a-z0-9-]+$/.test(lower)) {
      throw new RangeError("Invalid " + label);
    }
    return lower;
  }

  function canonicalizeCalendar(value) {
    if (value === undefined) {
      return null;
    }
    const lower = normalizeAsciiString(value, "calendar");
    if (!/^[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*$/.test(lower)) {
      throw new RangeError("Invalid calendar");
    }
    if (lower === "islamicc") {
      return "islamic-civil";
    }
    if (lower === "ethiopic-amete-alem") {
      return "ethioaa";
    }
    if (lower === "islamic" || lower === "islamic-rgsa") {
      return "islamic-civil";
    }
    return validCalendars.has(lower) ? lower : null;
  }

  function canonicalizeNumberingSystem(value) {
    if (value === undefined) {
      return null;
    }
    const lower = normalizeAsciiString(value, "numberingSystem");
    if (!/^[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*$/.test(lower)) {
      throw new RangeError("Invalid numberingSystem");
    }
    return validNumberingSystems.has(lower) ? lower : null;
  }

  function canonicalizeTimeZone(value) {
    const text = String(value);
    const offsetPattern = /^([+-])(\d{2})(?::?(\d{2}))?$/;
    if (text.startsWith("+") || text.startsWith("-")) {
      const match = text.match(offsetPattern);
      if (!match) {
        throw new RangeError("Invalid timeZone");
      }
      const sign = match[1];
      const hours = Number(match[2]);
      const minutes = Number(match[3] || "00");
      if (hours > 23 || minutes > 59) {
        throw new RangeError("Invalid timeZone");
      }
      if (hours === 0 && minutes === 0) {
        return "+00:00";
      }
      return sign + String(hours).padStart(2, "0") + ":" + String(minutes).padStart(2, "0");
    }
    if (/^(?:GMT|UTC)$/i.test(text)) {
      return text.toUpperCase();
    }
    if (/^Etc\/GMT$/i.test(text)) {
      return "Etc/GMT";
    }
    if (/^Etc\/UTC$/i.test(text)) {
      return "Etc/UTC";
    }
    if (text.startsWith("Etc/")) {
      return text;
    }
    if (!/^[A-Za-z0-9_]+(?:\/[A-Za-z0-9_+\-]+)*$/.test(text)) {
      throw new RangeError("Invalid timeZone");
    }
    return text
      .split("/")
      .map(segment =>
        segment
          .split("_")
          .map(part =>
            part
              .split("-")
              .map(piece => piece ? piece[0].toUpperCase() + piece.slice(1).toLowerCase() : piece)
              .join("-")
          )
          .join("_")
      )
      .join("/");
  }

  function parseRequestedLocale(locales) {
    const defaultLocale = new NativeDateTimeFormat().resolvedOptions().locale;
    const requestedList = Intl.getCanonicalLocales(locales);
    const requested = requestedList.length ? requestedList[0] : defaultLocale;
    const locale = new Intl.Locale(requested);
    const base = locale.baseName || requested || defaultLocale;
    const keywords = Object.create(null);
    if (locale.calendar) {
      keywords.ca = String(locale.calendar).toLowerCase();
    }
    if (locale.numberingSystem) {
      keywords.nu = String(locale.numberingSystem).toLowerCase();
    }
    if (locale.hourCycle) {
      keywords.hc = String(locale.hourCycle).toLowerCase();
    }
    return { requested, base, keywords };
  }

  function buildResolvedLocale(base, retainedKeywords) {
    const keys = ["ca", "hc", "nu"].filter(key => retainedKeywords[key] !== undefined);
    if (!keys.length) {
      return base;
    }
    return base + "-u-" + keys.map(key => key + "-" + retainedKeywords[key]).join("-");
  }

  function defaultHourCycleForLocale(base) {
    if (base === "ja" || base.startsWith("ja-")) {
      return "h11";
    }
    if (base === "de" || base.startsWith("de-")) {
      return "h23";
    }
    return "h12";
  }

  function getOptions(locales, options) {
    if (options === null) {
      throw new TypeError("cannot convert 'null' or 'undefined' to object");
    }
    const requestedLocale = parseRequestedLocale(locales);
    const nativeResolved = new NativeDateTimeFormat(locales, options).resolvedOptions();
    const resolvedLocale = typeof nativeResolved.locale === "string" ? nativeResolved.locale : "en-US";
    const resolvedBase = new Intl.Locale(resolvedLocale).baseName || resolvedLocale;
    const optionsObject = Object.create(null);
    if (options !== undefined) {
      const sourceOptions = Object(options);
      for (const key of optionOrder) {
        if (Object.prototype.hasOwnProperty.call(sourceOptions, key)) {
          optionsObject[key] = sourceOptions[key];
        }
      }
    }
    const state = Object.assign(Object.create(null), {
      locale: resolvedLocale,
      calendar: nativeResolved.calendar || "gregory",
      numberingSystem: nativeResolved.numberingSystem || "latn",
      timeZone: nativeResolved.timeZone || "UTC"
    });
    const retainedKeywords = Object.create(null);
    const seen = Object.create(null);
    for (const key of optionOrder) {
      seen[key] = optionsObject[key];
    }

    const localeCalendar = canonicalizeCalendar(requestedLocale.keywords.ca);
    if (localeCalendar !== null) {
      state.calendar = localeCalendar;
      retainedKeywords.ca = localeCalendar;
    }
    const localeNumberingSystem = canonicalizeNumberingSystem(requestedLocale.keywords.nu);
    if (localeNumberingSystem !== null) {
      state.numberingSystem = localeNumberingSystem;
      retainedKeywords.nu = localeNumberingSystem;
    }
    const localeHourCycle = requestedLocale.keywords.hc;
    if (validHourCycles.has(localeHourCycle)) {
      state.hourCycle = localeHourCycle;
      state.hour12 = localeHourCycle === "h11" || localeHourCycle === "h12";
      retainedKeywords.hc = localeHourCycle;
    }

    if (seen.calendar !== undefined) {
      const calendar = canonicalizeCalendar(seen.calendar);
      if (calendar !== null) {
        state.calendar = calendar;
        if (retainedKeywords.ca !== calendar) {
          delete retainedKeywords.ca;
        }
      }
    }
    if (seen.numberingSystem !== undefined) {
      const numberingSystem = canonicalizeNumberingSystem(seen.numberingSystem);
      if (numberingSystem !== null) {
        state.numberingSystem = numberingSystem;
        if (retainedKeywords.nu !== numberingSystem) {
          delete retainedKeywords.nu;
        }
      }
    }
    if (seen.timeZone !== undefined) {
      state.timeZone = canonicalizeTimeZone(seen.timeZone);
    }
    if (seen.dateStyle !== undefined) {
      if (!validStyles.has(String(seen.dateStyle))) {
        throw new RangeError("Invalid dateStyle");
      }
      state.dateStyle = String(seen.dateStyle);
    }
    if (seen.timeStyle !== undefined) {
      if (!validStyles.has(String(seen.timeStyle))) {
        throw new RangeError("Invalid timeStyle");
      }
      state.timeStyle = String(seen.timeStyle);
    }
    if (seen.dayPeriod !== undefined) {
      if (!validDayPeriods.has(String(seen.dayPeriod))) {
        throw new RangeError("Invalid dayPeriod");
      }
      state.dayPeriod = String(seen.dayPeriod);
    }
    if (seen.timeZoneName !== undefined) {
      if (!validTimeZoneNames.has(String(seen.timeZoneName))) {
        throw new RangeError("Invalid timeZoneName");
      }
      state.timeZoneName = String(seen.timeZoneName);
    }
    if (seen.fractionalSecondDigits !== undefined) {
      const digits = Number(seen.fractionalSecondDigits);
      if (!Number.isFinite(digits) || digits < 1 || digits > 3) {
        throw new RangeError("Invalid fractionalSecondDigits");
      }
      state.fractionalSecondDigits = Math.floor(digits);
    }

    let hasExplicitComponent = false;
    for (const key of explicitComponents) {
      const value = seen[key];
      if (value === undefined) {
        continue;
      }
      hasExplicitComponent = true;
      if (key === "fractionalSecondDigits" || key === "dayPeriod" || key === "timeZoneName") {
        continue;
      }
      state[key] = String(value);
    }

    if ((state.dateStyle !== undefined || state.timeStyle !== undefined) && hasExplicitComponent) {
      throw new TypeError("dateStyle/timeStyle conflicts with explicit components");
    }

    if (seen.hour12 !== undefined) {
      delete retainedKeywords.hc;
    }

    if (
      state.dateStyle === undefined &&
      state.timeStyle === undefined &&
      state.weekday === undefined &&
      state.year === undefined &&
      state.month === undefined &&
      state.day === undefined &&
      state.hour === undefined &&
      state.minute === undefined &&
      state.second === undefined &&
      state.dayPeriod === undefined
    ) {
      state.year = "numeric";
      state.month = "numeric";
      state.day = "numeric";
      state.__porfDefaultDateOnly = true;
    }

    if (
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.timeStyle !== undefined
    ) {
      if (seen.hour12 !== undefined) {
        state.hour12 = Boolean(seen.hour12);
        state.hourCycle = state.hour12
          ? defaultHourCycleForLocale(resolvedBase) === "h11" ? "h11" : "h12"
          : "h23";
        delete retainedKeywords.hc;
      } else if (seen.hourCycle !== undefined) {
        const hourCycle = String(seen.hourCycle);
        if (!validHourCycles.has(hourCycle)) {
          throw new RangeError("Invalid hourCycle");
        }
        state.hourCycle = hourCycle;
        state.hour12 = hourCycle === "h11" || hourCycle === "h12";
        delete retainedKeywords.hc;
      } else if (state.hourCycle === undefined) {
        state.hourCycle = defaultHourCycleForLocale(resolvedBase);
        state.hour12 = state.hourCycle === "h11" || state.hourCycle === "h12";
      }
    } else {
      delete state.hourCycle;
      delete state.hour12;
    }

    state.locale = resolvedLocale;
    return state;
  }

  function getState(value) {
    if (typeof value !== "object" || value === null) {
      throw new TypeError("Intl.DateTimeFormat method called on incompatible receiver");
    }
    const legacy = value[legacyConstructedSymbol];
    if (legacy && typeof legacy === "object" && stateKey in legacy) {
      return legacy[stateKey];
    }
    if (stateKey in value) {
      return value[stateKey];
    }
    if (!(stateKey in value)) {
      throw new TypeError("Intl.DateTimeFormat method called on incompatible receiver");
    }
    return value[stateKey];
  }

  function extractDateTimeRecord(value) {
    if (value instanceof Date) {
      return {
        kind: "Date",
        year: value.getFullYear(),
        month: value.getMonth() + 1,
        day: value.getDate(),
        hour: value.getHours(),
        minute: value.getMinutes(),
        second: value.getSeconds(),
        millisecond: value.getMilliseconds(),
        calendar: "iso8601"
      };
    }
    if (typeof Temporal !== "object" || value === null || typeof value !== "object") {
      return null;
    }
    const name = value.constructor && typeof value.constructor.name === "string"
      ? value.constructor.name
      : "";
    if (name === "Instant") {
      const epochMilliseconds = typeof value.epochMilliseconds === "number"
        ? value.epochMilliseconds
        : Number(value.epochNanoseconds / 1000000n);
      const date = new Date(epochMilliseconds);
      return {
        kind: "Instant",
        year: date.getUTCFullYear(),
        month: date.getUTCMonth() + 1,
        day: date.getUTCDate(),
        hour: date.getUTCHours(),
        minute: date.getUTCMinutes(),
        second: date.getUTCSeconds(),
        millisecond: date.getUTCMilliseconds(),
        calendar: "iso8601"
      };
    }
    if (!name.startsWith("Plain")) {
      return null;
    }
    const month = typeof value.month === "number"
      ? value.month
      : typeof value.monthCode === "string" && /^M\d{2}$/.test(value.monthCode)
      ? Number(value.monthCode.slice(1))
      : undefined;
    return {
      kind: name,
      year: typeof value.year === "number" ? value.year : undefined,
      month,
      day: typeof value.day === "number" ? value.day : undefined,
      hour: typeof value.hour === "number" ? value.hour : undefined,
      minute: typeof value.minute === "number" ? value.minute : 0,
      second: typeof value.second === "number" ? value.second : 0,
      millisecond: typeof value.millisecond === "number" ? value.millisecond : 0,
      monthCode: typeof value.monthCode === "string" ? value.monthCode : undefined,
      calendar: typeof value.calendarId === "string" ? value.calendarId : "iso8601",
      era: typeof value.era === "string" ? value.era : undefined,
      eraYear: typeof value.eraYear === "number" ? value.eraYear : undefined
    };
  }

  function toDateValue(value) {
    const number = value === undefined ? Date.now() : +value;
    if (!Number.isFinite(number)) {
      throw new RangeError("Invalid time value");
    }
    const date = new Date(number);
    if (!Number.isFinite(date.getTime())) {
      throw new RangeError("Invalid time value");
    }
    return date;
  }

  function dayPeriodForHour(hour, width) {
    const buckets = width === "narrow"
      ? ["in the morning", "n", "in the afternoon", "in the evening", "at night"]
      : width === "short"
      ? ["in the morning", "noon", "in the afternoon", "in the evening", "at night"]
      : ["in the morning", "noon", "in the afternoon", "in the evening", "at night"];
    if (hour < 12) {
      return buckets[0];
    }
    if (hour === 12) {
      return buckets[1];
    }
    if (hour < 18) {
      return buckets[2];
    }
    if (hour < 21) {
      return buckets[3];
    }
    return buckets[4];
  }

  function monthName(month, width) {
    const short = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const long = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    return (width === "long" ? long : short)[(month || 1) - 1];
  }

  function cloneState(state) {
    const copy = Object.create(null);
    for (const key of Object.keys(state)) {
      copy[key] = state[key];
    }
    return copy;
  }

  function usesTimeFields(state) {
    return state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.timeStyle !== undefined;
  }

  function usesDateFields(state) {
    return state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.dateStyle !== undefined;
  }

  function supportsStandaloneTimeZoneName(record) {
    return !record ||
      record.kind === "Date" ||
      record.kind === "Instant";
  }

  function hasDateOverlap(record, state) {
    const wantsDate = state.dateStyle !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined;
    if (!wantsDate) {
      return false;
    }
    if (record.kind === "PlainTime") {
      return false;
    }
    if (record.kind === "PlainMonthDay") {
      return state.month !== undefined || state.day !== undefined || state.dateStyle !== undefined;
    }
    if (record.kind === "PlainYearMonth") {
      return state.year !== undefined || state.month !== undefined || state.era !== undefined || state.dateStyle !== undefined;
    }
    return true;
  }

  function hasTimeOverlap(record, state) {
    const wantsTime = state.timeStyle !== undefined ||
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.dayPeriod !== undefined;
    if (!wantsTime) {
      return false;
    }
    return record.kind === "PlainTime" ||
      record.kind === "PlainDateTime" ||
      record.kind === "Instant";
  }

  function effectiveStateForValue(state, record) {
    const effective = cloneState(state);
    if (record && state.__porfDefaultDateOnly === true) {
      if (record.kind === "Instant" || record.kind === "PlainDateTime") {
        effective.hour = "numeric";
        effective.minute = "numeric";
        effective.second = "numeric";
        effective.hourCycle = effective.hourCycle || "h12";
        effective.hour12 = effective.hour12 !== false;
      } else if (record.kind === "PlainTime") {
        delete effective.year;
        delete effective.month;
        delete effective.day;
        effective.hour = "numeric";
        effective.minute = "numeric";
        effective.second = "numeric";
        effective.hourCycle = effective.hourCycle || "h12";
        effective.hour12 = effective.hour12 !== false;
      } else if (record.kind === "PlainMonthDay") {
        delete effective.year;
      } else if (record.kind === "PlainYearMonth") {
        delete effective.day;
      }
    }
    return effective;
  }

  function validateTemporalState(state, record) {
    if (!record || (record.kind !== "PlainDate" &&
        record.kind !== "PlainTime" &&
        record.kind !== "PlainDateTime" &&
        record.kind !== "PlainMonthDay" &&
        record.kind !== "PlainYearMonth" &&
        record.kind !== "Instant")) {
      return;
    }
    const dateOverlap = hasDateOverlap(record, state);
    const timeOverlap = hasTimeOverlap(record, state);
    const requestedDate = state.dateStyle !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined;
    const requestedTime = state.timeStyle !== undefined ||
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.dayPeriod !== undefined;
    if ((requestedDate || requestedTime) && !dateOverlap && !timeOverlap) {
      throw new TypeError("Temporal value does not overlap with formatter options");
    }
    if (!dateOverlap) {
      delete state.weekday;
      delete state.era;
      delete state.year;
      delete state.month;
      delete state.day;
      delete state.dateStyle;
    }
    if (!timeOverlap) {
      delete state.hour;
      delete state.minute;
      delete state.second;
      delete state.fractionalSecondDigits;
      delete state.dayPeriod;
      delete state.timeStyle;
      delete state.timeZoneName;
    }
    if (record.kind === "PlainMonthDay") {
      delete state.year;
      delete state.era;
    }
    if (record.kind === "PlainYearMonth") {
      delete state.day;
      delete state.weekday;
    }
    if (record.kind === "PlainDate") {
      delete state.timeZoneName;
    }
    if (record.kind === "PlainTime") {
      delete state.timeZoneName;
    }
    if (record.kind !== "Instant") {
      delete state.timeZoneName;
    }
  }

  function defineResultProperty(object, key, value) {
    Object.defineProperty(object, key, {
      value,
      writable: true,
      enumerable: true,
      configurable: true
    });
  }

  function weekdayName(year, month, day, width) {
    const names = width === "long"
      ? ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
      : ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    return names[new Date(Date.UTC(year || 1970, (month || 1) - 1, day || 1)).getUTCDay()];
  }

  function formatDatePiece(state, year, month, day) {
    if (state.dateStyle === "full") {
      if (year !== undefined && month !== undefined && day !== undefined) {
        return weekdayName(year, month, day, "long") + ", " + monthName(month, "long") + " " + day + ", " + year;
      }
      if (month !== undefined && day !== undefined) {
        return monthName(month, "long") + " " + day;
      }
      if (month !== undefined && year !== undefined) {
        return monthName(month, "long") + " " + year;
      }
    }
    if (state.dateStyle === "long" || state.dateStyle === "medium") {
      const width = state.dateStyle === "medium" ? "short" : "long";
      if (year !== undefined && month !== undefined && day !== undefined) {
        return monthName(month, width) + " " + day + ", " + year;
      }
      if (month !== undefined && day !== undefined) {
        return monthName(month, width) + " " + day;
      }
      if (month !== undefined && year !== undefined) {
        return monthName(month, width) + " " + year;
      }
    }
    if (state.dateStyle === "short" || (state.locale.startsWith("en") && state.year !== undefined)) {
      const yearText = state.dateStyle === "short" || state.year === "2-digit"
        ? String(year).slice(-2).padStart(2, "0")
        : String(year);
      if (month !== undefined && day !== undefined && year !== undefined) {
        return String(month) + "/" + String(day) + "/" + yearText;
      }
      if (month !== undefined && day !== undefined) {
        return String(month) + "/" + String(day);
      }
      if (month !== undefined && year !== undefined) {
        return String(month) + "/" + yearText;
      }
    }
    if (state.locale.startsWith("en")) {
      if (month !== undefined && day !== undefined) {
        return String(month) + "/" + String(day);
      }
      if (month !== undefined && year !== undefined) {
        return String(month) + "/" + String(year);
      }
    }
    if (month !== undefined && day !== undefined && year !== undefined) {
      return String(year) + "-" + String(month).padStart(2, "0") + "-" + String(day).padStart(2, "0");
    }
    return [year, month, day].filter(value => value !== undefined).join("-");
  }

  function formatTimePiece(state, hour, minute, second) {
    const useTwelveHour = state.hour12 !== false;
    const showHour = state.hour !== undefined || state.timeStyle !== undefined;
    const showMinute = showHour || state.minute !== undefined || state.second !== undefined || state.fractionalSecondDigits !== undefined;
    const showSecond = state.second !== undefined || state.fractionalSecondDigits !== undefined || (state.timeStyle !== undefined && state.timeStyle !== "short");
    const pieces = [];
    if (showHour) {
      pieces.push(useTwelveHour ? String(hour % 12 || 12) : String(hour).padStart(2, "0"));
    }
    if (showMinute) {
      pieces.push(String(minute).padStart(2, "0"));
    }
    if (showSecond) {
      pieces.push(String(second).padStart(2, "0"));
    }
    let rendered = pieces.join(":");
    if (state.fractionalSecondDigits !== undefined) {
      const ms = state.__porfMillisecond ?? 0;
      rendered += "." + String(ms).padStart(3, "0").slice(0, state.fractionalSecondDigits);
    }
    if (useTwelveHour && showHour) {
      rendered += " " + (state.dayPeriod !== undefined
        ? dayPeriodForHour(hour, state.dayPeriod)
        : hour < 12 ? "AM" : "PM");
    }
    if (state.timeZoneName !== undefined) {
      rendered += " " + (state.timeZone === "UTC" || state.timeZone === "+00:00" ? "UTC" : state.timeZone);
    }
    return rendered;
  }

  function monthPartValue(state, month) {
    if (state.dateStyle === "full" || state.dateStyle === "long") {
      return monthName(month, "long");
    }
    if (state.dateStyle === "medium") {
      return monthName(month, "short");
    }
    if (state.month === "long") {
      return monthName(month, "long");
    }
    if (state.month === "short" || state.month === "narrow") {
      return monthName(month, "short");
    }
    if (state.month === "2-digit") {
      return String(month).padStart(2, "0");
    }
    return String(month);
  }

  function formatValue(state, value) {
    const record = extractDateTimeRecord(value);
    const effectiveState = effectiveStateForValue(state, record);
    validateTemporalState(effectiveState, record);
    const date = record ? null : toDateValue(value);
    const year = record ? record.year : date.getFullYear();
    const month = record ? record.month : date.getMonth() + 1;
    const day = record ? record.day : date.getDate();
    const hour = record ? (record.hour ?? 0) : date.getHours();
    const minute = record ? (record.minute ?? 0) : date.getMinutes();
    const second = record ? (record.second ?? 0) : date.getSeconds();
    effectiveState.__porfMillisecond = record ? (record.millisecond ?? 0) : date.getMilliseconds();
    if (
      effectiveState.dayPeriod !== undefined &&
      !usesDateFields(effectiveState) &&
      !usesTimeFields(Object.assign(cloneState(effectiveState), { dayPeriod: undefined }))
    ) {
      return dayPeriodForHour(hour, effectiveState.dayPeriod);
    }

    if (
      effectiveState.dayPeriod !== undefined &&
      effectiveState.hour !== undefined &&
      effectiveState.minute === undefined &&
      effectiveState.second === undefined &&
      effectiveState.year === undefined &&
      effectiveState.month === undefined &&
      effectiveState.day === undefined &&
      effectiveState.weekday === undefined &&
      effectiveState.era === undefined
    ) {
      const hour12 = hour % 12 || 12;
      return String(hour12) + " " + dayPeriodForHour(hour, effectiveState.dayPeriod);
    }

    const pieces = [];
    if (usesDateFields(effectiveState)) {
      let datePiece = formatDatePiece(effectiveState, year, month, day);
      if (effectiveState.weekday !== undefined && effectiveState.dateStyle !== "full") {
        datePiece = weekdayName(year, month, day, effectiveState.weekday) + ", " + datePiece;
      }
      if (effectiveState.era !== undefined && year !== undefined) {
        datePiece += year <= 0 ? " BC" : " AD";
      }
      pieces.push(datePiece);
    }
    if (usesTimeFields(effectiveState)) {
      pieces.push(formatTimePiece(effectiveState, hour, minute, second));
    } else if (effectiveState.timeZoneName !== undefined && pieces.length && supportsStandaloneTimeZoneName(record)) {
      pieces[pieces.length - 1] += " " + (effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone);
    }
    if (!pieces.length) {
      pieces.push(
        record
          ? [year, month, day].filter(value => value !== undefined).join("-")
          : date.toISOString()
      );
    }
    return pieces.length > 1 ? pieces.join(", ") : pieces[0];
  }

  function formatParts(state, value) {
    const record = extractDateTimeRecord(value);
    const effectiveState = effectiveStateForValue(state, record);
    validateTemporalState(effectiveState, record);
    const date = record ? null : toDateValue(value);
    const year = record ? record.year : date.getFullYear();
    const month = record ? record.month : date.getMonth() + 1;
    const day = record ? record.day : date.getDate();
    const hour = record ? (record.hour ?? 0) : date.getHours();
    const minute = record ? (record.minute ?? 0) : date.getMinutes();
    const second = record ? (record.second ?? 0) : date.getSeconds();
    effectiveState.__porfMillisecond = record ? (record.millisecond ?? 0) : date.getMilliseconds();
    const parts = [];
    if (
      effectiveState.dayPeriod !== undefined &&
      !usesDateFields(effectiveState) &&
      !usesTimeFields(Object.assign(cloneState(effectiveState), { dayPeriod: undefined }))
    ) {
      return [{ type: "dayPeriod", value: dayPeriodForHour(hour, effectiveState.dayPeriod) }];
    }
    if (effectiveState.weekday !== undefined) {
      parts.push({ type: "weekday", value: weekdayName(year, month, day, effectiveState.weekday) });
      parts.push({ type: "literal", value: ", " });
    }
    if (effectiveState.month !== undefined || effectiveState.dateStyle !== undefined) {
      if (month !== undefined) {
        parts.push({ type: "month", value: monthPartValue(effectiveState, month) });
      }
    }
    if (effectiveState.day !== undefined || (effectiveState.dateStyle !== undefined && day !== undefined)) {
      if (parts.length && parts[parts.length - 1].type !== "literal") {
        const separator = effectiveState.dateStyle === "full" || effectiveState.dateStyle === "long" || effectiveState.dateStyle === "medium"
          ? " "
          : "/";
        parts.push({ type: "literal", value: separator });
      }
      parts.push({ type: "day", value: String(day) });
    }
    if (effectiveState.year !== undefined || (effectiveState.dateStyle !== undefined && year !== undefined)) {
      if (parts.length && parts[parts.length - 1].type !== "literal") {
        parts.push({ type: "literal", value: effectiveState.dateStyle === "short" ? "/" : ", " });
      }
      parts.push({
        type: "year",
        value: effectiveState.dateStyle === "short" || effectiveState.year === "2-digit"
          ? String(year).slice(-2).padStart(2, "0")
          : String(year)
      });
    }
    if (effectiveState.era !== undefined && year !== undefined) {
      parts.push({ type: "literal", value: " " });
      parts.push({ type: "era", value: year <= 0 ? "BC" : "AD" });
    }
    if ((effectiveState.calendar === "chinese" || effectiveState.calendar === "dangi") && year !== undefined) {
      parts.push({ type: "literal", value: " " });
      parts.push({ type: "relatedYear", value: String(year) });
    }
    if (usesTimeFields(effectiveState)) {
      if (parts.length) {
        parts.push({ type: "literal", value: ", " });
      }
      const useTwelveHour = effectiveState.hour12 !== false;
      parts.push({ type: "hour", value: useTwelveHour ? String(hour % 12 || 12) : String(hour).padStart(2, "0") });
      if (effectiveState.hour !== undefined || effectiveState.minute !== undefined || effectiveState.second !== undefined || effectiveState.timeStyle !== undefined || effectiveState.fractionalSecondDigits !== undefined) {
        parts.push({ type: "literal", value: ":" });
        parts.push({ type: "minute", value: String(minute).padStart(2, "0") });
      }
      if (effectiveState.second !== undefined || effectiveState.fractionalSecondDigits !== undefined || (effectiveState.timeStyle !== undefined && effectiveState.timeStyle !== "short")) {
        parts.push({ type: "literal", value: ":" });
        parts.push({ type: "second", value: String(second).padStart(2, "0") });
      }
      if (effectiveState.fractionalSecondDigits !== undefined) {
        parts.push({ type: "literal", value: "." });
        parts.push({
          type: "fractionalSecond",
          value: String(effectiveState.__porfMillisecond || 0).padStart(3, "0").slice(0, effectiveState.fractionalSecondDigits)
        });
      }
      if (useTwelveHour) {
        parts.push({ type: "literal", value: " " });
        parts.push({
          type: "dayPeriod",
          value: effectiveState.dayPeriod !== undefined
            ? dayPeriodForHour(hour, effectiveState.dayPeriod)
            : hour < 12 ? "AM" : "PM"
        });
      }
      if (effectiveState.timeZoneName !== undefined) {
        parts.push({ type: "literal", value: " " });
        parts.push({
          type: "timeZoneName",
          value: effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone
        });
      }
    } else if (effectiveState.timeZoneName !== undefined && supportsStandaloneTimeZoneName(record)) {
      parts.push({ type: "literal", value: " " });
      parts.push({
        type: "timeZoneName",
        value: effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone
      });
    }
    if (!parts.length) {
      parts.push({ type: "literal", value: formatValue(effectiveState, value) });
    }
    return parts;
  }

  function DateTimeFormat(locales, options) {
    if (!new.target && this && typeof this === "object" && (stateKey in this || this instanceof NativeDateTimeFormat)) {
      const legacyTarget = Reflect.construct(NativeDateTimeFormat, [], DateTimeFormat);
      Object.defineProperty(legacyTarget, stateKey, {
        value: getOptions(locales, options),
        configurable: true
      });
      Object.defineProperty(this, legacyConstructedSymbol, {
        value: legacyTarget,
        configurable: true
      });
      return this;
    }
    const instance = Reflect.construct(NativeDateTimeFormat, [], new.target || DateTimeFormat);
    Object.defineProperty(instance, stateKey, {
      value: getOptions(locales, options),
      configurable: true
    });
    return instance;
  }

  const prototypeMethods = {
    resolvedOptions() {
      const state = getState(this);
      const out = {};
      for (const key of [
        "locale",
        "calendar",
        "numberingSystem",
        "timeZone",
        "hourCycle",
        "hour12",
        "weekday",
        "era",
        "year",
        "month",
        "day",
        "dayPeriod",
        "hour",
        "minute",
        "second",
        "fractionalSecondDigits",
        "timeZoneName",
        "dateStyle",
        "timeStyle"
      ]) {
        if (state[key] !== undefined) {
          defineResultProperty(out, key, state[key]);
        }
      }
      return out;
    },
    formatToParts(date) {
      return formatParts(getState(this), date);
    },
    formatRange(startDate, endDate) {
      if (startDate === undefined || endDate === undefined) {
        throw new TypeError("formatRange requires two date arguments");
      }
      const state = getState(this);
      const left = extractDateTimeRecord(startDate);
      const right = extractDateTimeRecord(endDate);
      const leftTemporal = left && left.kind !== "Date";
      const rightTemporal = right && right.kind !== "Date";
      if ((leftTemporal || rightTemporal) && (!left || !right || left.kind !== right.kind)) {
        throw new TypeError("formatRange requires matching argument kinds");
      }
      const start = formatValue(state, startDate);
      const end = formatValue(state, endDate);
      return start === end ? start : start + " - " + end;
    },
    formatRangeToParts(startDate, endDate) {
      if (startDate === undefined || endDate === undefined) {
        throw new TypeError("formatRangeToParts requires two date arguments");
      }
      const state = getState(this);
      const left = extractDateTimeRecord(startDate);
      const right = extractDateTimeRecord(endDate);
      const leftTemporal = left && left.kind !== "Date";
      const rightTemporal = right && right.kind !== "Date";
      if ((leftTemporal || rightTemporal) && (!left || !right || left.kind !== right.kind)) {
        throw new TypeError("formatRangeToParts requires matching argument kinds");
      }
      const start = formatParts(state, startDate);
      const end = formatParts(state, endDate);
      if (formatValue(state, startDate) === formatValue(state, endDate)) {
        return start.map(part => Object.assign({ source: "shared" }, part));
      }
      return [
        ...start.map(part => Object.assign({ source: "startRange" }, part)),
        { type: "literal", value: " - ", source: "shared" },
        ...end.map(part => Object.assign({ source: "endRange" }, part))
      ];
    },
    supportedLocalesOf(locales) {
      const matcher = arguments[1] === undefined
        ? undefined
        : Object.prototype.hasOwnProperty.call(Object(arguments[1]), "localeMatcher")
        ? Object(arguments[1]).localeMatcher
        : undefined;
      return Intl.getCanonicalLocales(locales).filter(locale => {
        const lower = locale.toLowerCase();
        if (lower === "zxx" || lower === "") {
          return false;
        }
        const resolved = new DateTimeFormat([locale], matcher === undefined ? undefined : {
          localeMatcher: matcher
        }).resolvedOptions().locale;
        return locale === resolved || locale.indexOf(resolved) === 0;
      });
    }
  };
  const formatGetter = Object.getOwnPropertyDescriptor({
    get format() {
      const state = getState(this);
      if (!this[boundFormatKey]) {
        const target = {
          format(date) {
            return formatValue(state, date);
          }
        }.format;
        const bound = target.bind(undefined);
        Object.defineProperty(bound, "name", { value: "" });
        this[boundFormatKey] = bound;
      }
      return this[boundFormatKey];
    }
  }, "format").get;

  Object.defineProperty(DateTimeFormat, "length", {
    value: 0,
    configurable: true
  });
  Object.defineProperty(DateTimeFormat, "prototype", {
    value: nativePrototype,
    writable: false,
    enumerable: false,
    configurable: false
  });
  Object.defineProperty(nativePrototype, "constructor", {
    value: DateTimeFormat,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "resolvedOptions", {
    value: prototypeMethods.resolvedOptions,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "format", {
    get: formatGetter,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatToParts", {
    value: prototypeMethods.formatToParts,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatRange", {
    value: prototypeMethods.formatRange,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatRangeToParts", {
    value: prototypeMethods.formatRangeToParts,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, Symbol.toStringTag, {
    value: "Intl.DateTimeFormat",
    configurable: true
  });

  Object.defineProperty(DateTimeFormat, "supportedLocalesOf", {
    value: prototypeMethods.supportedLocalesOf,
    writable: true,
    configurable: true
  });

  Intl.DateTimeFormat = DateTimeFormat;
  function withDefaults(options, required, defaults) {
    if (options === undefined) {
      return defaults;
    }
    const source = options === null ? (() => { throw new TypeError("cannot convert 'null' or 'undefined' to object"); })() : Object(options);
    const merged = Object.assign(Object.create(null), source);
    let hasDate = false;
    let hasTime = false;
    for (const key of ["weekday", "year", "month", "day"]) {
      if (merged[key] !== undefined) {
        hasDate = true;
      }
    }
    for (const key of ["dayPeriod", "hour", "minute", "second", "fractionalSecondDigits"]) {
      if (merged[key] !== undefined) {
        hasTime = true;
      }
    }
    if (required === "all") {
      if (!hasDate && !hasTime && merged.dateStyle === undefined && merged.timeStyle === undefined) {
        return Object.assign(merged, defaults);
      }
    } else if (required === "date" && !hasDate && merged.dateStyle === undefined) {
      return Object.assign(merged, defaults);
    } else if (required === "time" && !hasTime && merged.timeStyle === undefined) {
      return Object.assign(merged, defaults);
    }
    return merged;
  }

  Date.prototype.toLocaleString = function toLocaleString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "all", {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
      second: "numeric"
    })).format(this);
  };
  Date.prototype.toLocaleDateString = function toLocaleDateString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "date", {
      year: "numeric",
      month: "numeric",
      day: "numeric"
    })).format(this);
  };
  Date.prototype.toLocaleTimeString = function toLocaleTimeString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "time", {
      hour: "numeric",
      minute: "numeric",
      second: "numeric"
    })).format(this);
  };

})();
"#;

const TEST262_STA_GLOBAL: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
const TEST262_ASSERT_GLOBAL: &str =
    include_str!("../../../test262/vendor/test262/harness/assert.js");

fn install_host_globals(context: &mut Context, argv: &[String]) -> Result<(), ExecutionError> {
    context
        .eval(Source::from_bytes(TEST262_STA_GLOBAL.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;
    context
        .eval(Source::from_bytes(TEST262_ASSERT_GLOBAL.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;

    context
        .register_global_builtin_callable(
            js_string!("print"),
            1,
            NativeFunction::from_fn_ptr(host_print),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(js_string!("gc"), 0, NativeFunction::from_fn_ptr(host_gc))
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfDetachArrayBuffer"),
            1,
            NativeFunction::from_fn_ptr(host_detach_array_buffer),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfCreateRealm"),
            0,
            NativeFunction::from_fn_ptr(host_create_realm),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfEvalScript"),
            1,
            NativeFunction::from_fn_ptr(host_eval_script),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentStart"),
            1,
            NativeFunction::from_fn_ptr(host_agent_start),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentBroadcast"),
            1,
            NativeFunction::from_fn_ptr(host_agent_broadcast),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentReceiveBroadcast"),
            1,
            NativeFunction::from_fn_ptr(host_agent_receive_broadcast),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentReport"),
            1,
            NativeFunction::from_fn_ptr(host_agent_report),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentGetReport"),
            0,
            NativeFunction::from_fn_ptr(host_agent_get_report),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentSleep"),
            1,
            NativeFunction::from_fn_ptr(host_agent_sleep),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentMonotonicNow"),
            0,
            NativeFunction::from_fn_ptr(host_agent_monotonic_now),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfAgentLeaving"),
            0,
            NativeFunction::from_fn_ptr(host_agent_leaving),
        )
        .map_err(|err| format_js_error(err, context))?;
    let argv_literal = json_string_array(argv);
    let bootstrap = format!(
        r#"
globalThis.__porfArgv = {argv_literal};
globalThis.$262 = {{
  global: globalThis,
  gc: gc,
  getGlobal(name) {{
    return globalThis[name];
  }},
  evalScript(code) {{
    if (typeof __porfEvalScript === "function") {{
      return __porfEvalScript(code);
    }}
    return (0, eval)(String(code));
  }},
  createRealm() {{
    if (typeof __porfCreateRealm === "function") {{
      return __porfCreateRealm();
    }}
    return {{
      global: globalThis,
      evalScript(code) {{
        return (0, eval)(String(code));
      }},
      destroy() {{}},
      getGlobal(name) {{
        return globalThis[name];
      }}
    }};
  }},
  detachArrayBuffer(buffer) {{
    return __porfDetachArrayBuffer(buffer);
  }},
  destroy() {{}},
  agent: {{
    start() {{
      return __porfAgentStart.apply(this, arguments);
    }},
    broadcast() {{
      return __porfAgentBroadcast.apply(this, arguments);
    }},
    receiveBroadcast() {{
      return __porfAgentReceiveBroadcast.apply(this, arguments);
    }},
    report() {{
      return __porfAgentReport.apply(this, arguments);
    }},
    getReport() {{
      return __porfAgentGetReport.apply(this, arguments);
    }},
    sleep() {{
      return __porfAgentSleep.apply(this, arguments);
    }},
    monotonicNow() {{
      return __porfAgentMonotonicNow.apply(this, arguments);
    }},
    leaving() {{
      return __porfAgentLeaving.apply(this, arguments);
    }}
  }}
}};
"#,
    );

    context
        .eval(Source::from_bytes(bootstrap.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;
    install_abstract_module_source_host_hook(context)?;
    install_html_dda_host_hook(context)?;
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, context))?;
    Ok(())
}

fn install_abstract_module_source_host_hook(context: &mut Context) -> Result<(), ExecutionError> {
    let abstract_module_source = context
        .intrinsics()
        .constructors()
        .abstract_module_source()
        .constructor();
    let test262 = context
        .global_object()
        .get(js_string!("$262"), context)
        .map_err(|err| format_js_error(err, context))?
        .as_object()
        .ok_or_else(|| ExecutionError::new("$262 host hook should be an object"))?;

    test262
        .define_property_or_throw(
            js_string!("AbstractModuleSource"),
            PropertyDescriptor::builder()
                .value(abstract_module_source)
                .writable(true)
                .enumerable(true)
                .configurable(true),
            context,
        )
        .map_err(|err| format_js_error(err, context))?;

    Ok(())
}

fn install_html_dda_host_hook(context: &mut Context) -> Result<(), ExecutionError> {
    let html_dda = context.intrinsics().objects().html_dda();
    let test262 = context
        .global_object()
        .get(js_string!("$262"), context)
        .map_err(|err| format_js_error(err, context))?
        .as_object()
        .ok_or_else(|| ExecutionError::new("$262 host hook should be an object"))?;

    test262
        .define_property_or_throw(
            js_string!("IsHTMLDDA"),
            PropertyDescriptor::builder()
                .value(html_dda)
                .writable(true)
                .enumerable(true)
                .configurable(true),
            context,
        )
        .map_err(|err| format_js_error(err, context))?;

    Ok(())
}

fn host_print(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let rendered = args
        .iter()
        .map(|value| {
            value
                .to_string(context)
                .map(|text| text.to_std_string_escaped())
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    if rendered.starts_with("Test262:AsyncTestFailure:") {
        return Err(JsNativeError::error().with_message(rendered).into());
    }
    if !rendered.is_empty() {
        println!("{rendered}");
    }
    Ok(JsValue::undefined())
}

fn host_gc(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn host_detach_array_buffer(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let Some(buffer) = args.first().and_then(JsValue::as_object) else {
        return Err(JsNativeError::typ()
            .with_message("detachArrayBuffer expects an ArrayBuffer")
            .into());
    };

    let buffer = JsArrayBuffer::from_object(buffer.clone())?;
    buffer.detach(&JsValue::undefined())?;
    Ok(JsValue::undefined())
}

fn host_agent_start(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if with_agent_state(|_| ()).is_some() {
        return Err(JsNativeError::typ()
            .with_message("nested agent.start is not supported")
            .into());
    }

    let source = args
        .first()
        .cloned()
        .unwrap_or_else(JsValue::undefined)
        .to_string(context)?
        .to_std_string_escaped();
    current_host_session()?
        .start_agent(source)
        .map_err(|err| JsNativeError::error().with_message(err.to_string()))?;
    Ok(JsValue::undefined())
}

fn host_agent_broadcast(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    if with_agent_state(|_| ()).is_some() {
        return Err(JsNativeError::typ()
            .with_message("nested agent.broadcast is not supported")
            .into());
    }

    let buffer = JsSharedArrayBuffer::try_from_js(args.get_or_undefined(0), context)?;
    let sent = current_host_session()?.broadcast(buffer.inner());
    Ok((sent as i32).into())
}

fn host_agent_receive_broadcast(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let callback = args.first().and_then(JsValue::as_callable).ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("receiveBroadcast expects a callable callback"),
        )
    })?;

    with_agent_state_mut(|state| {
        state.callback = Some(callback.clone());
        Ok(JsValue::undefined())
    })
}

fn host_agent_report(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let value = args
        .first()
        .cloned()
        .unwrap_or_else(JsValue::undefined)
        .to_string(context)?
        .to_std_string_escaped();

    if with_agent_state_mut(|state| {
        state.report_tx.send(value.clone()).map_err(|err| {
            JsError::from(
                JsNativeError::error().with_message(format!("failed to queue agent report: {err}")),
            )
        })?;
        Ok(JsValue::undefined())
    })
    .is_ok()
    {
        return Ok(JsValue::undefined());
    }

    current_host_session()?
        .reports_tx
        .send(value)
        .map_err(|err| {
            JsNativeError::error().with_message(format!("failed to queue report: {err}"))
        })?;
    Ok(JsValue::undefined())
}

fn host_agent_get_report(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    if with_agent_state(|_| ()).is_some() {
        return Ok(JsValue::null());
    }

    match current_host_session()?
        .next_report()
        .map_err(|err| JsNativeError::error().with_message(err.to_string()))?
    {
        Some(report) => Ok(js_string!(report).into()),
        None => Ok(JsValue::null()),
    }
}

fn host_agent_sleep(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let millis = args
        .first()
        .cloned()
        .unwrap_or_else(JsValue::undefined)
        .to_number(context)?
        .max(0.0);
    if millis.is_finite() && millis > 0.0 {
        thread::sleep(StdDuration::from_millis(millis as u64));
    }
    Ok(JsValue::undefined())
}

fn host_agent_monotonic_now(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let millis = if let Some(millis) =
        with_agent_state(|state| state.started_at.elapsed().as_secs_f64() * 1000.0)
    {
        millis
    } else {
        current_host_session()?.started_at.elapsed().as_secs_f64() * 1000.0
    };
    Ok(millis.into())
}

fn host_agent_leaving(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    if with_agent_state_mut(|state| {
        state.leaving = true;
        Ok(JsValue::undefined())
    })
    .is_ok()
    {
        return Ok(JsValue::undefined());
    }
    Ok(JsValue::undefined())
}

fn host_create_realm(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let (realm_id, global) = create_host_realm()?;

    let eval_script = NativeFunction::from_copy_closure(move |_this, args, context| {
        let source = args
            .first()
            .cloned()
            .unwrap_or_else(JsValue::undefined)
            .to_string(context)?
            .to_std_string_escaped();
        with_host_realm(realm_id, |realm| {
            let result = realm.eval(Source::from_bytes(source.as_bytes()))?;
            realm.run_jobs()?;
            Ok(result)
        })
    });
    let get_global = NativeFunction::from_copy_closure(move |_this, args, context| {
        let name = args
            .first()
            .cloned()
            .unwrap_or_else(JsValue::undefined)
            .to_string(context)?;
        with_host_realm(realm_id, |realm| realm.global_object().get(name, realm))
    });
    let destroy =
        NativeFunction::from_copy_closure(move |_this, _args, _context| Ok(JsValue::undefined()));

    let mut realm = ObjectInitializer::new(context);
    realm
        .property(js_string!("global"), global, Attribute::all())
        .function(eval_script, js_string!("evalScript"), 1)
        .function(get_global, js_string!("getGlobal"), 1)
        .function(destroy, js_string!("destroy"), 0);

    Ok(realm.build().into())
}

fn host_eval_script(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let source = args
        .first()
        .cloned()
        .unwrap_or_else(JsValue::undefined)
        .to_string(context)?
        .to_std_string_escaped();
    let result = context.eval(Source::from_bytes(source.as_bytes()))?;
    context.run_jobs()?;
    Ok(result)
}

fn create_host_realm() -> Result<(u64, boa_engine::JsObject), boa_engine::JsError> {
    let can_block = CURRENT_HOST_SESSION.with(|session| {
        session
            .borrow()
            .as_ref()
            .map(|session| session.can_block)
            .unwrap_or(false)
    });
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .can_block(can_block)
        .build()?;
    install_host_globals(&mut context, &[])
        .map_err(|err| JsNativeError::error().with_message(err.to_string()))?;
    let global = context.global_object();

    let (id, global) = HOST_REALMS.with(
        |store| -> Result<(u64, boa_engine::JsObject), boa_engine::JsError> {
            let mut store = store.borrow_mut();
            let id = store.next_id;
            store.next_id += 1;
            store.realms.insert(id, context);
            Ok((id, global))
        },
    )?;

    install_host_realm_eval(id)?;
    Ok((id, global))
}

fn with_host_realm(
    realm_id: u64,
    action: impl FnOnce(&mut Context) -> JsResult<JsValue>,
) -> JsResult<JsValue> {
    HOST_REALMS.with(|store| {
        let mut store = store.borrow_mut();
        let realm = store.realms.get_mut(&realm_id).ok_or_else(|| {
            JsNativeError::reference().with_message(format!("unknown host realm id {realm_id}"))
        })?;
        action(realm)
    })
}

fn install_host_realm_eval(realm_id: u64) -> JsResult<()> {
    let eval_impl = NativeFunction::from_copy_closure(move |_this, args, context| {
        let source = args
            .first()
            .cloned()
            .unwrap_or_else(JsValue::undefined)
            .to_string(context)?
            .to_std_string_escaped();
        with_host_realm(realm_id, |realm| {
            let result = realm.eval(Source::from_bytes(source.as_bytes()))?;
            realm.run_jobs()?;
            Ok(result)
        })
    });

    with_host_realm(realm_id, |realm| {
        realm.register_global_builtin_callable(js_string!("__porfHostRealmEval"), 1, eval_impl)?;
        realm.eval(Source::from_bytes(
            b"globalThis.eval = function eval(code) { return __porfHostRealmEval(code); };",
        ))?;
        Ok(JsValue::undefined())
    })
    .map(|_| ())
}

fn format_js_error(err: impl core::fmt::Display, _context: &mut Context) -> ExecutionError {
    ExecutionError::new(err.to_string())
}

fn format_opaque_error(value: JsValue, context: &mut Context) -> ExecutionError {
    let rendered = value
        .to_string(context)
        .map(|text| text.to_std_string_escaped())
        .unwrap_or_else(|_| value.display().to_string());
    ExecutionError::new(rendered)
}

fn json_string_array(values: &[String]) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&json_escape(value));
        out.push('"');
    }
    out.push(']');
    out
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Script;

    fn execute_script(
        source: &str,
        filename: Option<&str>,
        argv: &[String],
    ) -> Result<ExecutionOutcome, ExecutionError> {
        super::execute_script(source, filename, argv, false)
    }

    fn execute_module(
        source: &str,
        filename: Option<&str>,
        host: ModuleHostConfig,
        argv: &[String],
    ) -> Result<ExecutionOutcome, ExecutionError> {
        super::execute_module(source, filename, host, argv, false)
    }

    #[test]
    fn executes_simple_script() {
        execute_script("globalThis.answer = 40 + 2;", Some("script.js"), &[])
            .expect("spec exec script should run");
    }

    #[test]
    fn executes_simple_module() {
        execute_module(
            "export const value = 1;",
            Some("module.mjs"),
            ModuleHostConfig::default(),
            &[],
        )
        .expect("spec exec module should run");
    }

    #[test]
    fn detaches_array_buffer_via_host_hook() {
        execute_script(
            r#"
            const buffer = new ArrayBuffer(8);
            if (buffer.byteLength !== 8) {
              throw new Error("buffer should start attached");
            }
            __porfDetachArrayBuffer(buffer);
            if (buffer.byteLength !== 0) {
              throw new Error("buffer should be detached");
            }
            "#,
            Some("detach.js"),
            &[],
        )
        .expect("host detachArrayBuffer should detach real ArrayBuffers");
    }

    #[test]
    fn installs_uint8array_base64_and_hex_builtins() {
        execute_script(
            r#"
            const base64 = Uint8Array.fromBase64("x+/y");
            if (!(base64 instanceof Uint8Array) || base64.length !== 3) {
              throw new Error("fromBase64 should return Uint8Array");
            }
            if (base64[0] !== 199 || base64[1] !== 239 || base64[2] !== 242) {
              throw new Error("fromBase64 bytes mismatch");
            }

            const hex = Uint8Array.fromHex("666f6f");
            if (!(hex instanceof Uint8Array) || hex.length !== 3) {
              throw new Error("fromHex should return Uint8Array");
            }
            if (hex[0] !== 102 || hex[1] !== 111 || hex[2] !== 111) {
              throw new Error("fromHex bytes mismatch");
            }

            const target = new Uint8Array(6);
            const result = target.setFromBase64("ZXhhZg", { lastChunkHandling: "stop-before-partial" });
            if (result.read !== 4 || result.written !== 3) {
              throw new Error(`setFromBase64 result mismatch: ${result.read}/${result.written}`);
            }
            if (target[0] !== 101 || target[1] !== 120 || target[2] !== 97) {
              throw new Error("setFromBase64 write mismatch");
            }

            if ((new Uint8Array([199, 239, 242])).toBase64({ alphabet: "base64url" }) !== "x-_y") {
              throw new Error("toBase64 mismatch");
            }
            if ((new Uint8Array([0x66, 0x6f])).toHex() !== "666f") {
              throw new Error("toHex mismatch");
            }
            "#,
            Some("uint8array-base64-hex.js"),
            &[],
        )
        .expect("Uint8Array base64/hex builtins should work");
    }

    #[test]
    fn super_uninitialized_this_paths_throw_before_computed_expression() {
        execute_script(
            r#"
            let getOk = false;
            try {
              class A {}
              class B extends A {
                constructor() {
                  super[(function () { throw new Error("computed get should not run"); })()];
                }
              }
              new B();
            } catch (error) {
              getOk = error instanceof ReferenceError;
            }

            let deleteOk = false;
            try {
              class A {}
              class B extends A {
                constructor() {
                  delete super[(function () { throw new Error("computed delete should not run"); })()];
                }
              }
              new B();
            } catch (error) {
              deleteOk = error instanceof ReferenceError;
            }

            if (!getOk || !deleteOk) {
              throw new Error("super uninitialized-this ordering mismatch");
            }
            "#,
            Some("super-uninitialized-order.js"),
            &[],
        )
        .expect("super should read this before computed expression");
    }

    #[test]
    fn var_destructuring_resolves_binding_before_source_get() {
        execute_script(
            r#"
            var log = [];
            var sourceKey = {
              toString() {
                log.push("sourceKey");
                return "p";
              }
            };
            var source = {
              get p() {
                log.push("get source");
                return undefined;
              }
            };
            var env = new Proxy({}, {
              has(_target, key) {
                log.push("binding::" + key);
                return false;
              }
            });
            var defaultValue = 0;
            var varTarget;
            with (env) {
              var { [sourceKey]: varTarget = defaultValue } = source;
            }
            const expected = [
              "binding::source",
              "binding::sourceKey",
              "sourceKey",
              "binding::varTarget",
              "get source",
              "binding::defaultValue",
            ];
            if (JSON.stringify(log) !== JSON.stringify(expected)) {
              throw new Error(log.join(","));
            }
            "#,
            Some("destructuring-var-order.js"),
            &[],
        )
        .expect("var destructuring should resolve binding before source get");
    }

    #[test]
    fn typed_array_sort_allows_detach_during_compare_tonumber() {
        execute_script(
            r#"
            var ta = new Uint8Array(4);
            var ab = ta.buffer;
            var called = false;
            ta.sort(function(a, b) {
              __porfDetachArrayBuffer(ab);
              return {
                [Symbol.toPrimitive]() {
                  called = true;
                  return 0;
                }
              };
            });
            if (!called) {
              throw new Error("compareFn result should still go through ToNumber");
            }
            "#,
            Some("typedarray-sort-detach.js"),
            &[],
        )
        .expect("typed array sort should tolerate compare-time detachment");
    }

    #[test]
    fn create_realm_evaluates_in_a_distinct_global_object() {
        execute_script(
            r#"
            const other = $262.createRealm().global;
            const otherEval = other.eval;
            otherEval("var x = 23;");
            if (typeof x !== "undefined") {
              throw new Error("indirect eval should not leak into the current realm");
            }
            if (other.x !== 23) {
              throw new Error("indirect eval should bind on the created realm");
            }
            "#,
            Some("realm.js"),
            &[],
        )
        .expect("createRealm should produce a distinct global object");
    }

    #[test]
    fn create_realm_exposes_realm_specific_intrinsics() {
        execute_script(
            r#"
            const other = $262.createRealm().global;
            if (other.Array === Array) {
              throw new Error("created realms should expose distinct intrinsics");
            }
            if (Object.getPrototypeOf(new other.Object()) !== other.Object.prototype) {
              throw new Error("constructed objects should use the created realm prototype");
            }
            "#,
            Some("realm-intrinsics.js"),
            &[],
        )
        .expect("createRealm should expose realm-specific intrinsics");
    }

    #[test]
    fn installs_abstract_module_source_on_the_current_realm_host_hook() {
        execute_script(
            r#"
            if (typeof $262.AbstractModuleSource !== "function") {
              throw new Error("host hook should expose AbstractModuleSource");
            }
            if (Object.getPrototypeOf($262.AbstractModuleSource) !== Function.prototype) {
              throw new Error("AbstractModuleSource should use the current realm intrinsic");
            }
            const other = $262.createRealm().getGlobal("$262");
            if (other.AbstractModuleSource === $262.AbstractModuleSource) {
              throw new Error("created realms should expose distinct AbstractModuleSource intrinsics");
            }
            "#,
            Some("abstract-module-source.js"),
            &[],
        )
        .expect("host hook should expose a realm-specific AbstractModuleSource intrinsic");
    }

    #[test]
    fn installs_html_dda_on_the_current_realm_host_hook() {
        execute_script(
            r#"
            if ($262.IsHTMLDDA === undefined) {
              throw new Error("host hook should expose IsHTMLDDA");
            }
            if (typeof $262.IsHTMLDDA !== "undefined") {
              throw new Error("typeof should treat IsHTMLDDA like undefined");
            }
            if (!!$262.IsHTMLDDA) {
              throw new Error("IsHTMLDDA should be falsy");
            }
            if ($262.IsHTMLDDA != null) {
              throw new Error("IsHTMLDDA should abstract-equal null");
            }
            if ($262.IsHTMLDDA() !== null || $262.IsHTMLDDA("") !== null) {
              throw new Error("IsHTMLDDA calls should return null for no args and empty string");
            }
            const other = $262.createRealm().getGlobal("$262");
            if (other.IsHTMLDDA === $262.IsHTMLDDA) {
              throw new Error("created realms should expose distinct IsHTMLDDA objects");
            }
            "#,
            Some("html-dda.js"),
            &[],
        )
        .expect("host hook should expose a realm-specific HTMLDDA object");
    }

    #[test]
    fn html_dda_iterator_methods_are_not_treated_as_missing() {
        execute_script(
            r#"
            const items = {};
            items[Symbol.iterator] = $262.IsHTMLDDA;

            let threw = false;
            try {
              Array.from(items);
            } catch (error) {
              if (error.constructor !== TypeError) {
                throw error;
              }
              threw = true;
            }

            if (!threw) {
              throw new Error("Array.from should attempt to call the HTMLDDA iterator method");
            }
            "#,
            Some("html-dda-array-from.js"),
            &[],
        )
        .expect("Array.from should call an HTMLDDA @@iterator method");
    }

    #[test]
    fn installs_native_date_time_format() {
        execute_script(
            r#"
            const formatter = new Intl.DateTimeFormat("en", {
              calendar: "islamicc",
              timeZone: "Etc/UTC",
              dayPeriod: "long"
            });
            const resolved = formatter.resolvedOptions();
            if (resolved.calendar !== "islamic-civil") {
              throw new Error("calendar should be canonicalized");
            }
            if (resolved.timeZone !== "Etc/UTC") {
              throw new Error("timeZone should be preserved");
            }
            if (typeof formatter.format !== "function") {
              throw new Error("format getter should produce a function");
            }
            "#,
            Some("dtf.js"),
            &[],
        )
        .expect("native DateTimeFormat should be installed");
    }

    #[test]
    fn parses_and_executes_block_using_declaration() {
        execute_script(
            r#"
            {
              using x = null;
            }
            "#,
            Some("using-block.js"),
            &[],
        )
        .expect("block-scoped using declarations should parse and execute");
    }

    #[test]
    fn boa_script_parser_accepts_block_using_declaration() {
        let mut context = Context::default();
        Script::parse(
            Source::from_bytes(
                r#"
                {
                  using x = null;
                }
                "#,
            ),
            None,
            &mut context,
        )
        .expect("boa script parser should accept block-scoped using declarations");
    }

    #[test]
    fn await_using_function_initializer_preserves_binding_and_name() {
        execute_script(
            r#"
            let observed = "";
            let promiseState = "";
            Function.prototype[Symbol.dispose] = function () {};
            const promise = (async function () {
              await using arrow = () => {};
              const desc = Object.getOwnPropertyDescriptor(arrow, "name");
              observed = `${typeof arrow}:${String(arrow && arrow.name)}:${Object.hasOwn(arrow, "name")}:${!!desc}:${desc && desc.enumerable}:${desc && desc.writable}:${desc && desc.configurable}`;
            })();
            promiseState = `${typeof promise}:${typeof promise.then}`;
            if (observed !== "function:arrow:true:true:false:false:true") {
              throw new Error(observed);
            }
            if (promiseState !== "object:function") {
              throw new Error(promiseState);
            }
            "#,
            Some("await-using-fn-name.js"),
            &[],
        )
        .expect("await using should keep the initializer bound with its inferred function name");
    }

    #[test]
    fn native_date_time_format_exposes_builtin_shapes() {
        execute_script(
            r#"
            const formatGetter = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat.prototype, "format").get;
            if (Object.getPrototypeOf(Intl.DateTimeFormat) !== Function.prototype) {
              throw new Error("DateTimeFormat should inherit from Function.prototype");
            }
            const protoDesc = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat, "prototype");
            if (!protoDesc || protoDesc.writable || protoDesc.configurable) {
              throw new Error("DateTimeFormat.prototype should be fixed");
            }
            if (formatGetter.name !== "get format") {
              throw new Error("format getter should have built-in name");
            }
            if (Object.prototype.hasOwnProperty.call(formatGetter, "prototype")) {
              throw new Error("format getter should not expose prototype");
            }
            const bound = new Intl.DateTimeFormat("en-US").format;
            if (Object.prototype.hasOwnProperty.call(bound, "prototype")) {
              throw new Error("bound format should not expose prototype");
            }
            "#,
            Some("dtf-shape.js"),
            &[],
        )
        .expect("native DateTimeFormat should expose built-in shapes");
    }

    #[test]
    fn native_date_time_format_handles_null_options_and_to_locale_defaults() {
        execute_script(
            r#"
            let threw = false;
            try {
              new Intl.DateTimeFormat("en-US", null);
            } catch (err) {
              threw = err instanceof TypeError;
            }
            if (!threw) {
              throw new Error("null options should throw");
            }
            const formatted = new Date(0).toLocaleString("en-US");
            const reference = new Intl.DateTimeFormat("en-US", {
              year: "numeric",
              month: "numeric",
              day: "numeric",
              hour: "numeric",
              minute: "numeric",
              second: "numeric"
            }).format(new Date(0));
            if (formatted !== reference) {
              throw new Error("toLocaleString should use datetime defaults");
            }
            "#,
            Some("dtf-defaults.js"),
            &[],
        )
        .expect("native DateTimeFormat should apply null handling and locale defaults");
    }

    #[test]
    fn native_date_time_format_supports_legacy_constructed_symbol_unwrap() {
        execute_script(
            r#"
            const object = new Intl.DateTimeFormat();
            const legacy = Intl.DateTimeFormat.call(object);
            let seen = null;
            const proxy = new Proxy(legacy, {
              get(target, property, receiver) {
                seen = property;
                return Reflect.get(target, property, receiver);
              }
            });
            try {
              Intl.DateTimeFormat.prototype.resolvedOptions.call(proxy);
            } catch (_) {
            }
            if (typeof seen !== "symbol" || seen.description !== "IntlLegacyConstructedSymbol") {
              throw new Error("legacy constructed symbol should be observed during unwrap attempt");
            }
            "#,
            Some("dtf-legacy.js"),
            &[],
        )
        .expect("native DateTimeFormat should support legacy constructed symbol unwrap");
    }

    #[test]
    fn native_date_time_format_ignores_time_zone_name_for_plain_temporal_values() {
        execute_script(
            r#"
            const plainDateTime = new Temporal.PlainDateTime(2026, 1, 5, 11, 22);
            const formatter = new Intl.DateTimeFormat("en-US", {
              hour: "numeric",
              minute: "2-digit",
              timeZoneName: "long"
            });
            const formatted = formatter.format(plainDateTime);
            if (formatted.includes("UTC")) {
              throw new Error("plain temporal values should not include time zone names");
            }
            const parts = formatter.formatToParts(plainDateTime);
            if (parts.some(part => part.type === "timeZoneName")) {
              throw new Error("plain temporal values should not expose timeZoneName parts");
            }
            "#,
            Some("dtf-temporal-tzname.js"),
            &[],
        )
        .expect("native DateTimeFormat should suppress timeZoneName for plain Temporal values");
    }

    #[test]
    fn bootstrap_preserves_native_temporal_to_locale_string_shape() {
        execute_script(
            r#"
            const toLocaleString = Temporal.Instant.prototype.toLocaleString;
            if (toLocaleString.length !== 0) {
              throw new Error("Temporal toLocaleString length should stay native");
            }
            if (Object.prototype.hasOwnProperty.call(toLocaleString, "prototype")) {
              throw new Error("Temporal toLocaleString should not gain a prototype property");
            }
            try {
              new toLocaleString();
              throw new Error("Temporal toLocaleString should not be constructable");
            } catch (err) {
              if (!(err instanceof TypeError)) {
                throw err;
              }
            }
            try {
              toLocaleString.call({});
              throw new Error("Temporal toLocaleString should preserve native branding");
            } catch (err) {
              if (!(err instanceof TypeError)) {
                throw err;
              }
            }

            const formatted = new Intl.DateTimeFormat("en-US", {
              hour: "numeric",
              minute: "2-digit"
            }).format(new Temporal.PlainDateTime(2026, 1, 5, 11, 22));
            if (typeof formatted !== "string") {
              throw new Error("DateTimeFormat should still format Temporal values");
            }
            "#,
            Some("dtf-temporal-native-to-locale-string.js"),
            &[],
        )
        .expect("bootstrap should preserve native Temporal toLocaleString shape");
    }

    #[test]
    fn plain_year_month_add_reads_duration_before_options_and_rejects_lower_units() {
        execute_script(
            r#"
            const expected = [
              "days",
              "hours",
              "microseconds",
              "milliseconds",
              "minutes",
              "months",
              "nanoseconds",
              "seconds",
              "weeks",
              "years",
            ];
            const observed = [];
            const fields = {};
            for (const key of expected) {
              Object.defineProperty(fields, key, {
                get() {
                  observed.push(key);
                  return key === "days" ? 1 : 0;
                },
                configurable: true,
              });
            }

            const yearMonth = new Temporal.PlainYearMonth(2000, 5);
            try {
              yearMonth.add(fields, null);
              throw new Error("primitive options should still throw");
            } catch (err) {
              if (!(err instanceof TypeError)) {
                throw err;
              }
            }

            if (observed.join(",") !== expected.join(",")) {
              throw new Error("duration fields should be observed before primitive options");
            }

            for (const value of [{ days: 1 }, { hours: 1 }, { nanoseconds: 1 }]) {
              try {
                yearMonth.add(value);
                throw new Error("lower units should throw RangeError");
              } catch (err) {
                if (!(err instanceof RangeError)) {
                  throw err;
                }
              }
            }
            "#,
            Some("plain-year-month-add-order.js"),
            &[],
        )
        .expect("PlainYearMonth.add should observe duration before options and reject lower units");
    }

    #[test]
    fn temporal_duration_exactness_regressions() {
        execute_script(
            r#"
            const maxSafeInteger = 9007199254740991;
            const microseconds = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, maxSafeInteger, 0);
            const added = microseconds.add(
              new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, maxSafeInteger - 1, 0)
            );
            if (added.microseconds !== 18014398509481980) {
              throw new Error("Duration.add should collapse unsafe microsecond slots to float64-representable integers");
            }

            const rounded = new Temporal.Duration(0, 1, 0, 0, 10).round({
              smallestUnit: "months",
              roundingMode: "expand",
              relativeTo: new Temporal.PlainDate(2020, 1, 31),
            });
            if (rounded.months !== 2 || rounded.days !== 0) {
              throw new Error("relative month rounding should use the expanded window");
            }

            const zero = new Temporal.Duration();
            if (
              zero.round({
                smallestUnit: "hours",
                largestUnit: "years",
                relativeTo: new Temporal.PlainDateTime(1970, 1, 1),
              }).toString() !== "PT0S"
            ) {
              throw new Error("zero relative rounding should stay zero for PlainDateTime");
            }
            if (
              zero.round({
                smallestUnit: "hours",
                largestUnit: "years",
                relativeTo: new Temporal.ZonedDateTime(0n, "UTC"),
              }).toString() !== "PT0S"
            ) {
              throw new Error("zero relative rounding should stay zero for ZonedDateTime");
            }

            if (new Temporal.Duration(1, 0, 0, 0, 0, 0, 0, 0, 0, 1).toString() !== "P1YT0.000000001S") {
              throw new Error("Duration stringification should preserve mixed date and subsecond time parts");
            }

            const totalHours = new Temporal.Duration(0, 0, 0, 0, 816, 0, 0, 0, 0, 2049187497660)
              .total({ unit: "hours" });
            if (totalHours !== 816.56921874935) {
              throw new Error("Duration.total should round exact time totals once at the end");
            }
            "#,
            Some("temporal-duration-exactness.js"),
            &[],
        )
        .expect("Temporal duration exactness regressions should stay fixed");
    }

    #[test]
    fn plain_year_month_arithmetic_respects_iso_overflow_and_empty_partial_bags() {
        execute_script(
            r#"
            const lastMonth = new Temporal.PlainYearMonth(275760, 9);
            const addResult = lastMonth.add({ months: -1 });
            if (addResult.year !== 275760 || addResult.month !== 8) {
              throw new Error("adding negative months from the upper boundary should succeed");
            }
            const subtractResult = lastMonth.subtract({ months: 1 });
            if (subtractResult.year !== 275760 || subtractResult.month !== 8) {
              throw new Error("subtracting months from the upper boundary should succeed");
            }

            const base = new Temporal.PlainYearMonth(2023, 1);
            const constrain = base.add(new Temporal.Duration(1), { overflow: "constrain" });
            const reject = base.add(new Temporal.Duration(1), { overflow: "reject" });
            if (constrain.toString() !== reject.toString()) {
              throw new Error("ISO overflow option should not affect PlainYearMonth.add");
            }
            const march = new Temporal.PlainYearMonth(2023, 3);
            const marchConstrain = march.add({ months: -1 }, { overflow: "constrain" });
            const marchReject = march.add({ months: -1 }, { overflow: "reject" });
            if (marchConstrain.toString() !== marchReject.toString() || marchReject.toString() !== "2023-02") {
              throw new Error("ISO overflow option should not affect negative month arithmetic");
            }

            const yearMonth = Temporal.PlainYearMonth.from("2019-10");
            for (const value of [{}, { months: 12 }]) {
              try {
                yearMonth.with(value);
                throw new Error("empty partial year-month bags should throw");
              } catch (err) {
                if (!(err instanceof TypeError)) {
                  throw err;
                }
              }
            }
            "#,
            Some("plain-year-month-boundary.js"),
            &[],
        )
        .expect("PlainYearMonth arithmetic should preserve ISO overflow behavior and reject empty partial bags");
    }

    #[test]
    fn temporal_field_resolution_and_overflow_regressions() {
        execute_script(
            r#"
            try {
              Temporal.PlainDateTime.from({ monthCode: "M99L", day: 1, hour: 12 });
              throw new Error("missing year should beat invalid monthCode");
            } catch (error) {
              if (!(error instanceof TypeError)) {
                throw error;
              }
            }

            const observed = [];
            const options = new Proxy(
              {},
              {
                get(_target, key) {
                  observed.push(String(key));
                  return undefined;
                },
              }
            );
            try {
              Temporal.ZonedDateTime.from("2020-13-34T25:60:60+99:99[UTC]", options);
              throw new Error("invalid string should beat options access");
            } catch (error) {
              if (!(error instanceof RangeError)) {
                throw error;
              }
            }
            if (observed.length !== 0) {
              throw new Error("ZonedDateTime.from should parse invalid strings before reading options");
            }

            const leap = new Temporal.PlainMonthDay(2, 29, "iso8601", 1972);
            if (leap.with({ year: -999999 }).toString() !== "02-28") {
              throw new Error("ISO PlainMonthDay should use out-of-range years only for overflow");
            }

            try {
              new Temporal.PlainTime().with({ hour: 24 }, { overflow: "reject" });
              throw new Error("PlainTime.with should reject invalid fields when overflow is reject");
            } catch (error) {
              if (!(error instanceof RangeError)) {
                throw error;
              }
            }

            try {
              new Temporal.ZonedDateTime(0n, "UTC").with({});
              throw new Error("ZonedDateTime.with should reject empty partial objects");
            } catch (error) {
              if (!(error instanceof TypeError)) {
                throw error;
              }
            }

            const observedMonthDay = [];
            const observedFields = new Proxy(
              {
                year: {
                  valueOf() {
                    observedMonthDay.push("call year.valueOf");
                    return 2024.9;
                  },
                },
                get era() {
                  observedMonthDay.push("get era");
                  return "ce";
                },
                get eraYear() {
                  observedMonthDay.push("get eraYear");
                  return 2024;
                },
              },
              {
                get(target, key, receiver) {
                  observedMonthDay.push(`get ${String(key)}`);
                  return Reflect.get(target, key, receiver);
                },
              }
            );
            new Temporal.PlainMonthDay(5, 2).toPlainDate(observedFields);
            const expectedMonthDay = [
              "get year",
              "call year.valueOf",
            ];
            if (JSON.stringify(observedMonthDay) !== JSON.stringify(expectedMonthDay)) {
              throw new Error(`PlainMonthDay.toPlainDate should only read year when present: ${observedMonthDay.join(", ")}`);
            }
            "#,
            Some("temporal-field-resolution-overflow.js"),
            &[],
        )
        .expect("Temporal field resolution and overflow regressions should stay fixed");
    }

    #[test]
    fn temporal_zoneddatetime_with_uses_spec_default_offset() {
        execute_script(
            r#"
            const dstStartDay = Temporal.PlainDateTime.from("2000-04-02T12:00:01")
              .toZonedDateTime("America/Vancouver");
            const twoThirty = { hour: 2, minute: 30 };
            const implicit = dstStartDay.with(twoThirty);
            const explicit = dstStartDay.with(twoThirty, { offset: "prefer" });

            if (!implicit.equals(explicit)) {
              throw new Error("ZonedDateTime.with should default offset to prefer");
            }
            "#,
            Some("temporal-zdt-with-default-offset.js"),
            &[],
        )
        .expect("Temporal.ZonedDateTime.with should default offset to prefer");
    }

    #[test]
    fn native_date_time_format_filters_unsupported_locales_and_hc_extension() {
        execute_script(
            r#"
            const defaultLocale = new Intl.DateTimeFormat().resolvedOptions().locale;
            const supported = Intl.DateTimeFormat.supportedLocalesOf([defaultLocale, "zxx"]);
            if (supported.length !== 1 || supported[0] !== defaultLocale) {
              throw new Error("supportedLocalesOf should filter unsupported locales");
            }
            const withHourCycle = defaultLocale + "-u-hc-h11";
            const resolved = new Intl.DateTimeFormat(withHourCycle, {
              hour: "2-digit",
              hourCycle: "h23"
            }).resolvedOptions();
            if (resolved.locale.includes("-u-hc-")) {
              throw new Error("resolved locale should drop hc extension when hourCycle option is present");
            }
            "#,
            Some("dtf-supported-locales.js"),
            &[],
        )
        .expect("native DateTimeFormat should filter unsupported locales and drop hc extension");
    }

    #[test]
    fn supports_native_iterator_helpers() {
        execute_script(
            r#"
            const iterator = Iterator.from([1, 2, 3]).map(x => x * 2).drop(1);
            const values = iterator.toArray();
            if (values.length !== 2 || values[0] !== 4 || values[1] !== 6) {
              throw new Error("iterator helpers should transform and collect values");
            }
            if (!(iterator instanceof Iterator)) {
              throw new Error("iterator helper result should inherit from Iterator");
            }
            if (typeof Iterator.zip !== "function" || typeof Iterator.zipKeyed !== "function") {
              throw new Error("Iterator static helpers should exist");
            }
            "#,
            Some("iterator-helpers.js"),
            &[],
        )
        .expect("native Iterator helpers should be available");
    }

    #[test]
    fn iterator_helpers_support_concat_and_zip() {
        execute_script(
            r#"
            const concatValues = Iterator.concat([1, 2], new Set([3])).toArray();
            if (concatValues.join(",") !== "1,2,3") {
              throw new Error("Iterator.concat should flatten iterables in order");
            }
            const zipped = Iterator.zip([[1, 2], ["a", "b"]], { mode: "shortest" }).toArray();
            if (JSON.stringify(zipped) !== JSON.stringify([[1, "a"], [2, "b"]])) {
              throw new Error("Iterator.zip should zip iterables");
            }
            const keyed = Iterator.zipKeyed({ left: [1, 2], right: ["a", "b"] }, { mode: "shortest" }).toArray();
            if (JSON.stringify(keyed) !== JSON.stringify([{ left: 1, right: "a" }, { left: 2, right: "b" }])) {
              throw new Error("Iterator.zipKeyed should zip keyed iterables");
            }
            "#,
            Some("iterator-zip.js"),
            &[],
        )
        .expect("Iterator helpers should support concat and zip");
    }

    #[test]
    fn installs_intl_supported_helpers() {
        execute_script(
            r#"
            if (typeof Intl.supportedValuesOf !== "function") {
              throw new Error("supportedValuesOf should exist");
            }
            if (typeof Intl.DateTimeFormat.supportedLocalesOf !== "function") {
              throw new Error("supportedLocalesOf should exist");
            }
            const locales = Intl.DateTimeFormat.supportedLocalesOf(["en-US", "fr"]);
            if (locales.length !== 2) {
              throw new Error("supportedLocalesOf should return requested locales");
            }
            const values = Intl.supportedValuesOf("timeZone");
            if (!Array.isArray(values)) {
              throw new Error("supportedValuesOf should return an array");
            }
            "#,
            Some("intl-helpers.js"),
            &[],
        )
        .expect("Intl helpers should be installed");
    }

    #[test]
    fn promise_jobs_flush_before_return() {
        execute_script(
            "globalThis.done = false; Promise.resolve().then(() => { globalThis.done = true; });",
            Some("promise.js"),
            &[],
        )
        .expect("script with promises should run");
    }

    #[test]
    fn dataview_setter_rejects_immutable_backing_buffers_before_coercion() {
        execute_script(
            r#"
            const immutable = (new ArrayBuffer(8)).transferToImmutable();
            if (immutable.immutable !== true) {
              throw new Error(`immutable flag mismatch: ${immutable.immutable}`);
            }
            const view = new DataView(immutable);
            const calls = [];
            const byteOffset = {
              valueOf() {
                calls.push("byteOffset");
                return 0;
              }
            };
            const value = {
              valueOf() {
                calls.push("value");
                return 1;
              }
            };
            let threw = false;
            try {
              view.setUint8(byteOffset, value);
            } catch (error) {
              threw = true;
              if (error.constructor !== TypeError) {
                throw new Error(`wrong error constructor: ${error && error.constructor && error.constructor.name}:${error && error.name}:${error}`);
              }
              if (calls.length !== 0) {
                throw new Error("setUint8 should reject before coercing arguments");
              }
            }
            if (!threw) {
              throw new Error("setUint8 should throw TypeError for immutable buffers");
            }
            "#,
            Some("dataview-immutable.js"),
            &[],
        )
        .expect("DataView setters should reject immutable backing buffers before coercion");
    }

    #[test]
    fn resolves_sibling_fixture_module_from_test_path() {
        let root =
            std::env::temp_dir().join(format!("porffor-spec-exec-modules-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("temp module dir should exist");
        let test_path = root.join("module-import-resolution.js");
        let test_path_string = test_path.to_str().expect("utf-8 test path").to_string();
        let fixture_path = root.join("module-import-resolution_FIXTURE.js");
        std::fs::write(
            &fixture_path,
            "export default 42;\nexport const x = 'named';\nexport const y = 39;\n",
        )
        .expect("fixture module should be written");

        execute_module(
            "import foo from './module-import-resolution_FIXTURE.js';\nimport { x, y } from './module-import-resolution_FIXTURE.js';\nif (foo !== 42 || x !== 'named' || y !== 39) throw new Error('bad import');\n",
            Some(&test_path_string),
            ModuleHostConfig {
                module_root: Some(root.clone()),
                test_path: Some(test_path),
            },
            &[],
        )
        .expect("module host should resolve sibling fixture");
    }
}
