//! Shared Arguments exotic lookup used by synchronous iterator consumers.
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_arguments_iterator_method_to_locals(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        method_payload: u32,
        method_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_name = "$sync.iterator.arguments.source";
        self.push_scope();
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(
                source_name.to_string(),
                BindingStorage::Dynamic {
                    tag_local: source_tag,
                    payload_local: source_payload,
                },
            );
        let source = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Arguments,
                possible_kinds: KindSet::from_kind(ValueKind::Arguments),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
            ExprIr::Identifier(source_name.to_string()),
        );
        let method = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            ExprIr::PropertyRead {
                target: Box::new(source),
                key: PropertyKeyIr::StaticString("Symbol.iterator".to_string()),
            },
        );
        self.compile_expr_to_locals(&method, method_payload, method_tag, function)?;
        self.pop_scope();
        Ok(())
    }
}
