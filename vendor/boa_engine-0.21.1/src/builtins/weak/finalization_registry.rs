//! Boa's implementation of ECMAScript's `FinalizationRegistry` builtin object.

use boa_gc::{Finalize, Trace};
use boa_macros::JsData;

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, weak::WeakHeldValue,
        weak::can_be_held_weakly,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
};

#[derive(Clone, Trace, Finalize)]
struct FinalizationRegistryCell {
    target: WeakHeldValue,
    holdings: JsValue,
    unregister_token: Option<WeakHeldValue>,
}

#[derive(Trace, Finalize, JsData)]
struct FinalizationRegistryData {
    cleanup_callback: JsObject,
    cells: Vec<FinalizationRegistryCell>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FinalizationRegistry;

impl IntrinsicObject for FinalizationRegistry {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                crate::symbol::JsSymbol::to_string_tag(),
                js_string!("FinalizationRegistry"),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::register, js_string!("register"), 2)
            .method(Self::unregister, js_string!("unregister"), 1)
            .build();
    }
}

impl BuiltInObject for FinalizationRegistry {
    const NAME: JsString = js_string!("FinalizationRegistry");

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for FinalizationRegistry {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::finalization_registry;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("FinalizationRegistry: cannot call constructor without `new`")
                .into());
        }

        let cleanup_callback = args
            .get_or_undefined(0)
            .as_callable()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("FinalizationRegistry: cleanup callback is not callable")
            })?;

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::finalization_registry,
            context,
        )?;
        let registry = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            FinalizationRegistryData {
                cleanup_callback,
                cells: Vec::new(),
            },
        );
        Ok(registry.into())
    }
}

impl FinalizationRegistry {
    pub(crate) fn register(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let mut registry = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<FinalizationRegistryData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "FinalizationRegistry.prototype method called with non-FinalizationRegistry object",
                )
            })?;
        let target = args.get_or_undefined(0).clone();
        let holdings = args.get_or_undefined(1).clone();
        let unregister_token = args.get_or_undefined(2).clone();

        let target = WeakHeldValue::from_value(&target).ok_or_else(|| {
            JsNativeError::typ().with_message(
                "FinalizationRegistry.prototype.register: target cannot be held weakly",
            )
        })?;

        if JsValue::same_value(&target.value(), &holdings) {
            return Err(JsNativeError::typ()
                .with_message("FinalizationRegistry.prototype.register: target and holdings must not be the same")
                .into());
        }

        let unregister_token = if unregister_token.is_undefined() {
            None
        } else {
            Some(WeakHeldValue::from_value(&unregister_token).ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "FinalizationRegistry.prototype.register: unregister token cannot be held weakly",
                )
            })?)
        };

        registry.cells.push(FinalizationRegistryCell {
            target,
            holdings,
            unregister_token,
        });

        Ok(JsValue::undefined())
    }

    pub(crate) fn unregister(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let mut registry = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<FinalizationRegistryData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "FinalizationRegistry.prototype method called with non-FinalizationRegistry object",
                )
            })?;
        let unregister_token = args.get_or_undefined(0).clone();

        if !can_be_held_weakly(&unregister_token) {
            return Err(JsNativeError::typ()
                .with_message(
                    "FinalizationRegistry.prototype.unregister: unregister token cannot be held weakly",
                )
                .into());
        }

        let before = registry.cells.len();
        registry.cells.retain(|cell| {
            !cell
                .unregister_token
                .as_ref()
                .is_some_and(|token| token.matches(&unregister_token))
        });
        Ok((registry.cells.len() != before).into())
    }
}
