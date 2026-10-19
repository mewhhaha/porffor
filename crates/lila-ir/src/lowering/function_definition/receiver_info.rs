use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn receiver_info_for_function(
        &self,
        function: &FunctionPlan<'a>,
        exact_context_signature: Option<&FunctionSignature>,
    ) -> ValueInfo {
        // Only receiver facts are consumed here. A full signature clone
        // also copies unused parameter, return and global-object trees.
        let current_flow_signature =
            exact_context_signature.or_else(|| self.function_signatures.get(&function.id));
        let this_info = current_flow_signature
            .map(|signature| {
                if signature.this_observed {
                    ValueInfo {
                        kind: signature.this_info.kind,
                        possible_kinds: signature.this_info.possible_kinds,
                        heap_shape: if self.function_signature_shape_evidence
                            == FunctionSignatureShapeEvidence::Available
                        {
                            signature.this_info.heap_shape.clone()
                        } else {
                            None
                        },
                        function_targets: signature.this_info.function_targets.clone(),
                    }
                } else if function.strict {
                    ValueInfo::undefined()
                } else {
                    Self::unobserved_sloppy_this_info()
                }
            })
            .unwrap_or_else(ValueInfo::undefined);
        #[cfg(test)]
        {
            let previous = exact_context_signature
                .cloned()
                .or_else(|| self.function_signature_for_current_flow(&function.id))
                .map(|signature| {
                    if signature.this_observed {
                        signature.this_info
                    } else if function.strict {
                        ValueInfo::undefined()
                    } else {
                        Self::unobserved_sloppy_this_info()
                    }
                })
                .unwrap_or_else(ValueInfo::undefined);
            assert_eq!(this_info, previous, "receiver facts for {}", function.id);
        }
        this_info
    }
}
