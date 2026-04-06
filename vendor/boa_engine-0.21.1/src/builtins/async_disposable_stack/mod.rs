//! Boa's implementation of the ECMAScript `AsyncDisposableStack` object.

use crate::{
    Context, JsArgs, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        promise::PromiseCapability,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    environments::DisposableResource,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_macros::JsData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AsyncDisposableState {
    Pending,
    Disposed,
}

#[derive(Debug, Trace, Finalize, JsData)]
struct AsyncDisposableStackData {
    #[unsafe_ignore_trace]
    state: AsyncDisposableState,
    resources: Vec<DisposableResource>,
}

impl AsyncDisposableStackData {
    fn new() -> Self {
        Self {
            state: AsyncDisposableState::Pending,
            resources: Vec::new(),
        }
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct DisposableStackData;

#[derive(Debug, Trace, Finalize)]
struct DisposeAsyncState {
    resources: Vec<DisposableResource>,
    completion: Option<JsValue>,
    promise_capability: PromiseCapability,
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct AsyncDisposableStack;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct DisposableStack;

impl IntrinsicObject for DisposableStack {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }
}

impl BuiltInObject for DisposableStack {
    const NAME: JsString = js_string!("DisposableStack");
}

impl BuiltInConstructor for DisposableStack {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::disposable_stack;

    fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("DisposableStack: cannot call constructor without `new`")
                .into());
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::disposable_stack, context)?;
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            DisposableStackData,
        )
        .into())
    }
}

impl IntrinsicObject for AsyncDisposableStack {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        let disposed_getter = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_fn_ptr(Self::disposed),
        )
        .name(js_string!("get disposed"))
        .build();
        let dispose_async = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_fn_ptr(Self::dispose_async),
        )
        .name(js_string!("disposeAsync"))
        .length(0)
        .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("disposed"),
                Some(disposed_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .method(Self::use_, js_string!("use"), 1)
            .method(Self::adopt, js_string!("adopt"), 2)
            .method(Self::defer, js_string!("defer"), 1)
            .method(Self::move_, js_string!("move"), 0)
            .property(
                js_string!("disposeAsync"),
                dispose_async.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::async_dispose(),
                dispose_async,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }
}

impl BuiltInObject for AsyncDisposableStack {
    const NAME: JsString = js_string!("AsyncDisposableStack");
}

impl BuiltInConstructor for AsyncDisposableStack {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 9;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::async_disposable_stack;

    fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("AsyncDisposableStack: cannot call constructor without `new`")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::async_disposable_stack,
            context,
        )?;
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            AsyncDisposableStackData::new(),
        )
        .into())
    }
}

impl AsyncDisposableStack {
    fn type_error(method: &str) -> crate::JsError {
        JsNativeError::typ()
            .with_message(format!("{method}: called on incompatible value"))
            .into()
    }

    fn reference_error(method: &str) -> crate::JsError {
        JsNativeError::reference()
            .with_message(format!("{method}: stack is already disposed"))
            .into()
    }

    fn reject_promise(
        capability: &PromiseCapability,
        reason: JsValue,
        context: &mut Context,
    ) {
        capability
            .reject()
            .call(&JsValue::undefined(), &[reason], context)
            .expect("rejecting a promise capability cannot fail");
    }

    fn resolve_promise(
        capability: &PromiseCapability,
        value: JsValue,
        context: &mut Context,
    ) {
        capability
            .resolve()
            .call(&JsValue::undefined(), &[value], context)
            .expect("resolving a promise capability cannot fail");
    }

    fn create_intrinsic_stack_with_resources(
        resources: Vec<DisposableResource>,
        context: &mut Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .async_disposable_stack()
                .prototype(),
            AsyncDisposableStackData {
                state: AsyncDisposableState::Pending,
                resources,
            },
        )
    }

    fn continue_dispose(state: Gc<GcRefCell<DisposeAsyncState>>, context: &mut Context) {
        loop {
            let resource = state.borrow_mut().resources.pop();
            let Some(resource) = resource else {
                let (completion, capability) = {
                    let mut state = state.borrow_mut();
                    (state.completion.take(), state.promise_capability.clone())
                };
                if let Some(error) = completion {
                    Self::reject_promise(&capability, error, context);
                } else {
                    Self::resolve_promise(&capability, JsValue::undefined(), context);
                }
                return;
            };

            let result = match context.invoke_disposable_resource(&resource) {
                Ok(result) => result,
                Err(err) => {
                    let current = state.borrow_mut().completion.take();
                    let error = context.append_disposal_error(current, err);
                    state.borrow_mut().completion = Some(error);
                    continue;
                }
            };

            if !resource.r#async() {
                continue;
            }

            let wrapper = match crate::builtins::Promise::promise_resolve(
                &context.intrinsics().constructors().promise().constructor(),
                result,
                context,
            ) {
                Ok(wrapper) => wrapper,
                Err(err) => {
                    let current = state.borrow_mut().completion.take();
                    let error = context.append_disposal_error(current, err);
                    state.borrow_mut().completion = Some(error);
                    continue;
                }
            };

            let on_fulfilled = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_copy_closure_with_captures(
                    |_this, _args, state, context| {
                        AsyncDisposableStack::continue_dispose(state.clone(), context);
                        Ok(JsValue::undefined())
                    },
                    state.clone(),
                ),
            )
            .name("")
            .length(1)
            .build();
            let on_rejected = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_copy_closure_with_captures(
                    |_this, args, state, context| {
                        let current = state.borrow_mut().completion.take();
                        let error = context.append_disposal_error_value(
                            current,
                            args.get_or_undefined(0).clone(),
                        );
                        state.borrow_mut().completion = Some(error);
                        AsyncDisposableStack::continue_dispose(state.clone(), context);
                        Ok(JsValue::undefined())
                    },
                    state,
                ),
            )
            .name("")
            .length(1)
            .build();

            crate::builtins::Promise::perform_promise_then(
                &wrapper,
                Some(on_fulfilled),
                Some(on_rejected),
                None,
                context,
            );
            return;
        }
    }

    fn disposed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let object = this
            .as_object()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.disposed"))?;
        let stack = object
            .downcast_ref::<AsyncDisposableStackData>()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.disposed"))?;
        Ok(matches!(stack.state, AsyncDisposableState::Disposed).into())
    }

    fn use_(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0).clone();
        let object = this
            .as_object()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.use"))?;
        let mut stack = object
            .downcast_mut::<AsyncDisposableStackData>()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.use"))?;
        if matches!(stack.state, AsyncDisposableState::Disposed) {
            return Err(Self::reference_error("AsyncDisposableStack.prototype.use"));
        }

        let method = context.get_dispose_method(
            &value,
            true,
            "AsyncDisposableStack.prototype.use requires an object, null, or undefined",
            "AsyncDisposableStack.prototype.use requires a callable dispose method",
        )?;
        stack
            .resources
            .push(DisposableResource::from_value(value.clone(), method, true));
        Ok(value)
    }

    fn adopt(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0).clone();
        let on_dispose = args
            .get_or_undefined(1)
            .as_callable()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "AsyncDisposableStack.prototype.adopt requires a callable disposer",
                )
            })?;
        let object = this
            .as_object()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.adopt"))?;
        let mut stack = object
            .downcast_mut::<AsyncDisposableStackData>()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.adopt"))?;
        if matches!(stack.state, AsyncDisposableState::Disposed) {
            return Err(Self::reference_error("AsyncDisposableStack.prototype.adopt"));
        }

        stack.resources.push(DisposableResource::from_callback(
            Some(value.clone()),
            on_dispose,
            true,
        ));
        Ok(value)
    }

    fn defer(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let on_dispose = args
            .get_or_undefined(0)
            .as_callable()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "AsyncDisposableStack.prototype.defer requires a callable disposer",
                )
            })?;
        let object = this
            .as_object()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.defer"))?;
        let mut stack = object
            .downcast_mut::<AsyncDisposableStackData>()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.defer"))?;
        if matches!(stack.state, AsyncDisposableState::Disposed) {
            return Err(Self::reference_error("AsyncDisposableStack.prototype.defer"));
        }

        stack.resources.push(DisposableResource::from_callback(
            None,
            on_dispose,
            true,
        ));
        Ok(JsValue::undefined())
    }

    fn move_(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this
            .as_object()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.move"))?;
        let mut stack = object
            .downcast_mut::<AsyncDisposableStackData>()
            .ok_or_else(|| Self::type_error("AsyncDisposableStack.prototype.move"))?;
        if matches!(stack.state, AsyncDisposableState::Disposed) {
            return Err(Self::reference_error("AsyncDisposableStack.prototype.move"));
        }

        let resources = std::mem::take(&mut stack.resources);
        stack.state = AsyncDisposableState::Disposed;
        drop(stack);

        Ok(Self::create_intrinsic_stack_with_resources(resources, context).into())
    }

    fn dispose_async(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("intrinsic Promise constructor must create promise capabilities");
        let promise = promise_capability.promise().clone();

        let Some(object) = this.as_object() else {
            Self::reject_promise(
                &promise_capability,
                Self::type_error("AsyncDisposableStack.prototype.disposeAsync").to_opaque(context),
                context,
            );
            return Ok(promise.into());
        };

        let Some(mut stack) = object.downcast_mut::<AsyncDisposableStackData>() else {
            Self::reject_promise(
                &promise_capability,
                Self::type_error("AsyncDisposableStack.prototype.disposeAsync").to_opaque(context),
                context,
            );
            return Ok(promise.into());
        };

        if matches!(stack.state, AsyncDisposableState::Disposed) {
            Self::resolve_promise(&promise_capability, JsValue::undefined(), context);
            return Ok(promise.into());
        }

        let resources = std::mem::take(&mut stack.resources);
        stack.state = AsyncDisposableState::Disposed;
        drop(stack);

        let state = Gc::new(GcRefCell::new(DisposeAsyncState {
            resources,
            completion: None,
            promise_capability,
        }));
        Self::continue_dispose(state, context);

        Ok(promise.into())
    }
}
