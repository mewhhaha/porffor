use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Declaration order is the domain/report order. Wire spellings occur once and
// drive both serde and text; persisted count maps retain lexical key ordering
// through the snapshot serializer in the parent module.
macro_rules! taxonomy {
    ($name:ident { $($(#[$meta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum $name {
            $($(#[$meta])* #[serde(rename = $wire)] $variant),+
        }

        impl $name {
            pub const ALL: [Self; [$(stringify!($variant)),+].len()] = [$(Self::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }

            /// Zero-fill the closed domain without implying that the source
            /// run is complete. Completeness is owned by snapshot admission.
            pub fn ordered_counts(counts: &BTreeMap<Self, usize>) -> impl Iterator<Item = (Self, usize)> + '_ {
                Self::ALL.into_iter().map(|kind| (kind, counts.get(&kind).copied().unwrap_or(0)))
            }
        }

        impl super::TaxonomyWireLabel for $name {
            fn wire_label(&self) -> &'static str { self.as_str() }
        }
    };
}

taxonomy!(FailureKind {
    Parser => "Parser",
    EarlyError => "EarlyError",
    Lowering => "Lowering",
    Runtime => "Runtime",
    WasmBackend => "WasmBackend",
    HostHarness => "HostHarness",
    Unsupported => "Unsupported",
});

taxonomy!(OutcomeKind {
    Success => "Success",
    NotImplemented => "NotImplemented",
    Crash => "Crash",
    Bug => "Bug",
});

taxonomy!(FailureOrigin {
    Unknown => "unknown",
    LocalHarness => "local-harness",
    BoaRuntime => "boa-runtime",
    BoaParser => "boa-parser",
    IcuIntl => "icu-intl",
    SpecExecHost => "spec-exec-host",
    /// Wasmtime engine/capability rejection of emitted Wasm.
    WasmBackend => "wasm-backend",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_wire_domains_retain_their_original_order_and_spelling() {
        fn check<T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq>(
            values: &[T],
            expected: &[&str],
        ) {
            for (value, wire) in values.iter().zip(expected) {
                let json = serde_json::to_string(wire).unwrap();
                assert_eq!(serde_json::to_string(value).unwrap(), json);
                assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value);
            }
            assert_eq!(values.len(), expected.len());
            assert!(serde_json::from_str::<T>("\"unexpected\"").is_err());
        }
        check(
            &FailureKind::ALL,
            &[
                "Parser",
                "EarlyError",
                "Lowering",
                "Runtime",
                "WasmBackend",
                "HostHarness",
                "Unsupported",
            ],
        );
        check(
            &OutcomeKind::ALL,
            &["Success", "NotImplemented", "Crash", "Bug"],
        );
        check(
            &FailureOrigin::ALL,
            &[
                "unknown",
                "local-harness",
                "boa-runtime",
                "boa-parser",
                "icu-intl",
                "spec-exec-host",
                "wasm-backend",
            ],
        );
    }
}
