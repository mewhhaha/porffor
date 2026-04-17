//! This module implements global `SuppressedError` object.

use crate::{
    Context, JsArgs, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
};

use super::Error;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SuppressedError;

impl IntrinsicObject for SuppressedError {
    fn init(realm: &Realm) {
        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .prototype(realm.intrinsics().constructors().error().constructor())
            .inherits(Some(realm.intrinsics().constructors().error().prototype()))
            .property(js_string!("name"), Self::NAME, attribute)
            .property(js_string!("message"), js_string!(), attribute)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for SuppressedError {
    const NAME: JsString = js_string!("SuppressedError");
}

impl BuiltInConstructor for SuppressedError {
    const CONSTRUCTOR_ARGUMENTS: usize = 3;
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::suppressed_error;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let new_target = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .suppressed_error()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::suppressed_error,
            context,
        )?;
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Error::with_caller_position(super::ErrorKind::Error, context),
        );

        let message = args.get_or_undefined(2);
        if !message.is_undefined() {
            let message = message.to_string(context)?;
            object.create_non_enumerable_data_property_or_throw(
                js_string!("message"),
                message,
                context,
            );
        }

        object.define_property_or_throw(
            js_string!("error"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(false)
                .writable(true)
                .value(args.get_or_undefined(0).clone())
                .build(),
            context,
        )?;

        object.define_property_or_throw(
            js_string!("suppressed"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(false)
                .writable(true)
                .value(args.get_or_undefined(1).clone())
                .build(),
            context,
        )?;

        Ok(object.into())
    }
}
