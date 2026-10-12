//! Exact-context selection and current-flow parameter projection.
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn exact_signature_for_function(
        &self,
        function_id: &FunctionId,
        context_key_override: Option<&ExactCallbackContextKey>,
    ) -> Option<FunctionSignature> {
        let result = self
            .exact_signature_ref_for_function(function_id, context_key_override)
            .cloned()
            .map(|signature| self.function_signature_with_current_flow_evidence(signature));
        #[cfg(test)]
        assert_eq!(
            result,
            self.exact_signature_via_collected_observations(function_id, context_key_override),
            "exact signature selection for {function_id}"
        );
        result
    }

    fn exact_signature_ref_for_function(
        &self,
        function_id: &FunctionId,
        context_key_override: Option<&ExactCallbackContextKey>,
    ) -> Option<&FunctionSignature> {
        if self.function_has_untracked_captures(function_id) {
            return None;
        }
        if let Some(context_key) = context_key_override {
            return self
                .exact_context_function_observations
                .get(context_key)
                .or_else(|| self.exact_context_callback_observations.get(context_key));
        }
        let mut function_observations = self.exact_context_function_observations.iter().filter_map(
            |((observed_function_id, _), signature)| {
                (observed_function_id == function_id).then_some(signature)
            },
        );
        if let (Some(exact), None) = (function_observations.next(), function_observations.next()) {
            // A unique but uncovered function context blocks callback fallback,
            // exactly as before. Observe cardinality without cloning shapes.
            return self
                .exact_signature_covers_aggregate_inputs(function_id, exact)
                .then_some(exact);
        }
        let mut callback_observations = self.exact_context_callback_observations.iter().filter_map(
            |((callback_id, _), signature)| (callback_id == function_id).then_some(signature),
        );
        if let (Some(exact), None) = (callback_observations.next(), callback_observations.next()) {
            self.exact_signature_covers_aggregate_inputs(function_id, exact)
                .then_some(exact)
        } else {
            None
        }
    }

    #[cfg(test)]
    fn exact_signature_via_collected_observations(
        &self,
        function_id: &FunctionId,
        context_key_override: Option<&ExactCallbackContextKey>,
    ) -> Option<FunctionSignature> {
        if self.function_has_untracked_captures(function_id) {
            return None;
        }
        if let Some(context_key) = context_key_override {
            return self
                .exact_context_function_observations
                .get(context_key)
                .or_else(|| self.exact_context_callback_observations.get(context_key))
                .cloned()
                .map(|signature| self.function_signature_with_current_flow_evidence(signature));
        }
        let mut function_observations = self
            .exact_context_function_observations
            .iter()
            .filter_map(|((observed_function_id, _), signature)| {
                (observed_function_id == function_id).then(|| signature.clone())
            })
            .collect::<Vec<_>>();
        if function_observations.len() == 1 {
            let exact = function_observations.pop().unwrap();
            if self.exact_signature_covers_aggregate_inputs(function_id, &exact) {
                return Some(self.function_signature_with_current_flow_evidence(exact));
            }
            return None;
        }
        let mut callback_observations = self
            .exact_context_callback_observations
            .iter()
            .filter_map(|((callback_id, _), signature)| {
                (callback_id == function_id).then(|| signature.clone())
            })
            .collect::<Vec<_>>();
        if callback_observations.len() == 1 {
            let exact = callback_observations.pop().unwrap();
            self.exact_signature_covers_aggregate_inputs(function_id, &exact)
                .then_some(exact)
                .map(|signature| self.function_signature_with_current_flow_evidence(signature))
        } else {
            None
        }
    }

    pub(super) fn function_parameter_info_for_current_flow(
        &self,
        function_id: &FunctionId,
        context_key_override: Option<&ExactCallbackContextKey>,
        parameter: usize,
    ) -> Option<ValueInfo> {
        // Re-resolve on each parameter: preceding default initializers may
        // change flow evidence. Clone only the chosen parameter's facts.
        let signature = self
            .exact_signature_ref_for_function(function_id, context_key_override)
            .or_else(|| self.function_signatures.get(function_id))?;
        let param = signature.params.get(parameter)?;
        let info = ValueInfo {
            kind: param.possible_kinds.as_value_kind(),
            possible_kinds: param.possible_kinds,
            heap_shape: if self.function_signature_shape_evidence
                == FunctionSignatureShapeEvidence::Available
            {
                param.heap_shape.clone()
            } else {
                None
            },
            function_targets: param.function_targets.clone(),
        };
        #[cfg(test)]
        {
            let previous = self
                .exact_signature_for_function(function_id, context_key_override)
                .or_else(|| self.function_signature_for_current_flow(function_id))
                .and_then(|signature| signature.params.get(parameter).cloned())
                .map(|param| ValueInfo {
                    kind: param.possible_kinds.as_value_kind(),
                    possible_kinds: param.possible_kinds,
                    heap_shape: param.heap_shape,
                    function_targets: param.function_targets,
                });
            assert_eq!(
                Some(info.clone()),
                previous,
                "parameter facts for {function_id}/{parameter}"
            );
        }
        Some(info)
    }
}
