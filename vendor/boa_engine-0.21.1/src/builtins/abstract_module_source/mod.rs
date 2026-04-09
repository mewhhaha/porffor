//! Boa's implementation of the `%AbstractModuleSource%` intrinsic.

use boa_gc::{Finalize, Trace};
use boa_macros::JsData;

use crate::{
    Context, JsResult, JsString, JsValue,
    builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};

use super::BuiltInBuilder;

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct AbstractModuleSourceData {
    class_name: JsString,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct AbstractModuleSource;

impl IntrinsicObject for AbstractModuleSource {
    fn init(realm: &Realm) {
        let get_to_string_tag = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_fn_ptr(Self::get_to_string_tag),
        )
        .name(js_string!("get [Symbol.toStringTag]"))
        .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                JsSymbol::to_string_tag(),
                Some(get_to_string_tag),
                None,
                Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AbstractModuleSource {
    const NAME: JsString = js_string!("AbstractModuleSource");
}

impl BuiltInConstructor for AbstractModuleSource {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::abstract_module_source;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("AbstractModuleSource cannot be constructed")
            .into())
    }
}

impl AbstractModuleSource {
    fn get_to_string_tag(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let Some(object) = this.as_object() else {
            return Ok(JsValue::undefined());
        };

        let Some(module_source) = object.downcast_ref::<AbstractModuleSourceData>() else {
            return Ok(JsValue::undefined());
        };

        Ok(module_source.class_name.clone().into())
    }
}
