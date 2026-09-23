//! The one authority that may resolve a property name to a catalogued method
//! of an intrinsic prototype.
//!
//! Static call lowering maps `(receiver kind, property name)` to a builtin so
//! it can pick a fast path and a result kind. Each of those tables used to
//! trust the catalogue unconditionally, so a program that had replaced
//! `String.prototype.toUpperCase` or `Function.prototype.apply` got the
//! builtin's result kind — or the builtin itself — instead of its own
//! function. The catalogue is only a claim about a fresh realm; whether the
//! claim still holds at a program point is a flow fact.
//!
//! [`IntrinsicMethod`] is that fact. Its field is private to this module, so
//! [`ScriptLowerer::intrinsic_method`] is its only constructor, and every
//! static resolution that wants the exact builtin has to go through the same
//! proof. A catalogued name without a proof is [`IntrinsicMethodLookup::Unproven`]:
//! the builtin stays a possible (and therefore still emitted) callee, but the
//! read is an ordinary property read and licenses no result kind.
use super::*;

/// An intrinsic prototype object that static lowering resolves methods on.
///
/// Every variant is `%C.prototype%` for a global constructor `C` whose
/// catalogued shape carries a `prototype` data property, which is where
/// writes to the prototype are recorded (`IntrinsicPrototypeShape`
/// publication) and where aliasing writes, deletes and unknown effects erase
/// what was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IntrinsicPrototype {
    Function,
    String,
    Number,
    Boolean,
    BigInt,
    Symbol,
    Iterator,
    RegExp,
}

impl IntrinsicPrototype {
    const fn constructor(self) -> StandardBuiltinId {
        match self {
            Self::Function => StandardBuiltinId::FunctionConstructor,
            Self::String => StandardBuiltinId::StringConstructor,
            Self::Number => StandardBuiltinId::NumberConstructor,
            Self::Boolean => StandardBuiltinId::BooleanConstructor,
            Self::BigInt => StandardBuiltinId::BigIntConstructor,
            Self::Symbol => StandardBuiltinId::SymbolConstructor,
            Self::Iterator => StandardBuiltinId::IteratorConstructor,
            Self::RegExp => StandardBuiltinId::RegExpConstructor,
        }
    }

    /// The builtin `%C.prototype%[name]` holds in a fresh realm, for the names
    /// static lowering resolves. Any other name is `None`, including names the
    /// prototype does have but that no lowering path specializes.
    pub(super) fn catalogued_method(self, name: &str) -> Option<StandardBuiltinId> {
        use StandardBuiltinId as B;
        Some(match (self, name) {
            (Self::Function, "call") => B::FunctionPrototypeCall,
            (Self::Function, "apply") => B::FunctionPrototypeApply,
            (Self::Function, "bind") => B::FunctionPrototypeBind,
            (Self::Function, "toString") => B::FunctionPrototypeToString,
            (Self::String, "charAt") => B::StringPrototypeCharAt,
            (Self::String, "concat") => B::StringPrototypeConcat,
            (Self::String, "charCodeAt") => B::StringPrototypeCharCodeAt,
            (Self::String, "codePointAt") => B::StringPrototypeCodePointAt,
            (Self::String, "at") => B::StringPrototypeAt,
            (Self::String, "anchor") => B::StringPrototypeAnchor,
            (Self::String, "big") => B::StringPrototypeBig,
            (Self::String, "blink") => B::StringPrototypeBlink,
            (Self::String, "bold") => B::StringPrototypeBold,
            (Self::String, "fixed") => B::StringPrototypeFixed,
            (Self::String, "fontcolor") => B::StringPrototypeFontcolor,
            (Self::String, "fontsize") => B::StringPrototypeFontsize,
            (Self::String, "italics") => B::StringPrototypeItalics,
            (Self::String, "link") => B::StringPrototypeLink,
            (Self::String, "small") => B::StringPrototypeSmall,
            (Self::String, "strike") => B::StringPrototypeStrike,
            (Self::String, "sub") => B::StringPrototypeSub,
            (Self::String, "substr") => B::StringPrototypeSubstr,
            (Self::String, "substring") => B::StringPrototypeSubstring,
            (Self::String, "sup") => B::StringPrototypeSup,
            (Self::String, "match") => B::StringPrototypeMatch,
            (Self::String, "matchAll") => B::StringPrototypeMatchAll,
            (Self::String, "replace") => B::StringPrototypeReplace,
            (Self::String, "replaceAll") => B::StringPrototypeReplaceAll,
            (Self::String, "search") => B::StringPrototypeSearch,
            (Self::String, "indexOf") => B::StringPrototypeIndexOf,
            (Self::String, "lastIndexOf") => B::StringPrototypeLastIndexOf,
            (Self::String, "slice") => B::StringPrototypeSlice,
            (Self::String, "split") => B::StringPrototypeSplit,
            (Self::String, "padStart") => B::StringPrototypePadStart,
            (Self::String, "padEnd") => B::StringPrototypePadEnd,
            (Self::String, "repeat") => B::StringPrototypeRepeat,
            (Self::String, "endsWith") => B::StringPrototypeEndsWith,
            (Self::String, "includes") => B::StringPrototypeIncludes,
            (Self::String, "startsWith") => B::StringPrototypeStartsWith,
            (Self::String, "normalize") => B::StringPrototypeNormalize,
            (Self::String, "localeCompare") => B::StringPrototypeLocaleCompare,
            (Self::String, "toLocaleLowerCase") => B::StringPrototypeToLocaleLowerCase,
            (Self::String, "toLocaleUpperCase") => B::StringPrototypeToLocaleUpperCase,
            (Self::String, "toLowerCase") => B::StringPrototypeToLowerCase,
            (Self::String, "toUpperCase") => B::StringPrototypeToUpperCase,
            (Self::String, "toString") => B::StringPrototypeToString,
            (Self::String, "valueOf") => B::StringPrototypeValueOf,
            (Self::String, "trim") => B::StringPrototypeTrim,
            // B.2.2.15-16: the Annex B names are the same function objects.
            (Self::String, "trimStart" | "trimLeft") => B::StringPrototypeTrimStart,
            (Self::String, "trimEnd" | "trimRight") => B::StringPrototypeTrimEnd,
            (Self::String, "isWellFormed") => B::StringPrototypeIsWellFormed,
            (Self::String, "toWellFormed") => B::StringPrototypeToWellFormed,
            (Self::Number, "toExponential") => B::NumberPrototypeToExponential,
            (Self::Number, "toFixed") => B::NumberPrototypeToFixed,
            (Self::Number, "toLocaleString") => B::NumberPrototypeToLocaleString,
            (Self::Number, "toPrecision") => B::NumberPrototypeToPrecision,
            (Self::Number, "toString") => B::NumberPrototypeToString,
            (Self::Number, "valueOf") => B::NumberPrototypeValueOf,
            (Self::Boolean, "toString") => B::BooleanPrototypeToString,
            (Self::Boolean, "valueOf") => B::BooleanPrototypeValueOf,
            (Self::BigInt, "toString") => B::BigIntPrototypeToString,
            (Self::BigInt, "toLocaleString") => B::BigIntPrototypeToLocaleString,
            (Self::BigInt, "valueOf") => B::BigIntPrototypeValueOf,
            (Self::Symbol, "toString") => B::SymbolPrototypeToString,
            (Self::Symbol, "valueOf") => B::SymbolPrototypeValueOf,
            (Self::Iterator, "toArray") => B::IteratorPrototypeToArray,
            (Self::Iterator, "forEach") => B::IteratorPrototypeForEach,
            (Self::Iterator, "every") => B::IteratorPrototypeEvery,
            (Self::Iterator, "some") => B::IteratorPrototypeSome,
            (Self::Iterator, "find") => B::IteratorPrototypeFind,
            (Self::Iterator, "reduce") => B::IteratorPrototypeReduce,
            (Self::Iterator, "map") => B::IteratorPrototypeMap,
            (Self::Iterator, "filter") => B::IteratorPrototypeFilter,
            (Self::Iterator, "flatMap") => B::IteratorPrototypeFlatMap,
            (Self::Iterator, "take") => B::IteratorPrototypeTake,
            (Self::Iterator, "drop") => B::IteratorPrototypeDrop,
            (Self::RegExp, "exec") => B::RegExpPrototypeExec,
            (Self::RegExp, "test") => B::RegExpPrototypeTest,
            _ => return None,
        })
    }
}

/// Proof that, at the current program point, `%C.prototype%[name]` still
/// holds the builtin the catalogue names for it.
///
/// It says nothing about the receiver: a caller that reads the method from a
/// receiver must separately know that the receiver neither has an own property
/// of that name nor a different prototype chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IntrinsicMethod {
    builtin: StandardBuiltinId,
}

impl IntrinsicMethod {
    pub(super) fn builtin(self) -> StandardBuiltinId {
        self.builtin
    }

    /// The exact callee fact the proof licenses.
    pub(super) fn callee_info(self) -> ValueInfo {
        ScriptLowerer::standard_builtin_value_info(self.builtin)
    }
}

/// The outcome of resolving one property name on one intrinsic prototype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IntrinsicMethodLookup {
    Proven(IntrinsicMethod),
    /// The name is catalogued, but a write, delete or unknown effect may have
    /// replaced the builtin.
    Unproven(StandardBuiltinId),
    /// No lowering path specializes this name on this prototype.
    Uncatalogued,
}

impl IntrinsicMethodLookup {
    pub(super) fn proven(self) -> Option<IntrinsicMethod> {
        match self {
            Self::Proven(method) => Some(method),
            Self::Unproven(_) | Self::Uncatalogued => None,
        }
    }

    /// The same lookup for a receiver whose own properties are unknown: the
    /// prototype may still hold the builtin, but the receiver may shadow it.
    pub(super) fn unclaimed(self) -> Self {
        match self {
            Self::Proven(method) => Self::Unproven(method.builtin),
            Self::Unproven(builtin) => Self::Unproven(builtin),
            Self::Uncatalogued => Self::Uncatalogued,
        }
    }

    /// The callee fact a read of the looked-up name may carry: exact under a
    /// proof, and otherwise an open target set that keeps the builtin a
    /// possible — and so still emitted — callee without claiming it.
    pub(super) fn callee_info(self) -> Option<ValueInfo> {
        match self {
            Self::Proven(method) => Some(method.callee_info()),
            Self::Unproven(builtin) => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::Open(BTreeSet::from([
                    builtin.function_id()
                ])),
            }),
            Self::Uncatalogued => None,
        }
    }
}

impl ScriptLowerer<'_> {
    /// Resolve `name` on `prototype`, proving the catalogued builtin is still
    /// installed. The only constructor of [`IntrinsicMethod`].
    pub(super) fn intrinsic_method(
        &self,
        prototype: IntrinsicPrototype,
        name: &str,
    ) -> IntrinsicMethodLookup {
        let Some(builtin) = prototype.catalogued_method(name) else {
            return IntrinsicMethodLookup::Uncatalogued;
        };
        if self.intrinsic_prototype_still_holds(prototype, name, builtin) {
            IntrinsicMethodLookup::Proven(IntrinsicMethod { builtin })
        } else {
            IntrinsicMethodLookup::Unproven(builtin)
        }
    }

    /// Whether a fold that computes `%C.prototype%[name]`'s result at compile
    /// time may still assume the builtin is the method that runs.
    pub(super) fn intrinsic_method_is_proven(
        &self,
        prototype: IntrinsicPrototype,
        name: &str,
    ) -> bool {
        self.intrinsic_method(prototype, name).proven().is_some()
    }

    /// `receiver[name]` for an object receiver: its shape must resolve the
    /// name, through its own prototype chain, to the builtin `prototype`
    /// catalogues, and that prototype must still hold it. The shape alone is a
    /// snapshot taken when the object was created and says nothing about
    /// writes to the shared prototype since.
    pub(super) fn shaped_receiver_intrinsic_method(
        &self,
        receiver: &TypedExpr,
        prototype: IntrinsicPrototype,
        name: &str,
    ) -> Option<IntrinsicMethod> {
        let method = self.intrinsic_method(prototype, name).proven()?;
        match read_heap_shape_property(receiver.heap_shape.as_deref()?, name)? {
            ObjectShapeProperty::Data(info) => (info.function_targets.exact_single_target()
                == Some(&method.builtin.function_id()))
            .then_some(method),
            ObjectShapeProperty::Accessor { .. } => None,
        }
    }

    /// A read of `name` from a primitive (or otherwise own-property-free)
    /// receiver whose prototype is `prototype`, carrying only the callee fact
    /// the lookup licenses. `None` for an uncatalogued name.
    pub(super) fn intrinsic_method_read(
        &self,
        prototype: IntrinsicPrototype,
        receiver: &TypedExpr,
        name: &str,
    ) -> Option<TypedExpr> {
        let info = self.intrinsic_method(prototype, name).callee_info()?;
        Some(TypedExpr::from_info(
            info,
            ExprIr::PropertyRead {
                target: Box::new(receiver.clone()),
                key: PropertyKeyIr::StaticString(name.to_string()),
            },
        ))
    }

    /// The recorded value of `%C%.prototype` when the global binding `C` still
    /// denotes the intrinsic `constructor`.
    ///
    /// The global property is the only place a write to the intrinsic
    /// prototype is published, and every event that could change the
    /// prototype without publishing — an aliasing write, a delete, a
    /// `defineProperty`, unknown user code — erases this recorded shape. A
    /// snapshot of the prototype taken from anywhere else, such as the
    /// catalogue's fresh-realm shape, is not evidence about the current
    /// program point.
    pub(super) fn live_intrinsic_prototype(
        &self,
        constructor: StandardBuiltinId,
    ) -> Option<&ValueInfo> {
        let global = self
            .global_properties
            .get(constructor.global_name()?)
            .filter(|property| property.proven_present)?;
        if global.value_info.function_targets.exact_single_target()
            != Some(&constructor.function_id())
        {
            return None;
        }
        let HeapShape::Object(constructor_shape) = global.value_info.heap_shape.as_deref()? else {
            return None;
        };
        match constructor_shape.properties.get("prototype")? {
            ObjectShapeProperty::Data(prototype) => Some(prototype),
            ObjectShapeProperty::Accessor { .. } => None,
        }
    }

    fn intrinsic_prototype_still_holds(
        &self,
        prototype: IntrinsicPrototype,
        name: &str,
        builtin: StandardBuiltinId,
    ) -> bool {
        let Some(live) = self.live_intrinsic_prototype(prototype.constructor()) else {
            return false;
        };
        // `%Function.prototype%` is itself callable; the other prototypes are
        // ordinary objects. A recorded value of another identity is not the
        // intrinsic whatever its shape says.
        let expected_identity = (prototype == IntrinsicPrototype::Function)
            .then(|| StandardBuiltinId::FunctionPrototype.function_id());
        if live.function_targets.exact_single_target() != expected_identity.as_ref() {
            return false;
        }
        let Some(live_shape) = live.heap_shape.as_deref() else {
            return false;
        };
        match own_shape_property(live_shape, name) {
            Some(ObjectShapeProperty::Data(method)) => {
                method.function_targets.exact_single_target() == Some(&builtin.function_id())
            }
            Some(ObjectShapeProperty::Accessor { .. }) => false,
            // The fresh-realm shape omits some catalogued methods. A write
            // publishes an explicit fact, while delete and unknown effects
            // erase the shape, so absence is the intrinsic state exactly when
            // the fresh-realm shape omits the name too.
            None => {
                let pristine = Self::standard_builtin_function_shape(prototype.constructor());
                match own_shape_property(&pristine, "prototype") {
                    Some(ObjectShapeProperty::Data(prototype)) => prototype
                        .heap_shape
                        .as_deref()
                        .is_some_and(|shape| own_shape_property(shape, name).is_none()),
                    Some(ObjectShapeProperty::Accessor { .. }) | None => false,
                }
            }
        }
    }
}

fn own_shape_property<'shape>(
    shape: &'shape HeapShape,
    name: &str,
) -> Option<&'shape ObjectShapeProperty> {
    match shape {
        HeapShape::Object(object) => object.properties.get(name),
        HeapShape::Array(array) => array.properties.get(name),
    }
}
