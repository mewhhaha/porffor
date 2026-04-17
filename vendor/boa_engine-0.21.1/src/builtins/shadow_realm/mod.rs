//! Boa's implementation of the ECMAScript `ShadowRealm` object.

use std::{cell::RefCell, mem::MaybeUninit};

use boa_gc::{Finalize, Gc, Trace};
use boa_macros::JsData;
use dynify::Dynify;
use num_traits::ToPrimitive;

use crate::{
    Context, JsArgs, JsError, JsNativeError, JsResult, JsString, JsValue, Source,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, eval::Eval,
        promise::Promise, promise::PromiseCapability,
    },
    bytecompiler::eval_declaration_instantiation_context,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    environments::DeclarativeEnvironment,
    js_string,
    module::{Module, ModuleRequest, Referrer},
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsFunction, JsObject, internal_methods::get_prototype_from_constructor},
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
    symbol::JsSymbol,
};
use boa_ast::scope::Scope;
use boa_parser::Parser;

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct ShadowRealmData {
    realm: Realm,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct WrappedFunctionState {
    target: JsObject,
    caller_realm: Realm,
    target_realm: Realm,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct ImportValueLoadState {
    caller_realm: Realm,
    eval_realm: Realm,
    capability: PromiseCapability,
    request: ModuleRequest,
    export_name: JsString,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct ImportValueLinkState {
    caller_realm: Realm,
    eval_realm: Realm,
    capability: PromiseCapability,
    module: Module,
    export_name: JsString,
    on_rejected: JsFunction,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct ImportValueFulfillState {
    caller_realm: Realm,
    eval_realm: Realm,
    capability: PromiseCapability,
    module: Module,
    export_name: JsString,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct ShadowRealm;

impl IntrinsicObject for ShadowRealm {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::evaluate, js_string!("evaluate"), 1)
            .method(Self::import_value, js_string!("importValue"), 2)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ShadowRealm {
    const NAME: JsString = js_string!("ShadowRealm");
}

impl BuiltInConstructor for ShadowRealm {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::shadow_realm;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm constructor must be called with `new`")
                .into());
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::shadow_realm, context)?;
        let realm = context.create_realm()?;
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ShadowRealmData { realm },
        );
        Ok(object.into())
    }
}

impl ShadowRealm {
    fn incompatible_receiver(method: &str) -> JsError {
        JsNativeError::typ()
            .with_message(format!("ShadowRealm.prototype.{method} called on incompatible value"))
            .into()
    }

    fn wrapped_boundary_error() -> JsError {
        JsNativeError::typ()
            .with_message("ShadowRealm wrapped value must be primitive or callable")
            .into()
    }

    fn get_shadow_realm(this: &JsValue, method: &str) -> JsResult<Realm> {
        let object = this
            .as_object()
            .ok_or_else(|| Self::incompatible_receiver(method))?;
        let shadow_realm = object
            .downcast_ref::<ShadowRealmData>()
            .ok_or_else(|| Self::incompatible_receiver(method))?;
        Ok(shadow_realm.realm.clone())
    }

    fn switch_execution_realm(
        context: &mut Context,
        realm: &Realm,
    ) -> (Realm, Gc<DeclarativeEnvironment>) {
        let old_global = context.vm.environments.global().clone();
        let mut old_realm = realm.clone();
        context.swap_realm(&mut old_realm);
        context
            .vm
            .environments
            .replace_global(context.realm().environment().clone());
        (old_realm, old_global)
    }

    fn restore_execution_realm(
        context: &mut Context,
        old_realm: Realm,
        old_global: Gc<DeclarativeEnvironment>,
    ) {
        let mut old_realm = old_realm;
        context.swap_realm(&mut old_realm);
        context.vm.environments.replace_global(old_global);
    }

    fn wrap_value(caller_realm: &Realm, value: JsValue, context: &mut Context) -> JsResult<JsValue> {
        let Some(object) = value.as_object() else {
            return Ok(value);
        };

        if !object.is_callable() {
            return Err(Self::wrapped_boundary_error());
        }

        Self::wrap_callable(caller_realm, object.clone(), context).map(Into::into)
    }

    fn wrap_callable(
        caller_realm: &Realm,
        target: JsObject,
        context: &mut Context,
    ) -> JsResult<JsFunction> {
        let target_realm = target
            .get_function_realm(context)
            .map_err(|_| Self::wrapped_boundary_error())?;

        let length = Self::copy_length(&target, context).map_err(|_| Self::wrapped_boundary_error())?;
        let name = Self::copy_name(&target, context).map_err(|_| Self::wrapped_boundary_error())?;

        let wrapped = FunctionObjectBuilder::new(
            caller_realm,
            NativeFunction::from_copy_closure_with_captures(
                Self::call_wrapped_function,
                WrappedFunctionState {
                    target,
                    caller_realm: caller_realm.clone(),
                    target_realm,
                },
            ),
        )
        .name(js_string!())
        .length(0)
        .constructor(false)
        .build();

        wrapped.define_property_or_throw(
            js_string!("length"),
            PropertyDescriptor::builder()
                .value(length)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context,
        )?;
        wrapped.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(name)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context,
        )?;

        Ok(wrapped)
    }

    fn copy_length(target: &JsObject, context: &mut Context) -> JsResult<JsValue> {
        let has_length = target.has_own_property(js_string!("length"), context)?;
        if !has_length {
            return Ok(0.into());
        }

        let target_len = target.get(js_string!("length"), context)?;
        let Some(target_len) = target_len.as_number() else {
            return Ok(0.into());
        };

        if target_len.is_infinite() {
            return Ok(if target_len.is_sign_positive() {
                f64::INFINITY.into()
            } else {
                0.into()
            });
        }

        let integer = target_len
            .trunc()
            .to_i64()
            .unwrap_or_default()
            .max(0);
        Ok(integer.into())
    }

    fn copy_name(target: &JsObject, context: &mut Context) -> JsResult<JsValue> {
        let target_name = target.get(js_string!("name"), context)?;
        Ok(target_name
            .as_string()
            .map_or_else(|| js_string!().into(), Into::into))
    }

    fn call_wrapped_function(
        _this: &JsValue,
        args: &[JsValue],
        state: &WrappedFunctionState,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut wrapped_args = Vec::with_capacity(args.len());
        for arg in args {
            let value = if let Some(object) = arg.as_object() {
                if !object.is_callable() {
                    return Err(Self::wrapped_boundary_error());
                }
                Self::wrap_callable(&state.target_realm, object.clone(), context)?.into()
            } else {
                arg.clone()
            };
            wrapped_args.push(value);
        }

        let result = state
            .target
            .call(&JsValue::undefined(), &wrapped_args, context)
            .map_err(|_| Self::wrapped_boundary_error())?;
        Self::wrap_value(&state.caller_realm, result, context)
    }

    fn evaluate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let eval_realm = Self::get_shadow_realm(this, "evaluate")?;
        let caller_realm = context.realm().clone();
        let source_text = args.get_or_undefined(0);
        let Some(source_text) = source_text.as_string() else {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm.prototype.evaluate requires a string")
                .into());
        };
        let (old_realm, old_global) = Self::switch_execution_realm(context, &eval_realm);
        let result = Eval::perform_eval(&source_text.clone().into(), false, None, false, context);
        Self::restore_execution_realm(context, old_realm, old_global);

        match result {
            Ok(value) => Self::wrap_value(&caller_realm, value, context),
            Err(_) => {
                let (old_realm, old_global) = Self::switch_execution_realm(context, &eval_realm);
                let validation = Self::validate_eval_source(&source_text, context);
                Self::restore_execution_realm(context, old_realm, old_global);

                match validation {
                    Ok(()) => Err(Self::wrapped_boundary_error()),
                    Err(err) => Err(err.inject_realm(caller_realm)),
                }
            }
        }
    }

    fn validate_eval_source(source_text: &JsString, context: &mut Context) -> JsResult<()> {
        let source_text = source_text.to_vec();
        let source = Source::from_utf16(&source_text);
        let mut parser = Parser::new(source);
        parser.set_identifier(context.next_parser_identifier());
        let (mut body, _) = parser.parse_eval(false, context.interner_mut())?;

        let strict = body.strict();
        let variable_scope = context.realm().scope().clone();
        let lexical_scope = Scope::new(context.realm().scope().clone(), strict);
        let mut annex_b_function_names = Vec::new();

        eval_declaration_instantiation_context(
            &mut annex_b_function_names,
            &body,
            strict,
            if strict {
                &lexical_scope
            } else {
                &variable_scope
            },
            &lexical_scope,
            context,
        )?;

        body.analyze_scope_eval(
            strict,
            &variable_scope,
            &lexical_scope,
            &annex_b_function_names,
            context.interner(),
        )
        .map_err(|err| JsNativeError::syntax().with_message(err))?;

        Ok(())
    }

    fn import_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let eval_realm = Self::get_shadow_realm(this, "importValue")?;
        let caller_realm = context.realm().clone();
        let specifier = args.get_or_undefined(0).to_string(context)?;
        let Some(export_name) = args.get_or_undefined(1).as_string() else {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm.prototype.importValue requires a string export name")
                .into());
        };

        let capability = PromiseCapability::new(
            &caller_realm.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("intrinsic promise constructor must create capabilities");
        let promise = capability.promise().clone();
        let request = ModuleRequest::new(specifier);
        let load_state = ImportValueLoadState {
            caller_realm: caller_realm.clone(),
            eval_realm,
            capability: capability.clone(),
            request: request.clone(),
            export_name,
        };

        context.enqueue_job(
            crate::job::NativeAsyncJob::with_realm(
                async move |context| {
                    Self::load_import_value(load_state, context).await;
                    Ok(JsValue::undefined())
                },
                caller_realm,
            )
            .into(),
        );

        Ok(promise.into())
    }

    async fn load_import_value(
        state: ImportValueLoadState,
        context: &RefCell<&mut Context>,
    ) {
        let loader = context.borrow().module_loader();

        let (old_realm, old_global) = {
            let mut context = context.borrow_mut();
            Self::switch_execution_realm(&mut context, &state.eval_realm)
        };

        let future = loader.load_imported_module(
            Referrer::Realm(state.eval_realm.clone()),
            state.request.clone(),
            context,
        );
        let mut stack = [MaybeUninit::<u8>::uninit(); 16];
        let mut heap = Vec::<MaybeUninit<u8>>::new();
        let loaded_module: JsResult<Module> = future.init2(&mut stack, &mut heap).await;

        {
            let mut context = context.borrow_mut();
            Self::restore_execution_realm(&mut context, old_realm, old_global);
        }

        let module = match loaded_module {
            Ok(module) => {
                let mut loaded_modules = state.eval_realm.loaded_modules().borrow_mut();
                let entry = loaded_modules
                    .entry(state.request.clone())
                    .or_insert_with(|| module.clone());
                debug_assert_eq!(&module, entry);
                module
            }
            Err(_) => {
                let mut context = context.borrow_mut();
                Self::reject_type_error(&state.capability, &mut context);
                return;
            }
        };

        let load = {
            let mut context = context.borrow_mut();
            let (old_realm, old_global) =
                Self::switch_execution_realm(&mut context, &state.eval_realm);
            let promise = module.load(&mut context);
            Self::restore_execution_realm(&mut context, old_realm, old_global);
            promise
        };

        let on_rejected = FunctionObjectBuilder::new(
            &state.caller_realm,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, capability: &PromiseCapability, context| {
                    ShadowRealm::reject_type_error(capability, context);
                    Ok(JsValue::undefined())
                },
                state.capability.clone(),
            ),
        )
        .build();

        let link_and_evaluate = FunctionObjectBuilder::new(
            &state.caller_realm,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, state: &ImportValueLinkState, context| {
                    let (old_realm, old_global) =
                        ShadowRealm::switch_execution_realm(context, &state.eval_realm);
                    let linked = state.module.link(context);
                    ShadowRealm::restore_execution_realm(context, old_realm, old_global);

                    if linked.is_err() {
                        ShadowRealm::reject_type_error(&state.capability, context);
                        return Ok(JsValue::undefined());
                    }

                    let evaluate = {
                        let (old_realm, old_global) =
                            ShadowRealm::switch_execution_realm(context, &state.eval_realm);
                        let promise = state.module.evaluate(context);
                        ShadowRealm::restore_execution_realm(context, old_realm, old_global);
                        promise
                    };

                    let on_fulfilled = FunctionObjectBuilder::new(
                        &state.caller_realm,
                        NativeFunction::from_copy_closure_with_captures(
                            |_, _, state: &ImportValueFulfillState, context| {
                                let (old_realm, old_global) =
                                    ShadowRealm::switch_execution_realm(context, &state.eval_realm);
                                let namespace = state.module.namespace(context);
                                let export = match namespace
                                    .has_own_property(state.export_name.clone(), context)
                                {
                                    Ok(true) => namespace.get(state.export_name.clone(), context).ok(),
                                    Ok(false) | Err(_) => None,
                                };
                                ShadowRealm::restore_execution_realm(context, old_realm, old_global);

                                let Some(export) = export else {
                                    ShadowRealm::reject_type_error(&state.capability, context);
                                    return Ok(JsValue::undefined());
                                };

                                match ShadowRealm::wrap_value(&state.caller_realm, export, context) {
                                    Ok(export) => {
                                        state
                                            .capability
                                            .resolve()
                                            .call(&JsValue::undefined(), &[export], context)
                                            .expect("default resolve must not throw");
                                    }
                                    Err(_) => ShadowRealm::reject_type_error(&state.capability, context),
                                }

                                Ok(JsValue::undefined())
                            },
                            ImportValueFulfillState {
                                caller_realm: state.caller_realm.clone(),
                                eval_realm: state.eval_realm.clone(),
                                capability: state.capability.clone(),
                                module: state.module.clone(),
                                export_name: state.export_name.clone(),
                            },
                        ),
                    )
                    .build();

                    Promise::perform_promise_then(
                        &evaluate,
                        Some(on_fulfilled),
                        Some(state.on_rejected.clone()),
                        None,
                        context,
                    );

                    Ok(JsValue::undefined())
                },
                ImportValueLinkState {
                    caller_realm: state.caller_realm.clone(),
                    eval_realm: state.eval_realm.clone(),
                    capability: state.capability.clone(),
                    module,
                    export_name: state.export_name.clone(),
                    on_rejected: on_rejected.clone(),
                },
            ),
        )
        .build();

        Promise::perform_promise_then(
            &load,
            Some(link_and_evaluate),
            Some(on_rejected),
            None,
            &mut context.borrow_mut(),
        );
    }

    fn reject_type_error(capability: &PromiseCapability, context: &mut Context) {
        let err = JsNativeError::typ()
            .with_message("ShadowRealm import failed")
            .to_opaque(context);
        capability
            .reject()
            .call(&JsValue::undefined(), &[err.into()], context)
            .expect("default reject must not throw");
    }
}
