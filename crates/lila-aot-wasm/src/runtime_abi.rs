use lila_ir::ValueKind;

macro_rules! runtime_value_tags {
    ($($variant:ident = $wire:expr => $kind:ident;)+) => {
        /// Concrete value representations accepted at the current Wasm host boundary.
        ///
        /// Compiler analysis kinds cannot be embedded in this runtime domain.
        /// Each row fixes its wire tag and semantic kind together; a new row
        /// also requires both host decoders to handle the new representation.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(i32)]
        pub enum WasmRuntimeValueTag {
            $($variant = $wire,)+
        }

        impl WasmRuntimeValueTag {
            pub const fn tag(self) -> i32 {
                self as i32
            }

            pub const fn from_tag(tag: i32) -> Option<Self> {
                match tag {
                    $(wire if wire == Self::$variant as i32 => Some(Self::$variant),)+
                    _ => None,
                }
            }

            pub const fn value_kind(self) -> ValueKind {
                match self {
                    $(Self::$variant => ValueKind::$kind,)+
                }
            }
        }
    };
}

runtime_value_tags! {
    Undefined = ValueKind::Undefined.tag() => Undefined;
    Null = ValueKind::Null.tag() => Null;
    Boolean = ValueKind::Boolean.tag() => Boolean;
    Number = ValueKind::Number.tag() => Number;
    String = ValueKind::String.tag() => String;
    Symbol = ValueKind::Symbol.tag() => Symbol;
    Object = ValueKind::Object.tag() => Object;
    Array = ValueKind::Array.tag() => Array;
    Function = ValueKind::Function.tag() => Function;
    Arguments = ValueKind::Arguments.tag() => Arguments;
    BigInt = ValueKind::BigInt.tag() => BigInt;
}
