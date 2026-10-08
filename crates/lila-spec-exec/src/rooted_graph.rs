//! Retained completion roots and hook-free Boa graph inspection.
use super::*;
use boa_engine::builtins::function::OrdinaryFunction;
use boa_engine::builtins::proxy::Proxy;
use boa_engine::native_function::NativeFunctionObject;
use boa_engine::property::PropertyKey;
use boa_engine::realm::Realm;
use boa_engine::{JsObject, JsSymbol};
use lila_runtime::rooted_snapshot::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedGraphExecutionOutcome {
    pub completion: SnapshotCompletion,
    pub output_events: Vec<HostOutputEvent>,
    pub note: String,
}

enum RetainedCompletion {
    Normal(JsValue),
    Throw(JsValue),
}
impl RetainedCompletion {
    fn value(&self) -> &JsValue {
        match self {
            Self::Normal(value) | Self::Throw(value) => value,
        }
    }
    fn kind(&self) -> SnapshotCompletionKind {
        match self {
            Self::Normal(_) => SnapshotCompletionKind::Normal,
            Self::Throw(_) => SnapshotCompletionKind::Throw,
        }
    }
}
struct RetainedExecution {
    // Keep the execution's Context and its GC ownership alive during capture.
    _context: Context,
    entry: Realm,
    host_realms: HostRealmScope,
    completion: RetainedCompletion,
    note: RetainedNote,
}
enum RetainedNote {
    Text(String),
    Native(JsError),
}
impl RetainedNote {
    fn scalar(self) -> String {
        match self {
            Self::Text(note) => note,
            Self::Native(error) => error.to_string(),
        }
    }
    fn graph(self) -> String {
        match self {
            Self::Text(note) => note,
            Self::Native(_) => "uncaught ECMAScript native throw".into(),
        }
    }
}
impl RetainedExecution {
    fn scalar(self) -> ObservedExecutionOutcome {
        let value = observe_js_value(self.completion.value());
        ObservedExecutionOutcome {
            completion: match self.completion {
                RetainedCompletion::Normal(_) => ObservedCompletion::Normal(value),
                RetainedCompletion::Throw(_) => ObservedCompletion::Throw(value),
            },
            output_events: self.host_realms.finish_output_events(),
            note: self.note.scalar(),
        }
    }
    fn graph(self, limits: SnapshotLimits) -> ObservedGraphExecutionOutcome {
        let mut backend = BoaSnapshotBackend {
            entry: self.entry,
            inventory_checked: false,
        };
        let completion = SnapshotCompletion {
            kind: self.completion.kind(),
            outcome: capture_snapshot(&mut backend, self.completion.value(), limits),
        };
        ObservedGraphExecutionOutcome {
            completion,
            output_events: self.host_realms.finish_output_events(),
            note: self.note.graph(),
        }
    }
}
fn raw_throw_note(value: &JsValue) -> RetainedNote {
    let kind = match value.variant() {
        JsVariant::Undefined => "undefined",
        JsVariant::Null => "null",
        JsVariant::Boolean(_) => "boolean",
        JsVariant::Integer32(_) | JsVariant::Float64(_) => "number",
        JsVariant::String(_) => "string",
        JsVariant::BigInt(_) => "bigint",
        JsVariant::Symbol(_) => "symbol",
        JsVariant::Object(_) => "object",
    };
    RetainedNote::Text(format!("uncaught ECMAScript throw ({kind})"))
}
fn retained_error(error: &JsError, context: &mut Context) -> (RetainedCompletion, RetainedNote) {
    if let Some(value) = error.as_opaque() {
        (
            RetainedCompletion::Throw(value.clone()),
            raw_throw_note(value),
        )
    } else {
        (
            RetainedCompletion::Throw(error.to_opaque(context)),
            RetainedNote::Native(error.clone()),
        )
    }
}

fn script(
    source: &str,
    filename: Option<&str>,
    policy: ModuleLoadingPolicy,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
) -> Result<RetainedExecution, ExecutionError> {
    let host_realms = HostRealmScope::observed(can_block, policy.clone(), hooks);
    let mut context = build_host_context(can_block, policy.clone())
        .map_err(|error| ExecutionError::new(error.to_string()))?;
    install_host_globals(&mut context, argv)?;
    let entry = context.realm().clone();
    let script = parse_entry_script(source, filename, &policy, &mut context)?;
    let (mut completion, mut note) = match script.evaluate(&mut context) {
        Ok(value) => (
            RetainedCompletion::Normal(value),
            RetainedNote::Text("spec-exec script completed in Rust host".to_string()),
        ),
        Err(error) => retained_error(&error, &mut context),
    };
    // JsValue is a real GC root throughout the checkpoint. A queued throw
    // replaces a normal completion, never an already established primary throw.
    if let Err(error) = context.run_jobs() {
        if matches!(completion, RetainedCompletion::Normal(_)) {
            (completion, note) = retained_error(&error, &mut context);
        }
    }
    if matches!(completion, RetainedCompletion::Normal(_)) {
        check_test262_async_done(&mut context)?;
    }
    Ok(RetainedExecution {
        _context: context,
        entry,
        host_realms,
        completion,
        note,
    })
}
fn module(
    source: &str,
    filename: Option<&str>,
    host: ModuleHostConfig,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
) -> Result<RetainedExecution, ExecutionError> {
    if host.module_loading_policy == ModuleLoadingPolicy::RejectAll {
        return Err(ExecutionError::new(
            "module loading disabled by host policy",
        ));
    }
    let host_realms =
        HostRealmScope::observed(can_block, host.module_loading_policy.clone(), hooks);
    let (mut context, module, module_path) =
        prepare_module_context(source, filename, &host, argv, can_block)?;
    let entry = context.realm().clone();
    let load_promise = module.load(&mut context);
    context
        .run_jobs()
        .map_err(|error| format_js_error(error, &mut context))?;
    match load_promise.state() {
        PromiseState::Fulfilled(_) => {}
        PromiseState::Rejected(value) => {
            return Err(ExecutionError::new(JsError::from_opaque(value).to_string()))
        }
        PromiseState::Pending => {
            return Err(ExecutionError::new(
                "runtime module loading is still pending after host job flush",
            ))
        }
    }
    module
        .link(&mut context)
        .map_err(|error| format_js_error(error, &mut context))?;
    if let Some(prelude) = &host.prelude {
        let prelude = Script::parse(source_with_name(prelude, None), None, &mut context)
            .map_err(|error| format_js_error(error, &mut context))?;
        if let Err(error) = prelude.evaluate(&mut context) {
            let (completion, note) = retained_error(&error, &mut context);
            let _ = context.run_jobs();
            return Ok(RetainedExecution {
                _context: context,
                entry,
                host_realms,
                completion,
                note,
            });
        }
    }
    let promise = module.evaluate(&mut context);
    let job_throw = context
        .run_jobs()
        .err()
        .map(|error| retained_error(&error, &mut context));
    let normal_note = || {
        RetainedNote::Text(format!(
            "spec-exec module completed in Rust host{}",
            module_path
                .as_deref()
                .map(|path| format!(" ({})", path.display()))
                .unwrap_or_default()
        ))
    };
    let (completion, note) = match promise.state() {
        PromiseState::Fulfilled(value) => match job_throw {
            Some(thrown) => thrown,
            None => {
                check_test262_async_done(&mut context)?;
                (RetainedCompletion::Normal(value), normal_note())
            }
        },
        PromiseState::Rejected(value) => {
            let note = raw_throw_note(&value);
            (RetainedCompletion::Throw(value), note)
        }
        PromiseState::Pending => match job_throw {
            Some(thrown) => thrown,
            None => {
                return Err(ExecutionError::new(
                    "runtime module jobs are still pending after host job flush",
                ))
            }
        },
    };
    Ok(RetainedExecution {
        _context: context,
        entry,
        host_realms,
        completion,
        note,
    })
}

pub(super) fn observe_script_scalar(
    source: &str,
    filename: Option<&str>,
    policy: ModuleLoadingPolicy,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
) -> Result<ObservedExecutionOutcome, ExecutionError> {
    script(source, filename, policy, argv, can_block, hooks).map(RetainedExecution::scalar)
}
pub(super) fn observe_module_scalar(
    source: &str,
    filename: Option<&str>,
    host: ModuleHostConfig,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
) -> Result<ObservedExecutionOutcome, ExecutionError> {
    module(source, filename, host, argv, can_block, hooks).map(RetainedExecution::scalar)
}
pub fn observe_script_graph(
    source: &str,
    filename: Option<&str>,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
    limits: SnapshotLimits,
) -> Result<ObservedGraphExecutionOutcome, ExecutionError> {
    observe_script_graph_with_module_loading_policy(
        source,
        filename,
        ModuleLoadingPolicy::Filesystem,
        argv,
        can_block,
        hooks,
        limits,
    )
}
pub fn observe_script_graph_with_module_loading_policy(
    source: &str,
    filename: Option<&str>,
    policy: ModuleLoadingPolicy,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
    limits: SnapshotLimits,
) -> Result<ObservedGraphExecutionOutcome, ExecutionError> {
    script(source, filename, policy, argv, can_block, hooks)
        .map(|execution| execution.graph(limits))
}
pub fn observe_module_graph(
    source: &str,
    filename: Option<&str>,
    host: ModuleHostConfig,
    argv: &[String],
    can_block: bool,
    hooks: Arc<dyn HostHooks>,
    limits: SnapshotLimits,
) -> Result<ObservedGraphExecutionOutcome, ExecutionError> {
    module(source, filename, host, argv, can_block, hooks).map(|execution| execution.graph(limits))
}

struct BoaSnapshotBackend {
    entry: Realm,
    inventory_checked: bool,
}
impl BoaSnapshotBackend {
    fn check_inventory(&mut self, budget: &mut SnapshotBudget) -> Result<(), SnapshotRejection> {
        if self.inventory_checked {
            return Ok(());
        }
        // Only create_host_realm inserts here, after creating a fresh Realm;
        // the retained entry Realm is never inserted a second time.
        let count = HOST_REALMS
            .with(|store| store.borrow().realms.len().checked_add(1))
            .ok_or(SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Realms,
            })?;
        if count > budget.limits().get(SnapshotBudgetDimension::Realms) as usize {
            return Err(SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Realms,
            });
        }
        budget.work(count)?;
        self.inventory_checked = true;
        Ok(())
    }
}
impl SnapshotBackend for BoaSnapshotBackend {
    type Value = JsValue;
    type Object = JsObject;
    type Symbol = JsSymbol;
    type Realm = Realm;
    fn entry_realm(&self) -> Realm {
        self.entry.clone()
    }
    fn same_object(
        &mut self,
        left: &JsObject,
        right: &JsObject,
    ) -> Result<bool, SnapshotRejection> {
        Ok(left == right)
    }
    fn same_symbol(
        &mut self,
        left: &JsSymbol,
        right: &JsSymbol,
    ) -> Result<bool, SnapshotRejection> {
        Ok(left == right)
    }
    fn same_realm(&mut self, left: &Realm, right: &Realm) -> Result<bool, SnapshotRejection> {
        Ok(left == right)
    }
    fn value(
        &mut self,
        value: &JsValue,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotRawValue<JsObject, JsSymbol>, SnapshotRejection> {
        self.check_inventory(budget)?;
        Ok(match value.variant() {
            JsVariant::Undefined => SnapshotRawValue::Undefined,
            JsVariant::Null => SnapshotRawValue::Null,
            JsVariant::Boolean(value) => SnapshotRawValue::Boolean(value),
            JsVariant::Integer32(value) => {
                SnapshotRawValue::Number(ObservedNumber::from_f64(f64::from(value)).bits())
            }
            JsVariant::Float64(value) => {
                SnapshotRawValue::Number(ObservedNumber::from_f64(value).bits())
            }
            JsVariant::String(value) => {
                budget.work(value.len())?;
                SnapshotRawValue::String(budget.collect_utf16(value.iter())?)
            }
            JsVariant::BigInt(value) => {
                let (minimum, maximum) =
                    value
                        .decimal_byte_bounds()
                        .ok_or(SnapshotRejection::BudgetExceeded {
                            dimension: SnapshotBudgetDimension::BigIntDigits,
                        })?;
                let remaining = budget.remaining(SnapshotBudgetDimension::BigIntDigits) as usize;
                if minimum > remaining || maximum > remaining.saturating_add(1) {
                    return Err(SnapshotRejection::BudgetExceeded {
                        dimension: SnapshotBudgetDimension::BigIntDigits,
                    });
                }
                budget.work(maximum)?;
                SnapshotRawValue::BigInt(budget.take_bigint(value.to_string_radix(10))?)
            }
            JsVariant::Object(object) => SnapshotRawValue::Object(object),
            JsVariant::Symbol(symbol) => SnapshotRawValue::Symbol(symbol),
        })
    }
    fn symbol(
        &mut self,
        symbol: &JsSymbol,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotSymbolData, SnapshotRejection> {
        budget.work(SnapshotWellKnownSymbol::ALL.len())?;
        let description = symbol.with_raw_description(|units| {
            units
                .map(|units| {
                    budget.work(units.len())?;
                    budget.copy_utf16(units)
                })
                .transpose()
        })?;
        let origin = if let Some(name) = well_known(symbol) {
            SnapshotSymbolOrigin::WellKnown { name }
        } else {
            let key =
                boa_engine::builtins::symbol::with_registry_key_for_symbol(symbol, |units| {
                    units
                        .map(|units| {
                            budget.work(units.len())?;
                            budget.copy_utf16(units)
                        })
                        .transpose()
                })?;
            match key {
                Some(key) => SnapshotSymbolOrigin::Registry { key },
                None => SnapshotSymbolOrigin::Local {},
            }
        };
        Ok(SnapshotSymbolData {
            description,
            origin,
        })
    }
    fn object(
        &mut self,
        object: &JsObject,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotObjectData<JsValue, JsSymbol, Realm>, SnapshotRejection> {
        let kind =
            if object.is_ordinary() || object.is::<boa_engine::builtins::error::Error>() {
                SnapshotRawObjectKind::Ordinary
            } else if object.is_array() {
                SnapshotRawObjectKind::Array
            } else if let Some(function) = object.downcast_ref::<OrdinaryFunction>() {
                if !function.is_ordinary() {
                    return Err(SnapshotRejection::UnsupportedExotic {
                        exotic: SnapshotExotic::Other,
                    });
                }
                SnapshotRawObjectKind::Function {
                    constructable: object.is_constructor(),
                    realm: function.realm().clone(),
                }
            } else if let Some(function) = object.downcast_ref::<NativeFunctionObject>() {
                if function.is_html_dda() {
                    return Err(SnapshotRejection::UnsupportedExotic {
                        exotic: SnapshotExotic::Other,
                    });
                }
                let realm = function.realm().cloned().ok_or_else(|| {
                    SnapshotRejection::BackendInvariant {
                        detail: "native function has no initialized Realm".into(),
                    }
                })?;
                SnapshotRawObjectKind::Function {
                    constructable: object.is_constructor(),
                    realm,
                }
            } else {
                let exotic = if object.is::<Proxy>() {
                    SnapshotExotic::Proxy
                } else if object.is::<boa_engine::builtins::promise::Promise>() {
                    SnapshotExotic::Promise
                } else {
                    SnapshotExotic::Other
                };
                return Err(SnapshotRejection::UnsupportedExotic { exotic });
            };
        let raw = object.borrow();
        budget.work(1)?;
        if raw.raw_has_private_elements() {
            return Err(SnapshotRejection::UnsupportedExotic {
                exotic: SnapshotExotic::Other,
            });
        }
        let count = raw
            .raw_own_property_count()
            .ok_or(SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Properties,
            })?;
        if count > budget.remaining(SnapshotBudgetDimension::Properties) as usize {
            return Err(SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Properties,
            });
        }
        budget.work(
            count
                .checked_mul(3)
                .ok_or(SnapshotRejection::BudgetExceeded {
                    dimension: SnapshotBudgetDimension::Work,
                })?,
        )?;
        let descriptors = raw.raw_own_properties_bounded(count).ok_or_else(|| {
            SnapshotRejection::BackendInvariant {
                detail: "retained own-property count changed during snapshot".into(),
            }
        })?;
        let extensible = raw.raw_extensible();
        let prototype = raw.prototype().map_or_else(JsValue::null, JsValue::from);
        drop(raw);
        let mut properties = SnapshotProperties::default();
        for (key, descriptor) in descriptors {
            let key = match key {
                PropertyKey::Index(index) => {
                    SnapshotRawKey::String(index_key(index.get(), budget)?)
                }
                PropertyKey::String(text) => {
                    budget.work(text.len())?;
                    SnapshotRawKey::String(budget.collect_utf16(text.iter())?)
                }
                PropertyKey::Symbol(symbol) => SnapshotRawKey::Symbol(symbol),
            };
            let enumerable = descriptor.enumerable().unwrap_or(false);
            let configurable = descriptor.configurable().unwrap_or(false);
            let descriptor = if descriptor.is_accessor_descriptor() {
                SnapshotRawDescriptor::Accessor {
                    get: descriptor.get().cloned().unwrap_or_else(JsValue::undefined),
                    set: descriptor.set().cloned().unwrap_or_else(JsValue::undefined),
                    enumerable,
                    configurable,
                }
            } else if descriptor.is_data_descriptor() {
                SnapshotRawDescriptor::Data {
                    value: descriptor
                        .value()
                        .cloned()
                        .unwrap_or_else(JsValue::undefined),
                    writable: descriptor.writable().unwrap_or(false),
                    enumerable,
                    configurable,
                }
            } else {
                return Err(SnapshotRejection::BackendInvariant {
                    detail: "retained own property has no complete descriptor kind".into(),
                });
            };
            properties.push(budget, SnapshotRawProperty { key, descriptor })?;
        }
        let mut anchors = Vec::new();
        collect_anchors(&self.entry, object, budget, &mut anchors)?;
        HOST_REALMS.with(|store| -> Result<(), SnapshotRejection> {
            for owner in store.borrow().realms.values() {
                budget.work(1)?;
                match owner {
                    HostRealmContext::EmbeddedRealm(realm) => {
                        collect_anchors(realm, object, budget, &mut anchors)?
                    }
                    HostRealmContext::SeparateContext(context) => {
                        let context = context.try_borrow().map_err(|_| {
                            SnapshotRejection::BackendInvariant {
                                detail: "created Realm is active during snapshot".into(),
                            }
                        })?;
                        collect_anchors(context.realm(), object, budget, &mut anchors)?;
                    }
                }
            }
            Ok(())
        })?;
        Ok(SnapshotObjectData {
            kind,
            extensible,
            prototype,
            anchors,
            properties,
        })
    }
}

fn index_key(
    mut value: u32,
    budget: &mut SnapshotBudget,
) -> Result<SnapshotString, SnapshotRejection> {
    let mut units = [0u16; 10];
    let mut start = units.len();
    loop {
        start -= 1;
        units[start] = u16::from(b'0') + (value % 10) as u16;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    budget.work(units.len() - start)?;
    budget.copy_utf16(&units[start..])
}

fn well_known(symbol: &JsSymbol) -> Option<SnapshotWellKnownSymbol> {
    use SnapshotWellKnownSymbol::*;
    for name in SnapshotWellKnownSymbol::ALL.iter().copied() {
        let actual = match name {
            AsyncIterator => JsSymbol::async_iterator(),
            HasInstance => JsSymbol::has_instance(),
            IsConcatSpreadable => JsSymbol::is_concat_spreadable(),
            Iterator => JsSymbol::iterator(),
            Match => JsSymbol::r#match(),
            MatchAll => JsSymbol::match_all(),
            Replace => JsSymbol::replace(),
            Search => JsSymbol::search(),
            Species => JsSymbol::species(),
            Split => JsSymbol::split(),
            ToPrimitive => JsSymbol::to_primitive(),
            ToStringTag => JsSymbol::to_string_tag(),
            Unscopables => JsSymbol::unscopables(),
            Dispose => JsSymbol::dispose(),
            AsyncDispose => JsSymbol::async_dispose(),
        };
        if symbol == &actual {
            return Some(name);
        }
    }
    None
}
fn collect_anchors(
    realm: &Realm,
    object: &JsObject,
    budget: &mut SnapshotBudget,
    anchors: &mut Vec<SnapshotRawAnchor<Realm>>,
) -> Result<(), SnapshotRejection> {
    use SnapshotIntrinsic::*;
    let constructors = realm.intrinsics().constructors();
    for intrinsic in SnapshotIntrinsic::ALL.iter().copied() {
        budget.work(1)?;
        let candidate = match intrinsic {
            ObjectConstructor => constructors.object().constructor(),
            ObjectPrototype => constructors.object().prototype(),
            FunctionConstructor => constructors.function().constructor(),
            FunctionPrototype => constructors.function().prototype(),
            ArrayConstructor => constructors.array().constructor(),
            ArrayPrototype => constructors.array().prototype(),
            ErrorConstructor => constructors.error().constructor(),
            ErrorPrototype => constructors.error().prototype(),
            EvalErrorConstructor => constructors.eval_error().constructor(),
            EvalErrorPrototype => constructors.eval_error().prototype(),
            RangeErrorConstructor => constructors.range_error().constructor(),
            RangeErrorPrototype => constructors.range_error().prototype(),
            ReferenceErrorConstructor => constructors.reference_error().constructor(),
            ReferenceErrorPrototype => constructors.reference_error().prototype(),
            SyntaxErrorConstructor => constructors.syntax_error().constructor(),
            SyntaxErrorPrototype => constructors.syntax_error().prototype(),
            TypeErrorConstructor => constructors.type_error().constructor(),
            TypeErrorPrototype => constructors.type_error().prototype(),
            UriErrorConstructor => constructors.uri_error().constructor(),
            UriErrorPrototype => constructors.uri_error().prototype(),
            AggregateErrorConstructor => constructors.aggregate_error().constructor(),
            AggregateErrorPrototype => constructors.aggregate_error().prototype(),
        };
        if object == &candidate {
            anchors.push(SnapshotRawAnchor {
                realm: realm.clone(),
                intrinsic,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
