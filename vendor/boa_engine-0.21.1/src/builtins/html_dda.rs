//! Internal Annex B `[[IsHTMLDDA]]` host object support.

use crate::{
    Context, JsResult, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    native_function::NativeFunctionObject,
    object::JsObject,
    realm::Realm,
};

/// Realm-local callable object with the Annex B `[[IsHTMLDDA]]` internal slot.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IsHtmlDda;

impl IntrinsicObject for IsHtmlDda {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::call)
            .name(js_string!("IsHTMLDDA"))
            .length(0)
            .build();

        let object = Self::get(realm.intrinsics());
        object
            .downcast_mut::<NativeFunctionObject>()
            .expect("IsHTMLDDA intrinsic must be a native function object")
            .is_html_dda = true;
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().html_dda().into()
    }
}

impl IsHtmlDda {
    fn call(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        match args.first() {
            None => Ok(JsValue::null()),
            Some(first) if first.as_string().is_some_and(|text| text.is_empty()) => {
                Ok(JsValue::null())
            }
            _ => Ok(JsValue::undefined()),
        }
    }
}
