//! Native throw provenance and side-effect-free intrinsic constructor admission.
use super::*;
use boa_engine::builtins::error::{Error as ErrorData, ErrorKind};
use boa_engine::builtins::proxy::Proxy;
use boa_engine::context::intrinsics::StandardConstructors;
use boa_engine::{JsNativeErrorKind, JsObject};
use lila_runtime::{OracleExceptionPhase, OracleExceptionType};

thread_local! {
    // Original opaque objects keep host-failure provenance across a caught
    // rethrow without adding JS-visible fields or retaining a second model.
    static HOST_FAILURES: RefCell<Vec<JsObject>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn reset_host_failures() {
    HOST_FAILURES.with(|errors| errors.borrow_mut().clear());
}
pub(super) fn host_failure(error: JsError, context: &mut Context) -> JsError {
    let value = error.to_opaque(context);
    if let Some(object) = value.as_object() {
        HOST_FAILURES.with(|errors| errors.borrow_mut().push(object));
    }
    JsError::from_opaque(value)
}
/// A disabled host capability rejects with its original opaque value, while
/// retaining host provenance if JavaScript catches and rethrows it.
#[derive(Debug)]
pub(super) struct DisabledModuleLoader;
impl ModuleLoader for DisabledModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        _referrer: Referrer,
        _request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        Err(host_failure(
            JsNativeError::typ()
                .with_message("module loading disabled by host policy")
                .into(),
            &mut context.borrow_mut(),
        ))
    }
}
#[derive(Debug)]
pub(super) struct AmbientModuleLoader(Rc<boa_engine::module::SimpleModuleLoader>);
impl AmbientModuleLoader {
    pub(super) fn new() -> JsResult<Self> {
        boa_engine::module::SimpleModuleLoader::new(Path::new("."))
            .map(|loader| Self(Rc::new(loader)))
    }
}
impl ModuleLoader for AmbientModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        self.0
            .clone()
            .load_imported_module(referrer, request, context)
            .await
            .map_err(|error| host_failure(error, &mut context.borrow_mut()))
    }
}
pub(super) fn thrown(
    error: JsError,
    phase: OracleExceptionPhase,
    context: &mut Context,
) -> ExecutionError {
    let host_failure = error
        .as_opaque()
        .and_then(JsValue::as_object)
        .is_some_and(|object| {
            HOST_FAILURES.with(|errors| errors.borrow().iter().any(|actual| actual == &object))
        });
    let kind = if host_failure {
        None
    } else {
        exception_type(&error, context)
    };
    let (_, note) = observe_js_error(&error, context);
    ExecutionError {
        message: note,
        entry_syntax_rejection: false,
        javascript_exception: kind.map(|kind| (phase, kind)),
    }
}
fn exception_type(error: &JsError, context: &Context) -> Option<OracleExceptionType> {
    if let Some(native) = error.as_native() {
        return match &native.kind {
            JsNativeErrorKind::Error => Some(OracleExceptionType::Error),
            JsNativeErrorKind::Eval => Some(OracleExceptionType::Eval),
            JsNativeErrorKind::Range => Some(OracleExceptionType::Range),
            JsNativeErrorKind::Reference => Some(OracleExceptionType::Reference),
            JsNativeErrorKind::Syntax => Some(OracleExceptionType::Syntax),
            JsNativeErrorKind::Type => Some(OracleExceptionType::Type),
            JsNativeErrorKind::Uri => Some(OracleExceptionType::Uri),
            JsNativeErrorKind::Aggregate(_) => Some(OracleExceptionType::Aggregate),
            JsNativeErrorKind::RuntimeLimit => None,
            // Upstream's enum is non-exhaustive; future or fuzz-only failures
            // cannot acquire JavaScript exception provenance here.
            _ => None,
        };
    }
    let value = error.as_opaque()?;
    let Some(object) = value.as_object() else {
        return Some(OracleExceptionType::Primitive);
    };
    let Some(brand) = object.downcast_ref::<ErrorData>().map(|error| error.kind()) else {
        return Some(OracleExceptionType::UnclassifiedObject);
    };
    if brand == ErrorKind::RuntimeLimit {
        return None;
    }
    let Some(constructor) = raw_constructor(object) else {
        return Some(OracleExceptionType::UnclassifiedObject);
    };
    if let Some(kind) = intrinsic_type(&constructor, brand, context.intrinsics().constructors()) {
        return Some(kind);
    }
    let realms =
        HOST_REALMS.with(|store| store.borrow().realms.values().cloned().collect::<Vec<_>>());
    for realm in realms {
        let kind = match realm {
            HostRealmContext::EmbeddedRealm(realm) => {
                intrinsic_type(&constructor, brand, realm.intrinsics().constructors())
            }
            HostRealmContext::SeparateContext(realm) => {
                realm.try_borrow().ok().and_then(|context| {
                    intrinsic_type(&constructor, brand, context.intrinsics().constructors())
                })
            }
        };
        if kind.is_some() {
            return kind;
        }
    }
    Some(OracleExceptionType::UnclassifiedObject)
}
fn raw_constructor(mut object: JsObject) -> Option<JsObject> {
    let key = boa_engine::property::PropertyKey::String(js_string!("constructor"));
    let mut visited = Vec::new();
    loop {
        // No proxy trap, accessor or JS property lookup may run in reporting.
        if object.is::<Proxy>() || visited.iter().any(|previous| previous == &object) {
            return None;
        }
        let descriptor = object.borrow().properties().get(&key);
        if let Some(descriptor) = descriptor {
            return descriptor.value().and_then(JsValue::as_object);
        }
        visited.push(object.clone());
        object = object.prototype()?;
    }
}
fn intrinsic_type(
    constructor: &JsObject,
    brand: ErrorKind,
    constructors: &StandardConstructors,
) -> Option<OracleExceptionType> {
    use OracleExceptionType as Kind;
    let candidates = [
        (
            constructors.error().constructor(),
            ErrorKind::Error,
            Kind::Error,
        ),
        (
            constructors.eval_error().constructor(),
            ErrorKind::Eval,
            Kind::Eval,
        ),
        (
            constructors.range_error().constructor(),
            ErrorKind::Range,
            Kind::Range,
        ),
        (
            constructors.reference_error().constructor(),
            ErrorKind::Reference,
            Kind::Reference,
        ),
        (
            constructors.syntax_error().constructor(),
            ErrorKind::Syntax,
            Kind::Syntax,
        ),
        (
            constructors.type_error().constructor(),
            ErrorKind::Type,
            Kind::Type,
        ),
        (
            constructors.uri_error().constructor(),
            ErrorKind::Uri,
            Kind::Uri,
        ),
        (
            constructors.aggregate_error().constructor(),
            ErrorKind::Aggregate,
            Kind::Aggregate,
        ),
        (
            constructors.suppressed_error().constructor(),
            ErrorKind::Suppressed,
            Kind::Suppressed,
        ),
    ];
    candidates
        .into_iter()
        .find_map(|(actual, actual_brand, kind)| {
            (&actual == constructor && actual_brand == brand).then_some(kind)
        })
}

#[cfg(test)]
mod tests;
