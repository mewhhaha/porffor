//! https://github.com/tc39/proposal-error-stack-accessor/blob/4d0a932d27eb45dd44e416c33b26eb4d555f1e22/spec.emu
use super::*;
use crate::{
    NativeFunction,
    object::{FunctionObjectBuilder, builtins::JsFunction},
};

impl Error {
    pub(super) fn stack_name(kind: ErrorKind) -> JsString {
        match kind {
            ErrorKind::Error | ErrorKind::RuntimeLimit => js_string!("Error"),
            ErrorKind::Eval => js_string!("EvalError"),
            ErrorKind::Range => js_string!("RangeError"),
            ErrorKind::Reference => js_string!("ReferenceError"),
            ErrorKind::Syntax => js_string!("SyntaxError"),
            ErrorKind::Type => js_string!("TypeError"),
            ErrorKind::Uri => js_string!("URIError"),
            ErrorKind::Aggregate => js_string!("AggregateError"),
            ErrorKind::Suppressed => js_string!("SuppressedError"),
        }
    }

    pub(crate) fn capture_stack_message(object: &JsObject, message: &JsString) {
        let mut error = object.downcast_mut::<Self>().expect("native Error brand");
        if !message.is_empty() {
            error.stack = JsString::concat_array(&[
                Self::stack_name(error.tag).as_str(),
                js_str!(": "),
                message.as_str(),
            ]);
        }
    }

    pub(super) fn stack_getter(realm: &Realm) -> JsFunction {
        FunctionObjectBuilder::new(realm, NativeFunction::from_fn_ptr(Self::get_stack))
            .name(js_string!("get stack"))
            .length(0)
            .build()
    }

    pub(super) fn stack_setter(realm: &Realm) -> JsFunction {
        FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_copy_closure_with_captures(
                Self::set_stack,
                realm.intrinsics().constructors().error().prototype(),
            ),
        )
        .name(js_string!("set stack"))
        .length(1)
        .build()
    }

    fn get_stack(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Error stack accessor requires object receiver")
        })?;
        Ok(object
            .downcast_ref::<Self>()
            .map_or_else(JsValue::undefined, |error| error.stack.clone().into()))
    }

    fn set_stack(
        this: &JsValue,
        args: &[JsValue],
        home: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Error stack accessor requires object receiver")
        })?;
        let value = args.get_or_undefined(0);
        if !value.is_string() {
            return Err(JsNativeError::typ()
                .with_message("Error stack setter requires a String value")
                .into());
        }
        if JsObject::equals(&object, home) {
            return Err(JsNativeError::typ()
                .with_message("Error stack setter cannot write its home prototype")
                .into());
        }
        let key = js_string!("stack");
        if object.has_own_property(key.clone(), context)? {
            object.set(key, value.clone(), true, context)?;
        } else {
            object.create_data_property_or_throw(key, value.clone(), context)?;
        }
        Ok(JsValue::undefined())
    }
}
