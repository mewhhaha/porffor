//! Boa's implementation of ECMAScript's `IteratorRecord` and iterator prototype objects.

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::{
        Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, Promise,
        promise::{PromiseCapability, if_abrupt_reject_promise},
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{
        FunctionObjectBuilder, JsFunction, JsObject,
        internal_methods::{InternalMethodPropertyContext, get_prototype_from_constructor},
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::IntegerOrInfinity,
};
use boa_gc::{Finalize, Trace};

mod async_from_sync_iterator;
pub(crate) use async_from_sync_iterator::AsyncFromSyncIterator;

/// `IfAbruptCloseIterator ( value, iteratorRecord )`
///
/// `IfAbruptCloseIterator` is a shorthand for a sequence of algorithm steps that use an `Iterator`
/// Record.
///
/// More information:
///  - [ECMA reference][spec]
///
///  [spec]: https://tc39.es/ecma262/#sec-ifabruptcloseiterator
macro_rules! if_abrupt_close_iterator {
    ($value:expr, $iterator_record:expr, $context:expr) => {
        match $value {
            // 1. If value is an abrupt completion, return ? IteratorClose(iteratorRecord, value).
            Err(err) => return $iterator_record.close(Err(err), $context),
            // 2. Else if value is a Completion Record, set value to value.
            Ok(value) => value,
        }
    };
}

// Export macro to crate level
pub(crate) use if_abrupt_close_iterator;

use super::OrdinaryObject;

/// The built-in iterator prototypes.
#[derive(Debug, Trace, Finalize)]
pub struct IteratorPrototypes {
    /// The `IteratorPrototype` object.
    iterator: JsObject,

    /// The `%IteratorHelperPrototype%` object.
    iterator_helper: JsObject,

    /// The `%WrapForValidIteratorPrototype%` object.
    wrap_for_valid_iterator: JsObject,

    /// The `AsyncIteratorPrototype` object.
    async_iterator: JsObject,

    /// The `AsyncFromSyncIteratorPrototype` prototype object.
    async_from_sync_iterator: JsObject,

    /// The `ArrayIteratorPrototype` prototype object.
    array: JsObject,

    /// The `SetIteratorPrototype` prototype object.
    set: JsObject,

    /// The `StringIteratorPrototype` prototype object.
    string: JsObject,

    /// The `RegExpStringIteratorPrototype` prototype object.
    regexp_string: JsObject,

    /// The `MapIteratorPrototype` prototype object.
    map: JsObject,

    /// The `ForInIteratorPrototype` prototype object.
    for_in: JsObject,

    /// The `%SegmentIteratorPrototype%` prototype object.
    #[cfg(feature = "intl")]
    segment: JsObject,
}

impl IteratorPrototypes {
    #[must_use]
    pub fn new(iterator: JsObject) -> Self {
        Self {
            iterator,
            iterator_helper: JsObject::with_null_proto(),
            wrap_for_valid_iterator: JsObject::with_null_proto(),
            async_iterator: JsObject::with_null_proto(),
            async_from_sync_iterator: JsObject::with_null_proto(),
            array: JsObject::with_null_proto(),
            set: JsObject::with_null_proto(),
            string: JsObject::with_null_proto(),
            regexp_string: JsObject::with_null_proto(),
            map: JsObject::with_null_proto(),
            for_in: JsObject::with_null_proto(),
            #[cfg(feature = "intl")]
            segment: JsObject::with_null_proto(),
        }
    }

    /// Returns the `ArrayIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn array(&self) -> JsObject {
        self.array.clone()
    }

    /// Returns the `IteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn iterator(&self) -> JsObject {
        self.iterator.clone()
    }

    /// Returns the `%IteratorHelperPrototype%` object.
    #[inline]
    #[must_use]
    pub fn iterator_helper(&self) -> JsObject {
        self.iterator_helper.clone()
    }

    /// Returns the `%WrapForValidIteratorPrototype%` object.
    #[inline]
    #[must_use]
    pub fn wrap_for_valid_iterator(&self) -> JsObject {
        self.wrap_for_valid_iterator.clone()
    }

    /// Returns the `AsyncIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn async_iterator(&self) -> JsObject {
        self.async_iterator.clone()
    }

    /// Returns the `AsyncFromSyncIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn async_from_sync_iterator(&self) -> JsObject {
        self.async_from_sync_iterator.clone()
    }

    /// Returns the `SetIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn set(&self) -> JsObject {
        self.set.clone()
    }

    /// Returns the `StringIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn string(&self) -> JsObject {
        self.string.clone()
    }

    /// Returns the `RegExpStringIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn regexp_string(&self) -> JsObject {
        self.regexp_string.clone()
    }

    /// Returns the `MapIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn map(&self) -> JsObject {
        self.map.clone()
    }

    /// Returns the `ForInIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn for_in(&self) -> JsObject {
        self.for_in.clone()
    }

    /// Returns the `%SegmentIteratorPrototype%` object.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub fn segment(&self) -> JsObject {
        self.segment.clone()
    }
}

/// `%Iterator%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator
pub(crate) struct Iterator;

impl IntrinsicObject for Iterator {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::iterator, JsSymbol::iterator(), 0)
            .method(Self::dispose, JsSymbol::dispose(), 0)
            .method(Self::map, js_string!("map"), 1)
            .method(Self::filter, js_string!("filter"), 1)
            .method(Self::take, js_string!("take"), 1)
            .method(Self::drop, js_string!("drop"), 1)
            .method(Self::flat_map, js_string!("flatMap"), 1)
            .method(Self::reduce, js_string!("reduce"), 1)
            .method(Self::to_array, js_string!("toArray"), 0)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::some, js_string!("some"), 1)
            .method(Self::every, js_string!("every"), 1)
            .method(Self::find, js_string!("find"), 1)
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::concat, js_string!("concat"), 0)
            .static_method(Self::zip, js_string!("zip"), 1)
            .static_method(Self::zip_keyed, js_string!("zipKeyed"), 1)
            .build();

        Self::install_prototype_accessors(realm);
        IteratorHelperPrototype::init(realm);
        WrapForValidIteratorPrototype::init(realm);
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Iterator {
    const NAME: JsString = js_string!("Iterator");
}

impl BuiltInConstructor for Iterator {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 13;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 4;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::iterator;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined()
            || new_target
                .as_object()
                .is_some_and(|new_target| JsObject::equals(&new_target, &Self::get(context.intrinsics())))
        {
            return Err(JsNativeError::typ()
                .with_message("Iterator is abstract")
                .into());
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::iterator, context)?;
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        )
        .into())
    }
}

/// `%AsyncIteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-asynciteratorprototype
pub(crate) struct AsyncIterator;

impl IntrinsicObject for AsyncIterator {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(|v, _, _| Ok(v.clone()), JsSymbol::async_iterator(), 0)
            .static_method(Self::async_dispose, JsSymbol::async_dispose(), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().async_iterator()
    }
}

impl AsyncIterator {
    fn async_dispose(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("intrinsic Promise constructor must create promise capabilities");

        let r#return = this.get_method(js_string!("return"), context);
        let r#return = if_abrupt_reject_promise!(r#return, promise_capability, context);

        if let Some(r#return) = r#return {
            let result = r#return.call(this, &[JsValue::undefined()], context);
            let result = if_abrupt_reject_promise!(result, promise_capability, context);
            let result_wrapper = Promise::promise_resolve(
                &context.intrinsics().constructors().promise().constructor(),
                result,
                context,
            );
            let result_wrapper =
                if_abrupt_reject_promise!(result_wrapper, promise_capability, context);

            let on_fulfilled = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
            )
            .name("")
            .length(1)
            .build();
            Promise::perform_promise_then(
                &result_wrapper,
                Some(on_fulfilled),
                None,
                Some(promise_capability.clone()),
                context,
            );
        } else {
            promise_capability
                .resolve()
                .call(&JsValue::undefined(), &[JsValue::undefined()], context)
                .expect("resolving a promise capability cannot fail");
        }

        Ok(promise_capability.promise().clone().into())
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
enum ZipMode {
    Shortest,
    Longest,
    Strict,
}

#[derive(Debug, Clone, Trace, Finalize)]
enum IteratorLimit {
    Finite(u64),
    Infinity,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct ConcatSource {
    item: JsValue,
    method: JsObject,
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct WrapForValidIterator {
    iterator: JsObject,
    next_method: JsValue,
}

#[derive(Debug, Clone, Trace, Finalize)]
enum IteratorHelperKind {
    Map {
        iterated: IteratorRecord,
        mapper: JsObject,
        counter: u64,
    },
    Filter {
        iterated: IteratorRecord,
        predicate: JsObject,
        counter: u64,
    },
    Take {
        iterated: IteratorRecord,
        remaining: IteratorLimit,
        exhausted: bool,
    },
    Drop {
        iterated: IteratorRecord,
        remaining: IteratorLimit,
        advanced: bool,
    },
    FlatMap {
        iterated: IteratorRecord,
        mapper: JsObject,
        counter: u64,
        inner: Option<IteratorRecord>,
    },
    Concat {
        sources: Vec<ConcatSource>,
        current: Option<IteratorRecord>,
    },
    Zip {
        iterators: Vec<IteratorRecord>,
        finished: Vec<bool>,
        padding: Vec<JsValue>,
        #[unsafe_ignore_trace]
        mode: ZipMode,
    },
    ZipKeyed {
        keys: Vec<JsValue>,
        iterators: Vec<IteratorRecord>,
        finished: Vec<bool>,
        padding: Vec<JsValue>,
        #[unsafe_ignore_trace]
        mode: ZipMode,
    },
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct IteratorHelperObject {
    executing: bool,
    completed: bool,
    started: bool,
    kind: IteratorHelperKind,
}

struct IteratorHelperPrototype;

impl IntrinsicObject for IteratorHelperPrototype {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::return_, js_string!("return"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator_helper()
    }
}

struct WrapForValidIteratorPrototype;

impl IntrinsicObject for WrapForValidIteratorPrototype {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::return_, js_string!("return"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics
            .objects()
            .iterator_prototypes()
            .wrap_for_valid_iterator()
    }
}

fn iterator_done_result(context: &mut Context) -> JsValue {
    create_iter_result_object(JsValue::undefined(), true, context)
}

fn get_iterator_direct(value: &JsValue, context: &mut Context) -> JsResult<IteratorRecord> {
    let object = value.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("iterator helper receiver must be an object")
    })?;
    let next_method = object.get(js_string!("next"), context)?;
    Ok(IteratorRecord::new(object.clone(), next_method))
}

fn iterator_return(record: &IteratorRecord, context: &mut Context) -> JsResult<JsValue> {
    let Some(method) = record.iterator().get_method(js_string!("return"), context)? else {
        return Ok(iterator_done_result(context));
    };
    let result = method.call(&record.iterator().clone().into(), &[], context)?;
    IteratorResult::from_value(result)?;
    Ok(iterator_done_result(context))
}

fn iterator_return_result(record: &IteratorRecord, context: &mut Context) -> JsResult<JsValue> {
    let Some(method) = record.iterator().get_method(js_string!("return"), context)? else {
        return Ok(iterator_done_result(context));
    };
    let result = method.call(&record.iterator().clone().into(), &[], context)?;
    IteratorResult::from_value(result.clone())?;
    Ok(result)
}

fn close_if_possible(value: &JsValue, context: &mut Context) -> JsResult<()> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    let Some(method) = object.get_method(js_string!("return"), context)? else {
        return Ok(());
    };
    let result = method.call(&object.clone().into(), &[], context)?;
    IteratorResult::from_value(result)?;
    Ok(())
}

fn close_iterators_reverse(
    iterators: &[IteratorRecord],
    finished: &[bool],
    completion: JsResult<JsValue>,
    skip_index: Option<usize>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let mut completion = completion;
    for (index, iterator) in iterators.iter().enumerate().rev() {
        if skip_index == Some(index) || finished[index] {
            continue;
        }
        completion = iterator.close(completion, context);
    }
    completion
}

fn same_iterator_prototype(value: &JsValue, context: &mut Context) -> JsResult<bool> {
    let Some(mut current) = value.as_object() else {
        return Ok(false);
    };
    let iterator_prototype = context.intrinsics().objects().iterator_prototypes().iterator();

    while let Some(parent) =
        current.__get_prototype_of__(&mut InternalMethodPropertyContext::new(context))?
    {
        if JsObject::equals(&parent, &iterator_prototype) {
            return Ok(true);
        }
        current = parent;
    }

    Ok(false)
}

fn get_iterator_flattenable(
    value: &JsValue,
    allow_primitive_strings: bool,
    context: &mut Context,
) -> JsResult<IteratorRecord> {
    if !value.is_object() {
        let is_string = value.as_string().is_some();
        if !(allow_primitive_strings && is_string) {
            return Err(JsNativeError::typ()
                .with_message("Iterator helper requires an iterable or iterator object")
                .into());
        }
    }

    match value.get_method(JsSymbol::iterator(), context)? {
        Some(method) => value.get_iterator_from_method(&method, context),
        None => get_iterator_direct(value, context),
    }
}

fn to_options_object(value: &JsValue) -> JsResult<Option<JsObject>> {
    match value.as_object() {
        Some(object) => Ok(Some(object.clone())),
        None if value.is_undefined() => Ok(None),
        None => Err(JsNativeError::typ()
            .with_message("options must be an object")
            .into()),
    }
}

fn to_positive_integer_or_infinity(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<IteratorLimit> {
    let number = value.to_number(context)?;
    if number.is_nan() {
        return Err(JsNativeError::range()
            .with_message("limit must be a non-negative integer")
            .into());
    }
    match IntegerOrInfinity::from(number) {
        IntegerOrInfinity::PositiveInfinity => Ok(IteratorLimit::Infinity),
        IntegerOrInfinity::Integer(integer) if integer >= 0 => Ok(IteratorLimit::Finite(
            u64::try_from(integer).expect("non-negative integer fits in u64"),
        )),
        _ => Err(JsNativeError::range()
            .with_message("limit must be a non-negative integer")
            .into()),
    }
}

fn create_iterator_helper(kind: IteratorHelperKind, context: &Context) -> JsObject {
    JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .objects()
            .iterator_prototypes()
            .iterator_helper(),
        IteratorHelperObject {
            executing: false,
            completed: false,
            started: false,
            kind,
        },
    )
}

fn create_wrap_for_valid_iterator(
    iterator: JsObject,
    next_method: JsValue,
    context: &Context,
) -> JsObject {
    JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .objects()
            .iterator_prototypes()
            .wrap_for_valid_iterator(),
        WrapForValidIterator {
            iterator,
            next_method,
        },
    )
}

impl WrapForValidIteratorPrototype {
    fn get_wrap(this: &JsValue, method: &str) -> JsResult<JsObject> {
        this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message(format!("{method} called on incompatible value"))
                .into()
        })
    }

    fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = Self::get_wrap(this, "%WrapForValidIteratorPrototype%.next")?;
        let wrap = object.downcast_ref::<WrapForValidIterator>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("%WrapForValidIteratorPrototype%.next requires internal slot")
        })?;
        let result = wrap
            .next_method
            .call(&wrap.iterator.clone().into(), &[], context)?;
        IteratorResult::from_value(result.clone())?;
        Ok(result)
    }

    fn return_(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = Self::get_wrap(this, "%WrapForValidIteratorPrototype%.return")?;
        let wrap = object.downcast_ref::<WrapForValidIterator>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("%WrapForValidIteratorPrototype%.return requires internal slot")
        })?;
        let record = IteratorRecord::new(wrap.iterator.clone(), wrap.next_method.clone());
        iterator_return_result(&record, context)
    }
}

impl IteratorHelperObject {
    fn advance(&mut self, context: &mut Context) -> JsResult<JsValue> {
        match &mut self.kind {
            IteratorHelperKind::Map {
                iterated,
                mapper,
                counter,
            } => {
                let Some(value) = iterated.step_value(context)? else {
                    return Ok(iterator_done_result(context));
                };
                let mapped = mapper.call(
                    &JsValue::undefined(),
                    &[value, JsValue::new(*counter as i32)],
                    context,
                );
                *counter += 1;
                match mapped {
                    Ok(value) => Ok(create_iter_result_object(value, false, context)),
                    Err(error) => iterated.close(Err(error), context),
                }
            }
            IteratorHelperKind::Filter {
                iterated,
                predicate,
                counter,
            } => loop {
                let Some(value) = iterated.step_value(context)? else {
                    return Ok(iterator_done_result(context));
                };
                let keep = predicate.call(
                    &JsValue::undefined(),
                    &[value.clone(), JsValue::new(*counter as i32)],
                    context,
                );
                *counter += 1;
                match keep {
                    Ok(keep) if keep.to_boolean() => {
                        return Ok(create_iter_result_object(value, false, context));
                    }
                    Ok(_) => {}
                    Err(error) => return iterated.close(Err(error), context),
                }
            },
            IteratorHelperKind::Take {
                iterated,
                remaining,
                exhausted,
            } => {
                if *exhausted {
                    return Ok(iterator_done_result(context));
                }
                if matches!(remaining, IteratorLimit::Finite(0)) {
                    *exhausted = true;
                    return iterator_return(iterated, context);
                }
                let result = match iterated.step_value(context)? {
                    Some(value) => create_iter_result_object(value, false, context),
                    None => {
                        *exhausted = true;
                        iterator_done_result(context)
                    }
                };
                if let IteratorLimit::Finite(remaining) = remaining {
                    *remaining -= 1;
                }
                Ok(result)
            }
            IteratorHelperKind::Drop {
                iterated,
                remaining,
                advanced,
            } => {
                if !*advanced {
                    *advanced = true;
                    loop {
                        match remaining {
                            IteratorLimit::Finite(0) => break,
                            IteratorLimit::Finite(remaining_count) => {
                                if iterated.step_value(context)?.is_none() {
                                    return Ok(iterator_done_result(context));
                                }
                                *remaining_count -= 1;
                            }
                            IteratorLimit::Infinity => {
                                if iterated.step_value(context)?.is_none() {
                                    return Ok(iterator_done_result(context));
                                }
                            }
                        }
                    }
                }

                match iterated.step_value(context)? {
                    Some(value) => Ok(create_iter_result_object(value, false, context)),
                    None => Ok(iterator_done_result(context)),
                }
            }
            IteratorHelperKind::FlatMap {
                iterated,
                mapper,
                counter,
                inner,
            } => loop {
                if let Some(inner_record) = inner {
                    if let Some(value) = inner_record.step_value(context)? {
                        return Ok(create_iter_result_object(value, false, context));
                    }
                    *inner = None;
                }

                let Some(value) = iterated.step_value(context)? else {
                    return Ok(iterator_done_result(context));
                };

                let mapped = mapper.call(
                    &JsValue::undefined(),
                    &[value, JsValue::new(*counter as i32)],
                    context,
                );
                *counter += 1;
                let mapped = match mapped {
                    Ok(mapped) => mapped,
                    Err(error) => return iterated.close(Err(error), context),
                };

                match get_iterator_flattenable(&mapped, false, context) {
                    Ok(record) => *inner = Some(record),
                    Err(error) => return iterated.close(Err(error), context),
                }
            },
            IteratorHelperKind::Concat { sources, current } => loop {
                if current.is_none() {
                    if sources.is_empty() {
                        return Ok(iterator_done_result(context));
                    }
                    let source = sources.remove(0);
                    let iterator = source.method.call(&source.item, &[], context)?;
                    let iterator_object = iterator.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("Iterator.concat iterable produced a non-object iterator")
                    })?;
                    *current = Some(get_iterator_direct(&iterator_object.clone().into(), context)?);
                }

                let Some(current_record) = current.as_mut() else {
                    continue;
                };
                if let Some(value) = current_record.step_value(context)? {
                    return Ok(create_iter_result_object(value, false, context));
                }
                *current = None;
            },
            IteratorHelperKind::Zip {
                iterators,
                finished,
                padding,
                mode,
            } => Self::zip_next(iterators, finished, padding, mode.clone(), context),
            IteratorHelperKind::ZipKeyed {
                keys,
                iterators,
                finished,
                padding,
                mode,
            } => Self::zip_keyed_next(keys, iterators, finished, padding, mode.clone(), context),
        }
    }

    fn close(&mut self, context: &mut Context) -> JsResult<JsValue> {
        match &mut self.kind {
            IteratorHelperKind::Map { iterated, .. }
            | IteratorHelperKind::Filter { iterated, .. }
            | IteratorHelperKind::Take { iterated, .. }
            | IteratorHelperKind::Drop { iterated, .. } => iterator_return_result(iterated, context),
            IteratorHelperKind::FlatMap { iterated, inner, .. } => {
                if let Some(inner) = inner.as_ref() {
                    iterator_return(inner, context)?;
                }
                iterator_return_result(iterated, context)
            }
            IteratorHelperKind::Concat { current, .. } => {
                if let Some(current) = current.as_ref() {
                    iterator_return_result(current, context)
                } else {
                    Ok(iterator_done_result(context))
                }
            }
            IteratorHelperKind::Zip {
                iterators, finished, ..
            } => close_iterators_reverse(iterators, finished, Ok(iterator_done_result(context)), None, context),
            IteratorHelperKind::ZipKeyed {
                iterators, finished, ..
            } => close_iterators_reverse(iterators, finished, Ok(iterator_done_result(context)), None, context),
        }
    }

    fn zip_next(
        iterators: &mut [IteratorRecord],
        finished: &mut [bool],
        padding: &[JsValue],
        mode: ZipMode,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if iterators.is_empty() {
            return Ok(iterator_done_result(context));
        }

        let mut results = Vec::with_capacity(iterators.len());
        let mut done_count = 0usize;
        for index in 0..iterators.len() {
            if finished[index] {
                done_count += 1;
                results.push(padding[index].clone());
                continue;
            }

            match iterators[index].step(context) {
                Ok(false) => match iterators[index].value(context) {
                    Ok(value) => results.push(value),
                    Err(error) => {
                        return close_iterators_reverse(
                            iterators,
                            finished,
                            Err(error),
                            Some(index),
                            context,
                        );
                    }
                },
                Ok(true) => {
                    finished[index] = true;
                    done_count += 1;
                    if matches!(mode, ZipMode::Shortest) {
                        return close_iterators_reverse(
                            iterators,
                            finished,
                            Ok(iterator_done_result(context)),
                            Some(index),
                            context,
                        );
                    }
                    if matches!(mode, ZipMode::Strict) {
                        if index != 0 {
                            return close_iterators_reverse(
                                iterators,
                                finished,
                                Err(JsNativeError::typ()
                                    .with_message("Iterator.zip strict mode requires equal lengths")
                                    .into()),
                                None,
                                context,
                            );
                        }

                        for check_index in 1..iterators.len() {
                            if finished[check_index] {
                                continue;
                            }
                            match iterators[check_index].step(context) {
                                Ok(false) => {
                                    return close_iterators_reverse(
                                        iterators,
                                        finished,
                                        Err(JsNativeError::typ()
                                            .with_message(
                                                "Iterator.zip strict mode requires equal lengths",
                                            )
                                            .into()),
                                        None,
                                        context,
                                    );
                                }
                                Ok(true) => {
                                    finished[check_index] = true;
                                    done_count += 1;
                                }
                                Err(error) => {
                                    return close_iterators_reverse(
                                        iterators,
                                        finished,
                                        Err(error),
                                        Some(check_index),
                                        context,
                                    );
                                }
                            }
                        }
                        return Ok(iterator_done_result(context));
                    }
                    results.push(padding[index].clone());
                }
                Err(error) => {
                    return close_iterators_reverse(
                        iterators,
                        finished,
                        Err(error),
                        Some(index),
                        context,
                    );
                }
            }
        }

        if done_count == iterators.len() {
            return Ok(iterator_done_result(context));
        }

        Ok(create_iter_result_object(
            Array::create_array_from_list(results, context).into(),
            false,
            context,
        ))
    }

    fn zip_keyed_next(
        keys: &[JsValue],
        iterators: &mut [IteratorRecord],
        finished: &mut [bool],
        padding: &[JsValue],
        mode: ZipMode,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if iterators.is_empty() {
            return Ok(iterator_done_result(context));
        }

        let mut results = Vec::with_capacity(iterators.len());
        let mut done_count = 0usize;
        for index in 0..iterators.len() {
            if finished[index] {
                done_count += 1;
                results.push(padding[index].clone());
                continue;
            }

            match iterators[index].step(context) {
                Ok(false) => match iterators[index].value(context) {
                    Ok(value) => results.push(value),
                    Err(error) => {
                        return close_iterators_reverse(
                            iterators,
                            finished,
                            Err(error),
                            Some(index),
                            context,
                        );
                    }
                },
                Ok(true) => {
                    finished[index] = true;
                    done_count += 1;
                    if matches!(mode, ZipMode::Shortest) {
                        return close_iterators_reverse(
                            iterators,
                            finished,
                            Ok(iterator_done_result(context)),
                            Some(index),
                            context,
                        );
                    }
                    if matches!(mode, ZipMode::Strict) {
                        if index != 0 {
                            return close_iterators_reverse(
                                iterators,
                                finished,
                                Err(JsNativeError::typ()
                                    .with_message(
                                        "Iterator.zipKeyed strict mode requires equal lengths",
                                    )
                                    .into()),
                                None,
                                context,
                            );
                        }

                        for check_index in 1..iterators.len() {
                            if finished[check_index] {
                                continue;
                            }
                            match iterators[check_index].step(context) {
                                Ok(false) => {
                                    return close_iterators_reverse(
                                        iterators,
                                        finished,
                                        Err(JsNativeError::typ()
                                            .with_message(
                                                "Iterator.zipKeyed strict mode requires equal lengths",
                                            )
                                            .into()),
                                        None,
                                        context,
                                    );
                                }
                                Ok(true) => {
                                    finished[check_index] = true;
                                    done_count += 1;
                                }
                                Err(error) => {
                                    return close_iterators_reverse(
                                        iterators,
                                        finished,
                                        Err(error),
                                        Some(check_index),
                                        context,
                                    );
                                }
                            }
                        }
                        return Ok(iterator_done_result(context));
                    }
                    results.push(padding[index].clone());
                }
                Err(error) => {
                    return close_iterators_reverse(
                        iterators,
                        finished,
                        Err(error),
                        Some(index),
                        context,
                    );
                }
            }
        }

        if done_count == iterators.len() {
            return Ok(iterator_done_result(context));
        }

        let object = JsObject::with_null_proto();
        for (index, key) in keys.iter().enumerate() {
            object.create_data_property_or_throw(key.to_property_key(context)?, results[index].clone(), context)?;
        }

        Ok(create_iter_result_object(object.into(), false, context))
    }
}

impl IteratorHelperPrototype {
    fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("%IteratorHelperPrototype%.next called on non-object")
        })?;
        let kind = {
            let mut helper = object.downcast_mut::<IteratorHelperObject>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("%IteratorHelperPrototype%.next requires internal slot")
            })?;

            if helper.executing {
                return Err(JsNativeError::typ()
                    .with_message("generator is already running")
                    .into());
            }
            if helper.completed {
                return Ok(iterator_done_result(context));
            }

            helper.executing = true;
            helper.started = true;
            helper.kind.clone()
        };

        let mut helper = IteratorHelperObject {
            executing: true,
            completed: false,
            started: true,
            kind,
        };
        let result = helper.advance(context);
        let completed = if let Ok(result_value) = &result {
            IteratorResult::from_value(result_value.clone())?.complete(context)?
        } else {
            false
        };

        let mut stored = object.downcast_mut::<IteratorHelperObject>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("%IteratorHelperPrototype%.next requires internal slot")
        })?;
        stored.executing = false;
        stored.completed = completed;
        stored.started = true;
        stored.kind = helper.kind.clone();
        result
    }

    fn return_(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("%IteratorHelperPrototype%.return called on non-object")
        })?;
        let (kind, started) = {
            let mut helper = object.downcast_mut::<IteratorHelperObject>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("%IteratorHelperPrototype%.return requires internal slot")
            })?;

            if helper.executing {
                return Err(JsNativeError::typ()
                    .with_message("generator is already running")
                    .into());
            }
            if helper.completed {
                return Ok(iterator_done_result(context));
            }

            helper.completed = true;
            if helper.started {
                helper.executing = true;
            }
            (helper.kind.clone(), helper.started)
        };

        let mut helper = IteratorHelperObject {
            executing: started,
            completed: true,
            started,
            kind,
        };
        let result = helper.close(context);

        let mut stored = object.downcast_mut::<IteratorHelperObject>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("%IteratorHelperPrototype%.return requires internal slot")
        })?;
        stored.executing = false;
        stored.completed = true;
        stored.started = started;
        stored.kind = helper.kind.clone();
        result
    }
}

impl Iterator {
    fn install_prototype_accessors(realm: &Realm) {
        let iterator_prototype = realm.intrinsics().objects().iterator_prototypes().iterator();
        let iterator_constructor = Self::get(realm.intrinsics());

        let constructor_getter = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, constructor, _| Ok(constructor.clone().into()),
                iterator_constructor,
            ),
        )
        .name(js_string!("get constructor"))
        .length(0)
        .build();
        let constructor_setter = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_copy_closure_with_captures(
                Self::set_constructor,
                iterator_prototype.clone(),
            ),
        )
        .name(js_string!("set constructor"))
        .length(1)
        .build();
        let tag_getter = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_fn_ptr(Self::get_to_string_tag),
        )
        .name(js_string!("get [Symbol.toStringTag]"))
        .length(0)
        .build();
        let tag_setter = FunctionObjectBuilder::new(
            realm,
            NativeFunction::from_copy_closure_with_captures(
                Self::set_to_string_tag,
                iterator_prototype.clone(),
            ),
        )
        .name(js_string!("set [Symbol.toStringTag]"))
        .length(1)
        .build();

        iterator_prototype.borrow_mut().insert(
            js_string!("constructor"),
            PropertyDescriptor::builder()
                .get(constructor_getter)
                .set(constructor_setter)
                .enumerable(false)
                .configurable(true),
        );
        iterator_prototype.borrow_mut().insert(
            JsSymbol::to_string_tag(),
            PropertyDescriptor::builder()
                .get(tag_getter)
                .set(tag_setter)
                .enumerable(false)
                .configurable(true),
        );
    }

    fn setter_that_ignores_prototype_properties(
        this: &JsValue,
        value: JsValue,
        home: &JsObject,
        key: PropertyKey,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("setter requires object receiver")
        })?;

        if JsObject::equals(&object, home) {
            return Err(JsNativeError::typ()
                .with_message("cannot assign to Iterator prototype accessor on home object")
                .into());
        }

        if object.has_own_property(key.clone(), context)? {
            object.set(key, value, true, context)?;
        } else {
            object.create_data_property_or_throw(key, value, context)?;
        }

        Ok(JsValue::undefined())
    }

    fn set_constructor(
        this: &JsValue,
        args: &[JsValue],
        home: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::setter_that_ignores_prototype_properties(
            this,
            args.get_or_undefined(0).clone(),
            home,
            js_string!("constructor").into(),
            context,
        )
    }

    fn get_to_string_tag(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(js_string!("Iterator").into())
    }

    fn set_to_string_tag(
        this: &JsValue,
        args: &[JsValue],
        home: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::setter_that_ignores_prototype_properties(
            this,
            args.get_or_undefined(0).clone(),
            home,
            JsSymbol::to_string_tag().into(),
            context,
        )
    }

    fn iterator(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(this.clone())
    }

    fn dispose(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype[Symbol.dispose] requires an object receiver")
        })?;
        let Some(method) = object.get_method(js_string!("return"), context)? else {
            return Ok(JsValue::undefined());
        };
        let result = method.call(&object.clone().into(), &[], context)?;
        IteratorResult::from_value(result)?;
        Ok(JsValue::undefined())
    }

    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0);
        if value.as_object().is_none() && value.as_string().is_none() {
            return Err(JsNativeError::typ()
                .with_message("Iterator.from requires an iterable or iterator object")
                .into());
        }

        if value.as_string().is_some() && value.as_object().is_none() {
            let method = value.get_v(JsSymbol::iterator(), context)?;
            if !method.is_object()
                || !method
                    .as_object()
                    .is_some_and(|method| method.is_callable())
            {
                return Err(JsNativeError::typ()
                    .with_message("value returned for property of object is not a function")
                    .into());
            }
            let method = method
                .as_object()
                .expect("checked object above")
                .clone();
            let iterator = value.get_iterator_from_method(&method, context)?;
            if same_iterator_prototype(&iterator.iterator().clone().into(), context)? {
                return Ok(iterator.iterator().clone().into());
            }
            return Ok(create_wrap_for_valid_iterator(
                iterator.iterator().clone(),
                iterator.next_method().clone(),
                context,
            )
            .into());
        }

        if let Some(method) = value.get_method(JsSymbol::iterator(), context)? {
            let iterator = value.get_iterator_from_method(&method, context)?;
            if same_iterator_prototype(&iterator.iterator().clone().into(), context)? {
                return Ok(iterator.iterator().clone().into());
            }
            return Ok(create_wrap_for_valid_iterator(
                iterator.iterator().clone(),
                iterator.next_method().clone(),
                context,
            )
            .into());
        }

        let iterator = get_iterator_direct(value, context)?;
        if same_iterator_prototype(&iterator.iterator().clone().into(), context)? {
            return Ok(iterator.iterator().clone().into());
        }
        Ok(create_wrap_for_valid_iterator(
            iterator.iterator().clone(),
            iterator.next_method().clone(),
            context,
        )
        .into())
    }

    fn concat(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mut sources = Vec::with_capacity(args.len());
        for item in args {
            let object = item.as_object().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Iterator.concat arguments must be iterable objects")
            })?;
            let method = object
                .get_method(JsSymbol::iterator(), context)?
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("Iterator.concat arguments must be iterable objects")
                })?;
            sources.push(ConcatSource {
                item: item.clone(),
                method,
            });
        }

        Ok(create_iterator_helper(
            IteratorHelperKind::Concat {
                sources,
                current: None,
            },
            context,
        )
        .into())
    }

    fn zip(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let _iterables_object = iterables.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.zip requires an iterable object")
        })?;

        let options = to_options_object(args.get_or_undefined(1))?;
        let mode = match options
            .as_ref()
            .map(|options| options.get(js_string!("mode"), context))
            .transpose()?
        {
            None => ZipMode::Shortest,
            Some(value) if value.is_undefined() => ZipMode::Shortest,
            Some(value) if value == js_string!("shortest").into() => ZipMode::Shortest,
            Some(value) if value == js_string!("longest").into() => ZipMode::Longest,
            Some(value) if value == js_string!("strict").into() => ZipMode::Strict,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Iterator.zip mode must be shortest, longest, or strict")
                    .into())
            }
        };

        let padding_option = if matches!(mode, ZipMode::Longest) {
            let padding = options
                .as_ref()
                .map(|options| options.get(js_string!("padding"), context))
                .transpose()?;
            if padding.as_ref().is_some_and(|value| !value.is_undefined() && !value.is_object()) {
                return Err(JsNativeError::typ()
                    .with_message("padding must be an object")
                    .into());
            }
            padding
        } else {
            None
        };

        let mut outer = iterables.get_iterator(IteratorHint::Sync, context)?;
        let mut iterators = Vec::new();
        loop {
            let value = match outer.step_value(context) {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(
                        iterators.as_slice(),
                        &finished,
                        Err(error),
                        None,
                        context,
                    );
                }
            };
            match get_iterator_flattenable(&value, false, context) {
                Ok(iterator) => iterators.push(iterator),
                Err(error) => {
                    let mut finished = vec![outer.done()];
                    finished.extend(std::iter::repeat_n(false, iterators.len()));
                    let mut all = vec![outer.clone()];
                    all.extend(iterators.iter().cloned());
                    return close_iterators_reverse(&all, &finished, Err(error), None, context);
                }
            }
        }

        let padding = if matches!(mode, ZipMode::Longest) {
            let padding_value = padding_option.unwrap_or(JsValue::undefined());
            match Self::collect_zip_padding(iterators.len(), &padding_value, context) {
                Ok(padding) => padding,
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(
                        iterators.as_slice(),
                        &finished,
                        Err(error),
                        None,
                        context,
                    );
                }
            }
        } else {
            vec![JsValue::undefined(); iterators.len()]
        };
        let finished = vec![false; iterators.len()];
        Ok(create_iterator_helper(
            IteratorHelperKind::Zip {
                iterators,
                finished,
                padding,
                mode,
            },
            context,
        )
        .into())
    }

    fn zip_keyed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let object = iterables.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.zipKeyed requires an object")
        })?;

        let options = to_options_object(args.get_or_undefined(1))?;
        let mode = match options
            .as_ref()
            .map(|options| options.get(js_string!("mode"), context))
            .transpose()?
        {
            None => ZipMode::Shortest,
            Some(value) if value.is_undefined() => ZipMode::Shortest,
            Some(value) if value == js_string!("shortest").into() => ZipMode::Shortest,
            Some(value) if value == js_string!("longest").into() => ZipMode::Longest,
            Some(value) if value == js_string!("strict").into() => ZipMode::Strict,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Iterator.zipKeyed mode must be shortest, longest, or strict")
                    .into())
            }
        };
        let padding_option = if matches!(mode, ZipMode::Longest) {
            let padding = options
                .as_ref()
                .map(|options| options.get(js_string!("padding"), context))
                .transpose()?;
            if padding.as_ref().is_some_and(|value| !value.is_undefined() && !value.is_object()) {
                return Err(JsNativeError::typ()
                    .with_message("padding must be an object")
                    .into());
            }
            padding.filter(|value| !value.is_undefined())
        } else {
            None
        };

        let mut keys = Vec::new();
        let mut iterators = Vec::new();
        for key in object.own_property_keys(context)? {
            let desc = match object.__get_own_property__(
                &key,
                &mut InternalMethodPropertyContext::new(context),
            ) {
                Ok(desc) => desc,
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(
                        iterators.as_slice(),
                        &finished,
                        Err(error),
                        None,
                        context,
                    );
                }
            };
            let Some(desc) = desc else {
                continue;
            };
            if !matches!(desc.enumerable(), Some(true)) {
                continue;
            }
            let value = match object.get(key.clone(), context) {
                Ok(value) => value,
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(
                        iterators.as_slice(),
                        &finished,
                        Err(error),
                        None,
                        context,
                    );
                }
            };
            if value.is_undefined() {
                continue;
            }
            match get_iterator_flattenable(&value, false, context) {
                Ok(iterator) => {
                    keys.push(JsValue::from(key));
                    iterators.push(iterator);
                }
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(iterators.as_slice(), &finished, Err(error), None, context);
                }
            }
        }

        let padding = if matches!(mode, ZipMode::Longest) {
            match Self::collect_zip_keyed_padding(&keys, padding_option.as_ref(), context) {
                Ok(padding) => padding,
                Err(error) => {
                    let finished = vec![false; iterators.len()];
                    return close_iterators_reverse(
                        iterators.as_slice(),
                        &finished,
                        Err(error),
                        None,
                        context,
                    );
                }
            }
        } else {
            vec![JsValue::undefined(); keys.len()]
        };
        let finished = vec![false; iterators.len()];
        Ok(create_iterator_helper(
            IteratorHelperKind::ZipKeyed {
                keys,
                iterators,
                finished,
                padding,
                mode,
            },
            context,
        )
        .into())
    }

    fn map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mapper = args.get_or_undefined(0);
        let mapper = mapper.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("mapper must be callable")
        })?;
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(
            IteratorHelperKind::Map {
                iterated,
                mapper: mapper.clone(),
                counter: 0,
            },
            context,
        )
        .into())
    }

    fn filter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let predicate = args.get_or_undefined(0);
        let predicate = predicate.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("predicate must be callable")
        })?;
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(
            IteratorHelperKind::Filter {
                iterated,
                predicate: predicate.clone(),
                counter: 0,
            },
            context,
        )
        .into())
    }

    fn take(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_object() {
            return Err(JsNativeError::typ()
                .with_message("iterator helper receiver must be an object")
                .into());
        }
        let remaining = match to_positive_integer_or_infinity(args.get_or_undefined(0), context) {
            Ok(remaining) => remaining,
            Err(error) => return close_if_possible(this, context).and(Err(error)),
        };
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(
            IteratorHelperKind::Take {
                iterated,
                remaining,
                exhausted: false,
            },
            context,
        )
        .into())
    }

    fn drop(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_object() {
            return Err(JsNativeError::typ()
                .with_message("iterator helper receiver must be an object")
                .into());
        }
        let remaining = match to_positive_integer_or_infinity(args.get_or_undefined(0), context) {
            Ok(remaining) => remaining,
            Err(error) => return close_if_possible(this, context).and(Err(error)),
        };
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(
            IteratorHelperKind::Drop {
                iterated,
                remaining,
                advanced: false,
            },
            context,
        )
        .into())
    }

    fn flat_map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mapper = args.get_or_undefined(0);
        let mapper = mapper.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("mapper must be callable")
        })?;
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(
            IteratorHelperKind::FlatMap {
                iterated,
                mapper: mapper.clone(),
                counter: 0,
                inner: None,
            },
            context,
        )
        .into())
    }

    fn reduce(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let reducer = args.get_or_undefined(0);
        let reducer = reducer.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("reducer must be callable")
        })?;
        let mut iterated = get_iterator_direct(this, context)?;
        let mut index = 0u64;
        let mut accumulator = if args.len() > 1 {
            Some(args[1].clone())
        } else {
            None
        };

        while let Some(value) = iterated.step_value(context)? {
            if let Some(current) = accumulator {
                let next = reducer.call(
                    &JsValue::undefined(),
                    &[current, value, JsValue::new(index as i32)],
                    context,
                );
                accumulator = Some(match next {
                    Ok(next) => next,
                    Err(error) => return iterated.close(Err(error), context),
                });
                index += 1;
            } else {
                accumulator = Some(value);
                index += 1;
            }
        }

        accumulator.ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Reduce of empty iterator with no initial value")
                .into()
        })
    }

    fn to_array(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mut iterated = get_iterator_direct(this, context)?;
        let mut values = Vec::new();
        while let Some(value) = iterated.step_value(context)? {
            values.push(value);
        }
        Ok(Array::create_array_from_list(values, context).into())
    }

    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let callback = args.get_or_undefined(0);
        let callback = callback.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("callback must be callable")
        })?;
        let mut iterated = get_iterator_direct(this, context)?;
        let mut index = 0u64;
        while let Some(value) = iterated.step_value(context)? {
            if let Err(error) = callback.call(
                &JsValue::undefined(),
                &[value, JsValue::new(index as i32)],
                context,
            ) {
                return iterated.close(Err(error), context);
            }
            index += 1;
        }
        Ok(JsValue::undefined())
    }

    fn some(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let predicate = args.get_or_undefined(0);
        let predicate = predicate.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("predicate must be callable")
        })?;
        let mut iterated = get_iterator_direct(this, context)?;
        let mut index = 0u64;
        while let Some(value) = iterated.step_value(context)? {
            let matches = match predicate.call(
                &JsValue::undefined(),
                &[value.clone(), JsValue::new(index as i32)],
                context,
            ) {
                Ok(matches) => matches,
                Err(error) => return iterated.close(Err(error), context),
            };
            if matches.to_boolean() {
                iterator_return(&iterated, context)?;
                return Ok(true.into());
            }
            index += 1;
        }
        Ok(false.into())
    }

    fn every(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let predicate = args.get_or_undefined(0);
        let predicate = predicate.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("predicate must be callable")
        })?;
        let mut iterated = get_iterator_direct(this, context)?;
        let mut index = 0u64;
        while let Some(value) = iterated.step_value(context)? {
            let matches = match predicate.call(
                &JsValue::undefined(),
                &[value.clone(), JsValue::new(index as i32)],
                context,
            ) {
                Ok(matches) => matches,
                Err(error) => return iterated.close(Err(error), context),
            };
            if !matches.to_boolean() {
                iterator_return(&iterated, context)?;
                return Ok(false.into());
            }
            index += 1;
        }
        Ok(true.into())
    }

    fn find(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let predicate = args.get_or_undefined(0);
        let predicate = predicate.as_callable().ok_or_else(|| {
            let _ = close_if_possible(this, context);
            JsNativeError::typ().with_message("predicate must be callable")
        })?;
        let mut iterated = get_iterator_direct(this, context)?;
        let mut index = 0u64;
        while let Some(value) = iterated.step_value(context)? {
            let matches = match predicate.call(
                &JsValue::undefined(),
                &[value.clone(), JsValue::new(index as i32)],
                context,
            ) {
                Ok(matches) => matches,
                Err(error) => return iterated.close(Err(error), context),
            };
            if matches.to_boolean() {
                iterator_return(&iterated, context)?;
                return Ok(value);
            }
            index += 1;
        }
        Ok(JsValue::undefined())
    }

    fn collect_zip_padding(
        count: usize,
        padding_option: &JsValue,
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
        if padding_option.is_undefined() {
            return Ok(vec![JsValue::undefined(); count]);
        }
        let mut iterator = padding_option.get_iterator(IteratorHint::Sync, context)?;
        let mut values = Vec::with_capacity(count);
        let mut using_iterator = true;
        for _ in 0..count {
            if using_iterator {
                match iterator.step_value(context)? {
                    Some(value) => values.push(value),
                    None => {
                        using_iterator = false;
                        values.push(JsValue::undefined());
                    }
                }
            } else {
                values.push(JsValue::undefined());
            }
        }
        if using_iterator {
            iterator_return(&iterator, context)?;
        }
        Ok(values)
    }

    fn collect_zip_keyed_padding(
        keys: &[JsValue],
        padding_option: Option<&JsValue>,
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
        let Some(padding) = padding_option else {
            return Ok(vec![JsValue::undefined(); keys.len()]);
        };
        let padding = padding.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("padding must be an object")
        })?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(padding.get(key.to_property_key(context)?, context)?);
        }
        Ok(values)
    }
}

/// `CreateIterResultObject( value, done )`
///
/// Generates an object supporting the `IteratorResult` interface.
pub fn create_iter_result_object(value: JsValue, done: bool, context: &mut Context) -> JsValue {
    // 1. Assert: Type(done) is Boolean.
    // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
    // 3. Perform ! CreateDataPropertyOrThrow(obj, "value", value).
    // 4. Perform ! CreateDataPropertyOrThrow(obj, "done", done).
    let obj = context
        .intrinsics()
        .templates()
        .iterator_result()
        .create(OrdinaryObject, vec![value, done.into()]);

    // 5. Return obj.
    obj.into()
}

/// Iterator hint for `GetIterator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorHint {
    /// Hints that the iterator should be sync.
    Sync,

    /// Hints that the iterator should be async.
    Async,
}

impl JsValue {
    /// `GetIteratorFromMethod ( obj, method )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiteratorfrommethod
    pub fn get_iterator_from_method(
        &self,
        method: &JsObject,
        context: &mut Context,
    ) -> JsResult<IteratorRecord> {
        // 1. Let iterator be ? Call(method, obj).
        let iterator = method.call(self, &[], context)?;
        // 2. If iterator is not an Object, throw a TypeError exception.
        let iterator_obj = iterator.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("returned iterator is not an object")
        })?;
        // 3. Let nextMethod be ? Get(iterator, "next").
        let next_method = iterator_obj.get(js_string!("next"), context)?;
        // 4. Let iteratorRecord be the Iterator Record { [[Iterator]]: iterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 5. Return iteratorRecord.
        Ok(IteratorRecord::new(iterator_obj.clone(), next_method))
    }

    /// `GetIterator ( obj, kind )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiterator
    pub fn get_iterator(
        &self,
        hint: IteratorHint,
        context: &mut Context,
    ) -> JsResult<IteratorRecord> {
        let method = match hint {
            // 1. If kind is async, then
            IteratorHint::Async => {
                // a. Let method be ? GetMethod(obj, %Symbol.asyncIterator%).
                let Some(method) = self.get_method(JsSymbol::async_iterator(), context)? else {
                    // b. If method is undefined, then
                    //     i. Let syncMethod be ? GetMethod(obj, %Symbol.iterator%).
                    let sync_method =
                        self.get_method(JsSymbol::iterator(), context)?
                            .ok_or_else(|| {
                                // ii. If syncMethod is undefined, throw a TypeError exception.
                                JsNativeError::typ().with_message(format!(
                                    "value with type `{}` is not iterable",
                                    self.type_of()
                                ))
                            })?;
                    // iii. Let syncIteratorRecord be ? GetIteratorFromMethod(obj, syncMethod).
                    let sync_iterator_record =
                        self.get_iterator_from_method(&sync_method, context)?;
                    // iv. Return CreateAsyncFromSyncIterator(syncIteratorRecord).
                    return Ok(AsyncFromSyncIterator::create(sync_iterator_record, context));
                };

                Some(method)
            }
            // 2. Else,
            IteratorHint::Sync => {
                // a. Let method be ? GetMethod(obj, %Symbol.iterator%).
                self.get_method(JsSymbol::iterator(), context)?
            }
        };

        let method = method.ok_or_else(|| {
            // 3. If method is undefined, throw a TypeError exception.
            JsNativeError::typ().with_message(format!(
                "value with type `{}` is not iterable",
                self.type_of()
            ))
        })?;

        // 4. Return ? GetIteratorFromMethod(obj, method).
        self.get_iterator_from_method(&method, context)
    }
}

/// The result of the iteration process.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IteratorResult {
    object: JsObject,
}

impl IteratorResult {
    /// Gets a new `IteratorResult` from a value. Returns `Err` if
    /// the value is not a [`JsObject`]
    pub(crate) fn from_value(value: JsValue) -> JsResult<Self> {
        if let Some(object) = value.into_object() {
            Ok(Self { object })
        } else {
            Err(JsNativeError::typ()
                .with_message("next value should be an object")
                .into())
        }
    }

    /// Gets the inner object of this `IteratorResult`.
    pub(crate) const fn object(&self) -> &JsObject {
        &self.object
    }

    /// `IteratorComplete ( iterResult )`
    ///
    /// The abstract operation `IteratorComplete` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing a `Boolean` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorcomplete
    #[inline]
    pub fn complete(&self, context: &mut Context) -> JsResult<bool> {
        // 1. Return ToBoolean(? Get(iterResult, "done")).
        Ok(self.object.get(js_string!("done"), context)?.to_boolean())
    }

    /// `IteratorValue ( iterResult )`
    ///
    /// The abstract operation `IteratorValue` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing an ECMAScript language value or a throw
    /// completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorvalue
    #[inline]
    pub fn value(&self, context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? Get(iterResult, "value").
        self.object.get(js_string!("value"), context)
    }
}

/// Iterator Record
///
/// An Iterator Record is a Record value used to encapsulate an
/// `Iterator` or `AsyncIterator` along with the `next` method.
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-records
#[derive(Clone, Debug, Finalize, Trace)]
pub struct IteratorRecord {
    /// `[[Iterator]]`
    ///
    /// An object that conforms to the `Iterator` or `AsyncIterator` interface.
    iterator: JsObject,

    /// `[[NextMethod]]`
    ///
    /// The `next` method of the `[[Iterator]]` object.
    next_method: JsValue,

    /// `[[Done]]`
    ///
    /// Whether the iterator has been closed.
    done: bool,

    /// The result of the last call to `next`.
    last_result: IteratorResult,
}

impl IteratorRecord {
    /// Creates a new `IteratorRecord` with the given iterator object, next method and `done` flag.
    #[inline]
    #[must_use]
    pub fn new(iterator: JsObject, next_method: JsValue) -> Self {
        Self {
            iterator,
            next_method,
            done: false,
            last_result: IteratorResult {
                object: JsObject::with_null_proto(),
            },
        }
    }

    /// Get the `[[Iterator]]` field of the `IteratorRecord`.
    pub(crate) const fn iterator(&self) -> &JsObject {
        &self.iterator
    }

    /// Gets the `[[NextMethod]]` field of the `IteratorRecord`.
    pub(crate) const fn next_method(&self) -> &JsValue {
        &self.next_method
    }

    /// Gets the last result object of the iterator record.
    pub(crate) const fn last_result(&self) -> &IteratorResult {
        &self.last_result
    }

    /// Runs `f`, setting the `done` field of this `IteratorRecord` to `true` if `f` returns
    /// an error.
    fn set_done_on_err<R, F>(&mut self, f: F) -> JsResult<R>
    where
        F: FnOnce(&mut Self) -> JsResult<R>,
    {
        let result = f(self);
        if result.is_err() {
            self.done = true;
        }
        result
    }

    /// Gets the current value of the `IteratorRecord`.
    pub(crate) fn value(&mut self, context: &mut Context) -> JsResult<JsValue> {
        self.set_done_on_err(|iter| iter.last_result.value(context))
    }

    /// Get the `[[Done]]` field of the `IteratorRecord`.
    pub(crate) const fn done(&self) -> bool {
        self.done
    }

    /// Updates the current result value of this iterator record.
    pub(crate) fn update_result(&mut self, result: JsValue, context: &mut Context) -> JsResult<()> {
        self.set_done_on_err(|iter| {
            // 3. If Type(result) is not Object, throw a TypeError exception.
            // 4. Return result.
            // `IteratorResult::from_value` does this for us.

            // `IteratorStep(iteratorRecord)`
            // https://tc39.es/ecma262/#sec-iteratorstep

            // 1. Let result be ? IteratorNext(iteratorRecord).
            let result = IteratorResult::from_value(result)?;
            // 2. Let done be ? IteratorComplete(result).
            // 3. If done is true, return false.
            iter.done = result.complete(context)?;

            iter.last_result = result;

            Ok(())
        })
    }

    /// `IteratorNext ( iteratorRecord [ , value ] )`
    ///
    /// The abstract operation `IteratorNext` takes argument `iteratorRecord` (an `Iterator`
    /// Record) and optional argument `value` (an ECMAScript language value) and returns either a
    /// normal completion containing an `Object` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratornext
    pub(crate) fn next(
        &mut self,
        value: Option<&JsValue>,
        context: &mut Context,
    ) -> JsResult<IteratorResult> {
        // 1. If value is not present, then
        //     a. Let result be Completion(Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]])).
        // 2. Else,
        //     a. Let result be Completion(Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »)).
        // 3. If result is a throw completion, then
        //     a. Set iteratorRecord.[[Done]] to true.
        //     b. Return ? result.
        // 4. Set result to ! result.
        // 5. If result is not an Object, then
        //     a. Set iteratorRecord.[[Done]] to true.
        //     b. Throw a TypeError exception.
        // 6. Return result.
        // NOTE: In this case, `set_done_on_err` does all the heavylifting for us, which
        // simplifies the instructions below.
        self.set_done_on_err(|iter| {
            iter.next_method
                .call(
                    &iter.iterator.clone().into(),
                    value.map_or(&[], std::slice::from_ref),
                    context,
                )
                .and_then(IteratorResult::from_value)
        })
    }

    /// `IteratorStep ( iteratorRecord )`
    ///
    /// Updates the `IteratorRecord` and returns `true` if the next result record returned
    /// `done: true`, otherwise returns `false`. This differs slightly from the spec, but also
    /// simplifies some logic around iterators.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstep
    pub(crate) fn step(&mut self, context: &mut Context) -> JsResult<bool> {
        self.set_done_on_err(|iter| {
            // 1. Let result be ? IteratorNext(iteratorRecord).
            let result = iter.next(None, context)?;

            // 2. Let done be Completion(IteratorComplete(result)).
            // 3. If done is a throw completion, then
            //     a. Set iteratorRecord.[[Done]] to true.
            //     b. Return ? done.
            // 4. Set done to ! done.
            // 5. If done is true, then
            //     a. Set iteratorRecord.[[Done]] to true.
            //     b. Return done.
            iter.done = result.complete(context)?;

            iter.last_result = result;

            // 6. Return result.
            Ok(iter.done)
        })
    }

    /// `IteratorStepValue ( iteratorRecord )`
    ///
    /// Updates the `IteratorRecord` and returns `Some(value)` if the next result record returned
    /// `done: true`, otherwise returns `None`.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstepvalue
    pub(crate) fn step_value(&mut self, context: &mut Context) -> JsResult<Option<JsValue>> {
        // 1. Let result be ? IteratorStep(iteratorRecord).
        if self.step(context)? {
            // 2. If result is done, then
            //     a. Return done.
            Ok(None)
        } else {
            // 3. Let value be Completion(IteratorValue(result)).
            // 4. If value is a throw completion, then
            //     a. Set iteratorRecord.[[Done]] to true.
            // 5. Return ? value.
            self.value(context).map(Some)
        }
    }

    /// `IteratorClose ( iteratorRecord, completion )`
    ///
    /// The abstract operation `IteratorClose` takes arguments `iteratorRecord` (an
    /// [Iterator Record][Self]) and `completion` (a `Completion` Record) and returns a
    /// `Completion` Record. It is used to notify an iterator that it should perform any actions it
    /// would normally perform when it has reached its completed state.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratorclose
    pub(crate) fn close(
        &self,
        completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(iteratorRecord.[[Iterator]]) is Object.

        // 2. Let iterator be iteratorRecord.[[Iterator]].
        let iterator = &self.iterator;

        // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
        let inner_result = iterator.get_method(js_string!("return"), context);

        // 4. If innerResult.[[Type]] is normal, then
        let inner_result = match inner_result {
            Ok(inner_result) => {
                // a. Let return be innerResult.[[Value]].
                let r#return = inner_result;

                if let Some(r#return) = r#return {
                    // c. Set innerResult to Completion(Call(return, iterator)).
                    r#return.call(&iterator.clone().into(), &[], context)
                } else {
                    // b. If return is undefined, return ? completion.
                    return completion;
                }
            }
            Err(inner_result) => {
                // 5. If completion.[[Type]] is throw, return ? completion.
                completion?;

                // 6. If innerResult.[[Type]] is throw, return ? innerResult.
                return Err(inner_result);
            }
        };

        // 5. If completion.[[Type]] is throw, return ? completion.
        let completion = completion?;

        // 6. If innerResult.[[Type]] is throw, return ? innerResult.
        let inner_result = inner_result?;

        if inner_result.is_object() {
            // 8. Return ? completion.
            Ok(completion)
        } else {
            // 7. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
            Err(JsNativeError::typ()
                .with_message("inner result was not an object")
                .into())
        }
    }

    /// `IteratorToList ( iteratorRecord )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratortolist
    pub(crate) fn into_list(mut self, context: &mut Context) -> JsResult<Vec<JsValue>> {
        // 1. Let values be a new empty List.
        let mut values = Vec::new();

        // 2. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(value) = self.step_value(context)? {
            // c. Append next to values.
            values.push(value);
        }

        //     b. If next is done, then
        //         i. Return values.
        Ok(values)
    }
}
