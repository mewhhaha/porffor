//! The closed ECMA-262 10.5 Proxy trap domain used by lowering.
//!
//! Property names enter as strings only at the object-shape boundary. Past
//! that boundary, every consumer handles a closed trap and its semantic
//! argument record exhaustively.

/// The eight distinct argument records used by the thirteen Proxy traps.
///
/// This is deliberately semantic rather than an arity: three arguments can
/// mean `(target, key, receiver)`, `(target, key, descriptor)`,
/// `(target, thisArgument, argumentsList)`, or `(target, argumentsList,
/// newTarget)`, and treating those as interchangeable is exactly the mistake
/// this seam prevents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProxyTrapSignature {
    Target,
    TargetAndPropertyKey,
    TargetPropertyKeyReceiver,
    TargetPropertyKeyValueReceiver,
    TargetPropertyKeyDescriptor,
    TargetAndPrototype,
    TargetThisArguments,
    TargetArgumentsNewTarget,
}

/// Declare every member of the fixed Proxy trap domain once.
///
/// `ALL`, the JavaScript property-name boundary, the semantic signature and
/// the deliberately narrower generic-object heuristic are generated from the
/// same rows. The declared count makes adding or removing a row without
/// acknowledging the thirteen-name contract a type error.
macro_rules! proxy_traps {
    (
        $count:literal;
        $(
            $variant:ident => ($name:literal, $signature:ident, $generic_hint:literal)
        ),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) enum ProxyTrap {
            $($variant),+
        }

        impl ProxyTrap {
            pub(super) const ALL: [Self; $count] = [$(Self::$variant),+];

            pub(super) fn from_property_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Self::$variant),)+
                    _ => None,
                }
            }

            pub(super) const fn property_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            pub(super) const fn signature(self) -> ProxyTrapSignature {
                match self {
                    $(Self::$variant => ProxyTrapSignature::$signature),+
                }
            }

            /// The generic object-literal heuristic is intentionally narrower
            /// than the spec domain: these observations are not proof that an
            /// arbitrary object will be used as a Proxy handler.
            pub(super) const fn has_conservative_object_literal_hint(self) -> bool {
                match self {
                    $(Self::$variant => $generic_hint),+
                }
            }
        }
    };
}

proxy_traps! {
    13;
    GetPrototypeOf => ("getPrototypeOf", Target, false),
    SetPrototypeOf => ("setPrototypeOf", TargetAndPrototype, false),
    IsExtensible => ("isExtensible", Target, false),
    PreventExtensions => ("preventExtensions", Target, false),
    GetOwnPropertyDescriptor => ("getOwnPropertyDescriptor", TargetAndPropertyKey, true),
    DefineProperty => ("defineProperty", TargetPropertyKeyDescriptor, true),
    Has => ("has", TargetAndPropertyKey, true),
    Get => ("get", TargetPropertyKeyReceiver, true),
    Set => ("set", TargetPropertyKeyValueReceiver, false),
    DeleteProperty => ("deleteProperty", TargetAndPropertyKey, true),
    OwnKeys => ("ownKeys", Target, false),
    Apply => ("apply", TargetThisArguments, false),
    Construct => ("construct", TargetArgumentsNewTarget, false),
}
