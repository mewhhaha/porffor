pub(crate) const COMPLETION_KIND_NORMAL: i64 = 0;
pub(crate) const COMPLETION_KIND_THROW: i64 = 1;
pub(crate) const COMPLETION_KIND_RETURN: i64 = 2;
pub(crate) const COMPLETION_KIND_BREAK: i64 = 3;
pub(crate) const COMPLETION_KIND_CONTINUE: i64 = 4;
pub(crate) const COMPLETION_KIND_EMPTY: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompletionKindSlot {
    pub name: &'static str,
    pub value: i64,
}

pub(crate) const COMPLETION_KIND_REGISTRY: &[CompletionKindSlot] = &[
    CompletionKindSlot {
        name: "normal",
        value: COMPLETION_KIND_NORMAL,
    },
    CompletionKindSlot {
        name: "throw",
        value: COMPLETION_KIND_THROW,
    },
    CompletionKindSlot {
        name: "return",
        value: COMPLETION_KIND_RETURN,
    },
    CompletionKindSlot {
        name: "break",
        value: COMPLETION_KIND_BREAK,
    },
    CompletionKindSlot {
        name: "continue",
        value: COMPLETION_KIND_CONTINUE,
    },
    CompletionKindSlot {
        name: "empty",
        value: COMPLETION_KIND_EMPTY,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations_completion_kind_registry_is_stable_and_dense() {
        for (expected, slot) in COMPLETION_KIND_REGISTRY.iter().enumerate() {
            assert_eq!(
                slot.value, expected as i64,
                "completion kind {} should stay at {}",
                slot.name, expected
            );
        }
    }

    #[test]
    fn operations_completion_kind_registry_names_t04_variants() {
        let names = COMPLETION_KIND_REGISTRY
            .iter()
            .map(|slot| slot.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["normal", "throw", "return", "break", "continue", "empty"]
        );
    }
}
