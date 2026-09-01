use lila_ir::CompletionKindIr;

pub(crate) const COMPLETION_KIND_NORMAL: i64 = CompletionKindIr::Normal.abi_code();
pub(crate) const COMPLETION_KIND_THROW: i64 = CompletionKindIr::Throw.abi_code();
pub(crate) const COMPLETION_KIND_RETURN: i64 = CompletionKindIr::Return.abi_code();
pub(crate) const COMPLETION_KIND_BREAK: i64 = CompletionKindIr::Break.abi_code();
pub(crate) const COMPLETION_KIND_CONTINUE: i64 = CompletionKindIr::Continue.abi_code();

pub(crate) const COMPLETION_KIND_REGISTRY: &[CompletionKindIr] = CompletionKindIr::ALL;

#[cfg(test)]
mod tests {
    use super::*;
    use lila_ir::COMPLETION_ABI_SLOTS;

    #[test]
    fn operations_completion_kind_registry_is_stable_and_dense() {
        for (expected, kind) in COMPLETION_KIND_REGISTRY.iter().enumerate() {
            assert_eq!(
                kind.abi_code(),
                expected as i64,
                "completion kind {} should stay at {}",
                kind.name(),
                expected
            );
        }
    }

    #[test]
    fn operations_completion_kind_registry_names_t04_variants() {
        let names = COMPLETION_KIND_REGISTRY
            .iter()
            .map(|kind| kind.name())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["normal", "throw", "return", "break", "continue", "empty"]
        );
    }

    #[test]
    fn operations_completion_kind_registry_matches_ir_abi() {
        assert_eq!(COMPLETION_KIND_REGISTRY.len(), COMPLETION_ABI_SLOTS.len());
        for (backend, ir) in COMPLETION_KIND_REGISTRY
            .iter()
            .zip(COMPLETION_ABI_SLOTS.iter())
        {
            assert_eq!(*backend, ir.kind());
            assert_eq!(backend.name(), ir.name());
            assert_eq!(backend.abi_code(), ir.code());
        }
    }
}
