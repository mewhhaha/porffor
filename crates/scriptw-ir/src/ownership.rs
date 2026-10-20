//! Closed conformance areas shared by compiler diagnostics and failure reports.
//!
//! Owner identifiers cross compiler reports and conformance ownership data, so
//! their text is a stable interface. The enum, complete registry, parser and
//! formatter are generated from one list: adding an area cannot update one
//! representation while forgetting another.

use std::{error::Error, fmt, str::FromStr};

macro_rules! define_conformance_owners {
    ($($variant:ident = $number:literal => $text:literal => $area:literal),+ $(,)?) => {
        /// A permanent compiler or conformance ownership area.
        ///
        /// The public variants are the constructors. Consequently, code that
        /// holds a `ConformanceOwner` cannot hold a syntactically valid but nonexistent
        /// id such as `T30`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum ConformanceOwner {
            $($variant = $number),+
        }

        impl ConformanceOwner {
            /// Every conformance area, in stable report order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The stable report representation.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            /// The durable technical owner of this report code.
            pub const fn area(self) -> &'static str {
                match self {
                    $(Self::$variant => $area),+
                }
            }

            /// The numeric part of the owner id.
            pub const fn number(self) -> u8 {
                self as u8
            }
        }
    };
}

define_conformance_owners! {
    T00 = 0 => "T00" => "Contribution protocol",
    T01 = 1 => "T01" => "Baseline and failure evidence",
    T02 = 2 => "T02" => "IR and Wasm backend architecture",
    T03 = 3 => "T03" => "Test262 harness integrity and host contract",
    T04 = 4 => "T04" => "Shared ECMAScript operations and completion ABI",
    T05 = 5 => "T05" => "Value representation, heap, GC and weak reachability",
    T06 = 6 => "T06" => "Realms, intrinsics and cross-realm semantics",
    T07 = 7 => "T07" => "Parser boundary, grammar coverage and early errors",
    T08 = 8 => "T08" => "Environments, references, control flow and abrupt completion",
    T09 = 9 => "T09" => "Functions, constructors, classes and private elements",
    T10 = 10 => "T10" => "Object model, descriptors and exotic-object protocol",
    T11 = 11 => "T11" => "Proxy and Reflect meta-object protocol",
    T12 = 12 => "T12" => "Modules, linking, loading and namespace objects",
    T13 = 13 => "T13" => "Dynamic source evaluation: `eval`, `Function` and realm evaluation",
    T14 = 14 => "T14" => "Promise jobs, async functions and async iteration",
    T15 = 15 => "T15" => "Generators, iterators, iterator helpers and resource management",
    T16 = 16 => "T16" => "Array exotic semantics and complete Array API",
    T17 = 17 => "T17" => "ArrayBuffer, DataView, TypedArray, SharedArrayBuffer and Atomics",
    T18 = 18 => "T18" => "Strings, Unicode and the complete String API",
    T19 = 19 => "T19" => "Complete ECMAScript RegExp semantics",
    T20 = 20 => "T20" => "Number, BigInt, Math and JSON",
    T21 = 21 => "T21" => "Symbols, collections, weak collections and weak references",
    T22 = 22 => "T22" => "Date and Temporal",
    T23 = 23 => "T23" => "Complete ECMA-402 Intl implementation",
    T24 = 24 => "T24" => "Globals, native errors, Annex B and remaining host-visible builtins",
    T25 = 25 => "T25" => "Differential testing, fuzzing and performance discipline",
    T26 = 26 => "T26" => "Conformance release gate",
    T27 = 27 => "T27" => "Interpreter quarantine and Wasm-AOT product default",
    T28 = 28 => "T28" => "Legacy retirement boundary",
    T29 = 29 => "T29" => "Canonical product identity",
}

impl fmt::Display for ConformanceOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for ConformanceOwner {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<ConformanceOwner> for &'static str {
    fn from(task: ConformanceOwner) -> Self {
        task.as_str()
    }
}

/// An external string did not name an area in the closed ownership registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseConformanceOwnerError {
    invalid: String,
}

impl ParseConformanceOwnerError {
    pub fn invalid(&self) -> &str {
        &self.invalid
    }
}

impl fmt::Display for ParseConformanceOwnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown conformance owner {:?}; expected T00 through T29",
            self.invalid
        )
    }
}

impl Error for ParseConformanceOwnerError {}

impl FromStr for ConformanceOwner {
    type Err = ParseConformanceOwnerError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        for task in Self::ALL {
            if task.as_str() == text {
                return Ok(*task);
            }
        }

        Err(ParseConformanceOwnerError {
            invalid: text.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owners_have_stable_text_and_numeric_order() {
        assert_eq!(ConformanceOwner::ALL.len(), 30);
        let expected = (0..=29).map(|number| format!("T{number:02}"));

        for (number, (task, expected)) in ConformanceOwner::ALL.iter().zip(expected).enumerate() {
            assert_eq!(task.number(), number as u8);
            assert_eq!(task.as_str(), expected);
            assert_eq!(task.to_string(), expected);
            assert_eq!(expected.parse::<ConformanceOwner>(), Ok(*task));
        }
    }

    #[test]
    fn owner_parser_rejects_unknown_areas_and_classification_buckets() {
        // `T26-unclassified` is a Test262 ownership fallback bucket. It is not
        // an area and must not become valid merely because it starts with one.
        for invalid in ["T30", "T4", "T4x", "T26-unclassified"] {
            let error = invalid.parse::<ConformanceOwner>().unwrap_err();
            assert_eq!(error.invalid(), invalid);
        }
    }
}
