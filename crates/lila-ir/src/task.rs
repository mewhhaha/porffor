//! Closed identifiers for the repository task ledger.
//!
//! Task identifiers cross compiler reports and conformance ownership data, so
//! their text is a stable interface. The enum, complete registry, parser and
//! formatter are generated from one list: adding a task cannot update one
//! representation while forgetting another.

use std::{error::Error, fmt, str::FromStr};

macro_rules! define_task_ids {
    ($($variant:ident = $number:literal => $text:literal),+ $(,)?) => {
        /// A task in the repository's `tasks/` ledger.
        ///
        /// The public variants are the constructors. Consequently, code that
        /// holds a `TaskId` cannot hold a syntactically valid but nonexistent
        /// id such as `T30`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum TaskId {
            $($variant = $number),+
        }

        impl TaskId {
            /// Every task, in ledger order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The stable ledger and report representation.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            /// The numeric part of the task id.
            pub const fn number(self) -> u8 {
                self as u8
            }
        }
    };
}

define_task_ids! {
    T00 = 0 => "T00",
    T01 = 1 => "T01",
    T02 = 2 => "T02",
    T03 = 3 => "T03",
    T04 = 4 => "T04",
    T05 = 5 => "T05",
    T06 = 6 => "T06",
    T07 = 7 => "T07",
    T08 = 8 => "T08",
    T09 = 9 => "T09",
    T10 = 10 => "T10",
    T11 = 11 => "T11",
    T12 = 12 => "T12",
    T13 = 13 => "T13",
    T14 = 14 => "T14",
    T15 = 15 => "T15",
    T16 = 16 => "T16",
    T17 = 17 => "T17",
    T18 = 18 => "T18",
    T19 = 19 => "T19",
    T20 = 20 => "T20",
    T21 = 21 => "T21",
    T22 = 22 => "T22",
    T23 = 23 => "T23",
    T24 = 24 => "T24",
    T25 = 25 => "T25",
    T26 = 26 => "T26",
    T27 = 27 => "T27",
    T28 = 28 => "T28",
    T29 = 29 => "T29",
}

impl fmt::Display for TaskId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for TaskId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<TaskId> for &'static str {
    fn from(task: TaskId) -> Self {
        task.as_str()
    }
}

/// An external string did not name a task in the closed ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTaskIdError {
    invalid: String,
}

impl ParseTaskIdError {
    pub fn invalid(&self) -> &str {
        &self.invalid
    }
}

impl fmt::Display for ParseTaskIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown task id {:?}; expected T00 through T29",
            self.invalid
        )
    }
}

impl Error for ParseTaskIdError {}

impl FromStr for TaskId {
    type Err = ParseTaskIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        for task in Self::ALL {
            if task.as_str() == text {
                return Ok(*task);
            }
        }

        Err(ParseTaskIdError {
            invalid: text.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_ids_have_stable_text_and_numeric_order() {
        assert_eq!(TaskId::ALL.len(), 30);
        let expected = (0..=29).map(|number| format!("T{number:02}"));

        for (number, (task, expected)) in TaskId::ALL.iter().zip(expected).enumerate() {
            assert_eq!(task.number(), number as u8);
            assert_eq!(task.as_str(), expected);
            assert_eq!(task.to_string(), expected);
            assert_eq!(expected.parse::<TaskId>(), Ok(*task));
        }
    }

    #[test]
    fn task_id_parser_rejects_non_tasks_and_classification_buckets() {
        // `T26-unclassified` is a Test262 ownership fallback bucket. It is not
        // a task and must not become valid merely because it starts with one.
        for invalid in ["T30", "T4", "T4x", "T26-unclassified"] {
            let error = invalid.parse::<TaskId>().unwrap_err();
            assert_eq!(error.invalid(), invalid);
        }
    }
}
