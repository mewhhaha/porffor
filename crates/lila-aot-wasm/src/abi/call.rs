//! The emitted call shapes that transport JavaScript completions.
//!
//! Each slot declares its Wasm type once. The type section, function-local
//! offsets, standard completion handling, and stack-guard wrappers consume
//! these declarations. A future GC-reference slot therefore changes a real
//! Wasm signature and the standard result bindings must account for it.

use wasm_encoder::ValType;

/// Wasm local containing a reference to the private GC argument vector.
/// Its index is an encoder operand, never an integer representation of the
/// reference itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArgVectorLocal(u32);

impl ArgVectorLocal {
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

macro_rules! call_slots {
    ($name:ident, $offset:expr; { $($slot:ident: $ty:expr),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum $name {
            $($slot),+
        }

        impl $name {
            pub(crate) const ALL: [Self; call_slots!(@count $($slot),+)] = [$(Self::$slot),+];

            pub(crate) const fn index(self) -> u32 {
                $offset + self as u32
            }

            pub(crate) const fn wasm_type(self) -> ValType {
                match self {
                    $(Self::$slot => $ty),+
                }
            }
        }
    };
    (@count $($slot:ident),+) => {
        <[()]>::len(&[$(call_slots!(@one $slot)),+])
    };
    (@one $slot:ident) => { () };
}

// Source functions and builtins take a native GC reference in the final slot.
// Seven-scalar runtime helpers have their own Raw signature below.
call_slots!(JsCallParameter, 0; {
    Environment: ValType::I64,
    ThisPayload: ValType::I64,
    ThisTag: ValType::I64,
    NewTargetPayload: ValType::I64,
    NewTargetTag: ValType::I64,
    Argc: ValType::I64,
    Argv: crate::gc_types::arg_vector::ref_type(),
});

call_slots!(PreparedScriptParameter, JsCallParameter::ALL.len() as u32; {
    VariableEnvironment: ValType::I64,
    PrivateEnvironment: ValType::I64,
    DirectEvalContext: ValType::I64,
});

call_slots!(CallResultSlot, 0; {
    Payload: ValType::I64,
    Tag: ValType::I64,
    Completion: ValType::I64,
    Aux: ValType::I64,
});

impl CallResultSlot {
    /// Local declaration for a body whose completion slots start at
    /// `first_result_local`. The caller's other locals remain scalar during
    /// this migration; changing a completion slot to a GC reference changes
    /// its actual Wasm local type here, not just its Rust name.
    pub(crate) fn declared_local_types(
        parameter_count: usize,
        first_result_local: u32,
        declared_local_count: usize,
    ) -> Vec<ValType> {
        let mut locals = vec![ValType::I64; declared_local_count];
        let first = first_result_local as usize;
        assert!(
            first >= parameter_count,
            "completion local overlaps call parameters"
        );
        let relative = first - parameter_count;
        assert!(
            relative + Self::ALL.len() <= declared_local_count,
            "completion locals exceed the function local declaration",
        );
        for slot in Self::ALL {
            locals[relative + slot.index() as usize] = slot.wasm_type();
        }
        locals
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallAbi {
    Js,
    Raw,
    Dispatch,
    PreparedScript,
}

impl CallAbi {
    pub(crate) const fn type_index(self) -> u32 {
        match self {
            Self::Js => 2,
            Self::Raw => 3,
            Self::Dispatch => 17,
            Self::PreparedScript => 18,
        }
    }

    pub(crate) const fn parameter_count(self) -> usize {
        match self {
            Self::Js | Self::Raw | Self::Dispatch => JsCallParameter::ALL.len(),
            Self::PreparedScript => JsCallParameter::ALL.len() + PreparedScriptParameter::ALL.len(),
        }
    }

    pub(crate) fn parameter_types(self) -> Vec<ValType> {
        match self {
            Self::Js => JsCallParameter::ALL
                .map(JsCallParameter::wasm_type)
                .to_vec(),
            Self::Raw => vec![ValType::I64; JsCallParameter::ALL.len()],
            Self::Dispatch => {
                let mut types = vec![ValType::I64; JsCallParameter::ALL.len()];
                types[5] = crate::gc_types::arg_vector::ref_type();
                types
            }
            Self::PreparedScript => {
                let mut types = JsCallParameter::ALL
                    .map(JsCallParameter::wasm_type)
                    .to_vec();
                types.extend(PreparedScriptParameter::ALL.map(PreparedScriptParameter::wasm_type));
                types
            }
        }
    }

    pub(crate) fn result_types(self) -> Vec<ValType> {
        match self {
            Self::Js | Self::Raw | Self::Dispatch | Self::PreparedScript => {
                CallResultSlot::ALL.map(CallResultSlot::wasm_type).to_vec()
            }
        }
    }
}
