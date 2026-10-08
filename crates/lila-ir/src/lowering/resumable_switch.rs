//! The actual CaseBlock, selector effects and fallthrough algorithm is shared.
use super::*;
use crate::async_generator_source::{
    AsyncGeneratorScopedResourceSourceStates, AsyncGeneratorSourceRange,
    AsyncGeneratorSwitchSourceStates,
};
use crate::generator_loop_control::GeneratorLoopSourceRange;
use crate::lowering_helpers::GeneratorSwitchSourceStates;

// The private protocol keeps ordinary, async and mixed source ranges paired
// with their actual regions and cases.
pub(super) trait ResumableSwitchProtocol: sealed::Sealed {
    type States;
    type Range: Copy;
    type Region;
    type Expression;
    type Case;
    const DISCRIMINANT_HINT: &'static str;
    const VALUE_HINT: &'static str;
    const INVALID_DISCRIMINANT: &'static str;
    const NO_DISCRIMINANT_VALUE: &'static str;
    const INVALID_PLAN: &'static str;
    fn discriminant(states: &Self::States) -> Self::Range;
    fn selectors(states: &Self::States) -> &[Option<Self::Range>];
    fn bodies(states: &Self::States) -> &[Self::Range];
    fn exit(states: &Self::States) -> u32;
    fn entry(range: Self::Range) -> u32;
    fn set_phase(lowerer: &mut ScriptLowerer<'_>, state: u32);
    fn lower_value(
        lowerer: &mut ScriptLowerer<'_>,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)>;
    fn finish_region(
        lowerer: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
    ) -> Option<Self::Region>;
    fn expression(region: Self::Region, value: TypedExpr) -> Self::Expression;
    fn case(selector: Option<Self::Expression>, body: Self::Region) -> Option<Self::Case>;
    fn enter_body(lowerer: &mut ScriptLowerer<'_>);
    fn leave_body(lowerer: &mut ScriptLowerer<'_>);
    fn resource_source(states: &Self::States) -> Option<&AsyncGeneratorScopedResourceSourceStates>;
    fn lower_body_items(
        lowerer: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
        scope: &mut LexicalScopeInstantiation,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<BlockIr> {
        if resource.is_some() {
            return None;
        }
        Some(lowerer.lower_statement_items_without_function_initialization(items, scope))
    }
    fn finish_body_region(
        lowerer: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<Self::Region> {
        if resource.is_some() {
            return None;
        }
        Self::finish_region(lowerer, block, range)
    }
    fn resource_owner(
        states: &Self::States,
        capability: Option<OwnedEnvBindingIr>,
        cases: &[Self::Case],
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<Option<AsyncGeneratorScopedResourceIr>>;
    fn plan(
        states: Self::States,
        discriminant: Self::Expression,
        discriminant_binding: OwnedEnvBindingIr,
        environment: Option<LexicalEnvironmentIr>,
        declarations: Vec<StatementIr>,
        cases: Vec<Self::Case>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<StatementIr>;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::OrdinarySwitch {}
    impl Sealed for super::MixedSwitch {}
    impl<const ASYNC: bool> Sealed for super::CompleteSwitch<ASYNC> {}
}

pub(super) struct OrdinarySwitch;
impl ResumableSwitchProtocol for OrdinarySwitch {
    type States = GeneratorSwitchSourceStates;
    type Range = GeneratorLoopSourceRange;
    type Region = GeneratorLoopRegionIr;
    type Expression = GeneratorLoopExpressionIr;
    type Case = OrdinaryGeneratorSwitchCaseIr;
    const DISCRIMINANT_HINT: &'static str = "generator.switch.discriminant.";
    const VALUE_HINT: &'static str = "generator.switch.value.";
    const INVALID_DISCRIMINANT: &'static str =
        "ordinary generator Switch discriminant differs from its source region";
    const NO_DISCRIMINANT_VALUE: &'static str =
        "ordinary generator Switch discriminant has no value owner";
    const INVALID_PLAN: &'static str =
        "ordinary generator Switch differs from its consumed source plan";
    fn discriminant(s: &Self::States) -> Self::Range {
        s.discriminant()
    }
    fn selectors(s: &Self::States) -> &[Option<Self::Range>] {
        s.selectors()
    }
    fn bodies(s: &Self::States) -> &[Self::Range] {
        s.bodies()
    }
    fn exit(s: &Self::States) -> u32 {
        s.exit()
    }
    fn entry(r: Self::Range) -> u32 {
        r.entry
    }
    fn set_phase(l: &mut ScriptLowerer<'_>, state: u32) {
        l.current_generator_resume_state = Some(state);
    }
    fn lower_value(
        l: &mut ScriptLowerer<'_>,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        l.lower_staged_generator_expression(source)
    }
    fn finish_region(
        l: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
    ) -> Option<Self::Region> {
        (l.plain_generator_entry_state()? == range.end).then_some(())?;
        GeneratorLoopRegionIr::new(block, range).ok()
    }
    fn expression(region: Self::Region, value: TypedExpr) -> Self::Expression {
        GeneratorLoopExpressionIr::new(region, value)
    }
    fn case(selector: Option<Self::Expression>, body: Self::Region) -> Option<Self::Case> {
        OrdinaryGeneratorSwitchCaseIr::new(selector, body).ok()
    }
    fn enter_body(l: &mut ScriptLowerer<'_>) {
        l.ordinary_generator_region_depth += 1;
        l.ordinary_generator_switch_depth += 1;
    }
    fn leave_body(l: &mut ScriptLowerer<'_>) {
        l.ordinary_generator_switch_depth -= 1;
        l.ordinary_generator_region_depth -= 1;
    }
    fn resource_source(_: &Self::States) -> Option<&AsyncGeneratorScopedResourceSourceStates> {
        None
    }
    fn resource_owner(
        _: &Self::States,
        capability: Option<OwnedEnvBindingIr>,
        _: &[Self::Case],
        _: &[OwnedEnvBindingIr],
    ) -> Option<Option<AsyncGeneratorScopedResourceIr>> {
        capability.is_none().then_some(None)
    }
    fn plan(
        states: Self::States,
        discriminant: Self::Expression,
        discriminant_binding: OwnedEnvBindingIr,
        environment: Option<LexicalEnvironmentIr>,
        declarations: Vec<StatementIr>,
        cases: Vec<Self::Case>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<StatementIr> {
        resource.is_none().then_some(())?;
        OrdinaryGeneratorSwitchIr::new(
            states,
            discriminant,
            discriminant_binding,
            environment,
            declarations,
            cases,
            value_binding,
            inventory,
        )
        .ok()
        .map(|plan| StatementIr::OrdinaryGeneratorSwitch(Box::new(plan)))
    }
}

pub(super) struct MixedSwitch;
impl ResumableSwitchProtocol for MixedSwitch {
    type States = AsyncGeneratorSwitchSourceStates;
    type Range = AsyncGeneratorSourceRange;
    type Region = AsyncGeneratorLoopRegionIr;
    type Expression = AsyncGeneratorLoopExpressionIr;
    type Case = AsyncGeneratorSwitchCaseIr;
    const DISCRIMINANT_HINT: &'static str = "async.generator.switch.discriminant.";
    const VALUE_HINT: &'static str = "async.generator.switch.value.";
    const INVALID_DISCRIMINANT: &'static str =
        "async-generator Switch discriminant differs from its source region";
    const NO_DISCRIMINANT_VALUE: &'static str =
        "async-generator Switch discriminant has no value owner";
    const INVALID_PLAN: &'static str =
        "async-generator Switch differs from its consumed source plan";
    fn discriminant(s: &Self::States) -> Self::Range {
        s.discriminant()
    }
    fn selectors(s: &Self::States) -> &[Option<Self::Range>] {
        s.selectors()
    }
    fn bodies(s: &Self::States) -> &[Self::Range] {
        s.bodies()
    }
    fn exit(s: &Self::States) -> u32 {
        s.exit()
    }
    fn entry(r: Self::Range) -> u32 {
        r.entry()
    }
    fn set_phase(l: &mut ScriptLowerer<'_>, state: u32) {
        l.set_async_generator_phase(state);
    }
    fn lower_value(
        l: &mut ScriptLowerer<'_>,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        l.lower_mixed_generator_value(source)
    }
    fn finish_region(
        l: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
    ) -> Option<Self::Region> {
        l.finish_async_generator_region(block, range).ok()
    }
    fn expression(region: Self::Region, value: TypedExpr) -> Self::Expression {
        AsyncGeneratorLoopExpressionIr::new(region, value)
    }
    fn case(selector: Option<Self::Expression>, body: Self::Region) -> Option<Self::Case> {
        AsyncGeneratorSwitchCaseIr::new(selector, body).ok()
    }
    fn enter_body(l: &mut ScriptLowerer<'_>) {
        l.mixed_async_generator_region_depth += 1;
    }
    fn leave_body(l: &mut ScriptLowerer<'_>) {
        l.mixed_async_generator_region_depth -= 1;
    }
    fn resource_source(states: &Self::States) -> Option<&AsyncGeneratorScopedResourceSourceStates> {
        states.resource()
    }
    fn lower_body_items(
        lowerer: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
        scope: &mut LexicalScopeInstantiation,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<BlockIr> {
        match resource {
            Some((states, capability)) => {
                lowerer.lower_mixed_case_resource_items(items, scope, states, capability)
            }
            None => {
                Some(lowerer.lower_statement_items_without_function_initialization(items, scope))
            }
        }
    }
    fn finish_body_region(
        lowerer: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<Self::Region> {
        match resource {
            Some((states, capability)) => {
                lowerer.finish_mixed_resource_region(states, range, capability, block)
            }
            None => Self::finish_region(lowerer, block, range),
        }
    }
    fn resource_owner(
        states: &Self::States,
        capability: Option<OwnedEnvBindingIr>,
        cases: &[Self::Case],
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<Option<AsyncGeneratorScopedResourceIr>> {
        match (states.resource(), capability) {
            (Some(states), Some(capability)) => {
                let regions = cases
                    .iter()
                    .filter_map(|case| case.selector().map(|selector| selector.region()))
                    .chain(cases.iter().map(|case| case.body()))
                    .collect::<Vec<_>>();
                Some(Some(
                    AsyncGeneratorScopedResourceIr::new_resumable(
                        states, capability, &regions, inventory,
                    )
                    .ok()?,
                ))
            }
            (None, None) => Some(None),
            (Some(_), None) | (None, Some(_)) => None,
        }
    }
    fn plan(
        states: Self::States,
        discriminant: Self::Expression,
        discriminant_binding: OwnedEnvBindingIr,
        environment: Option<LexicalEnvironmentIr>,
        declarations: Vec<StatementIr>,
        cases: Vec<Self::Case>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<StatementIr> {
        AsyncGeneratorSwitchIr::new(
            states,
            discriminant,
            discriminant_binding,
            environment,
            declarations,
            cases,
            value_binding,
            resource,
            inventory,
        )
        .ok()
        .map(|plan| StatementIr::AsyncGeneratorSwitch(Box::new(plan)))
    }
}

pub(super) struct CompleteSwitch<const ASYNC: bool>;
impl<const ASYNC: bool> CompleteSwitch<ASYNC> {
    fn execution() -> ResumableRegionProtocolIr {
        if ASYNC {
            ResumableRegionProtocolIr::Async
        } else {
            ResumableRegionProtocolIr::Generator
        }
    }
}
impl<const ASYNC: bool> ResumableSwitchProtocol for CompleteSwitch<ASYNC> {
    type States = AsyncGeneratorSwitchSourceStates;
    type Range = AsyncGeneratorSourceRange;
    type Region = ResumableRegionIr;
    type Expression = ResumableExpressionIr;
    type Case = AsyncGeneratorSwitchCaseIr;
    const DISCRIMINANT_HINT: &'static str = "complete.switch.discriminant.";
    const VALUE_HINT: &'static str = "complete.switch.value.";
    const INVALID_DISCRIMINANT: &'static str =
        "complete Switch discriminant differs from its checked protocol region";
    const NO_DISCRIMINANT_VALUE: &'static str =
        "complete Switch discriminant has no retained value";
    const INVALID_PLAN: &'static str =
        "complete CaseBlock differs from its checked source lifetime";
    fn discriminant(s: &Self::States) -> Self::Range {
        s.discriminant()
    }
    fn selectors(s: &Self::States) -> &[Option<Self::Range>] {
        s.selectors()
    }
    fn bodies(s: &Self::States) -> &[Self::Range] {
        s.bodies()
    }
    fn exit(s: &Self::States) -> u32 {
        s.exit()
    }
    fn entry(r: Self::Range) -> u32 {
        r.entry()
    }
    fn set_phase(l: &mut ScriptLowerer<'_>, state: u32) {
        l.set_resource_phase(Self::execution(), state);
    }
    fn lower_value(
        l: &mut ScriptLowerer<'_>,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = if ASYNC {
            super::resumable_operand::ResumableOperandProtocol::Async
        } else {
            super::resumable_operand::ResumableOperandProtocol::Generator
        };
        protocol.lower(l, source)
    }
    fn finish_region(
        l: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
    ) -> Option<Self::Region> {
        (l.resource_entry_state(Self::execution())? == range.end()).then_some(())?;
        ResumableRegionIr::new(block, range, Self::execution()).ok()
    }
    fn expression(region: Self::Region, value: TypedExpr) -> Self::Expression {
        ResumableExpressionIr::new(region, value)
    }
    fn case(selector: Option<Self::Expression>, body: Self::Region) -> Option<Self::Case> {
        AsyncGeneratorSwitchCaseIr::new_complete(selector, body).ok()
    }
    fn enter_body(l: &mut ScriptLowerer<'_>) {
        if ASYNC {
            l.plain_async_resource_depth += 1;
        } else {
            l.ordinary_generator_region_depth += 1;
            l.ordinary_generator_switch_depth += 1;
        }
    }
    fn leave_body(l: &mut ScriptLowerer<'_>) {
        if ASYNC {
            l.plain_async_resource_depth -= 1;
        } else {
            l.ordinary_generator_switch_depth -= 1;
            l.ordinary_generator_region_depth -= 1;
        }
    }
    fn resource_source(states: &Self::States) -> Option<&AsyncGeneratorScopedResourceSourceStates> {
        states.resource()
    }
    fn lower_body_items(
        l: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
        scope: &mut LexicalScopeInstantiation,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<BlockIr> {
        match resource {
            Some((states, capability)) => {
                l.lower_mixed_case_resource_items(items, scope, states, capability)
            }
            None => Some(l.lower_statement_items_without_function_initialization(items, scope)),
        }
    }
    fn finish_body_region(
        l: &ScriptLowerer<'_>,
        block: BlockIr,
        range: Self::Range,
        resource: Option<(
            &AsyncGeneratorScopedResourceSourceStates,
            &OwnedEnvBindingIr,
        )>,
    ) -> Option<Self::Region> {
        match resource {
            Some((states, capability)) => {
                l.finish_resumable_resource_region(states, range, capability, block)
            }
            None => Self::finish_region(l, block, range),
        }
    }
    fn resource_owner(
        states: &Self::States,
        capability: Option<OwnedEnvBindingIr>,
        cases: &[Self::Case],
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<Option<AsyncGeneratorScopedResourceIr>> {
        match (states.resource(), capability) {
            (Some(states), Some(capability)) => {
                let regions = cases
                    .iter()
                    .filter_map(|case| case.selector().map(|selector| selector.region()))
                    .chain(cases.iter().map(|case| case.body()))
                    .collect::<Vec<_>>();
                Some(Some(
                    AsyncGeneratorScopedResourceIr::new_resumable(
                        states, capability, &regions, inventory,
                    )
                    .ok()?,
                ))
            }
            (None, None) => Some(None),
            (Some(_), None) | (None, Some(_)) => None,
        }
    }
    fn plan(
        states: Self::States,
        discriminant: Self::Expression,
        discriminant_binding: OwnedEnvBindingIr,
        environment: Option<LexicalEnvironmentIr>,
        declarations: Vec<StatementIr>,
        cases: Vec<Self::Case>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<StatementIr> {
        (states.execution() == Self::execution()).then_some(())?;
        AsyncGeneratorSwitchIr::new_complete(
            states,
            discriminant,
            discriminant_binding,
            environment,
            declarations,
            cases,
            value_binding,
            resource,
            inventory,
        )
        .ok()
        .map(|plan| StatementIr::AsyncGeneratorSwitch(Box::new(plan)))
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_complete_resource_switch(
        &mut self,
        switch: &AstSwitch,
        execution: ResumableRegionProtocolIr,
    ) -> (StatementIr, ValueKind) {
        self.lower_complete_switch_for_protocol(switch, execution)
    }
    pub(super) fn lower_complete_async_switch(
        &mut self,
        switch: &AstSwitch,
    ) -> (StatementIr, ValueKind) {
        self.lower_complete_switch_for_protocol(switch, ResumableRegionProtocolIr::Async)
    }
    fn lower_complete_switch_for_protocol(
        &mut self,
        switch: &AstSwitch,
        execution: ResumableRegionProtocolIr,
    ) -> (StatementIr, ValueKind) {
        let source = crate::async_generator_source::AsyncGeneratorSwitchSource::for_protocol(
            switch, execution,
        );
        let states = source.and_then(|source| source.states(self.resource_entry_state(execution)?));
        let Some(states) = states else {
            self.unsupported("CaseBlock has no checked complete source lifetime");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let previous = self.async_value_branch_context;
        if execution == ResumableRegionProtocolIr::Async {
            self.async_value_branch_context = AsyncValueBranchContext::Switch {
                loop_depth: self.loop_depth,
            };
        }
        let result = match execution {
            ResumableRegionProtocolIr::Generator => {
                self.lower_resumable_switch::<CompleteSwitch<false>>(switch, states)
            }
            ResumableRegionProtocolIr::Async => {
                self.lower_resumable_switch::<CompleteSwitch<true>>(switch, states)
            }
            ResumableRegionProtocolIr::AsyncGenerator => {
                self.unsupported("mixed CaseBlock must use its original checked entry");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        };
        self.async_value_branch_context = previous;
        result
    }
    pub(super) fn lower_resumable_switch<P: ResumableSwitchProtocol>(
        &mut self,
        switch: &AstSwitch,
        states: P::States,
    ) -> (StatementIr, ValueKind) {
        let discriminant_range = P::discriminant(&states);
        let (mut prefix, value) = match P::lower_value(self, switch.val()) {
            Some(lowered) => lowered,
            None => {
                self.unsupported(P::NO_DISCRIMINANT_VALUE);
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        };
        // This private cell retains the whole GetValue outside CaseBlock.
        // Caller code may mutate its referent during later selector resumes.
        let mut info = value.value_info();
        info.heap_shape = None;
        let discriminant_binding =
            self.allocate_generator_switch_binding(P::DISCRIMINANT_HINT, info.clone());
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: discriminant_binding.name.clone(),
            init: value,
        });
        let Some(discriminant_region) = P::finish_region(
            self,
            BlockIr {
                statements: prefix,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            discriminant_range,
        ) else {
            self.unsupported(P::INVALID_DISCRIMINANT);
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let discriminant = P::expression(
            discriminant_region,
            TypedExpr::from_info(info, ExprIr::Identifier(discriminant_binding.name.clone())),
        );
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.breakable_depth += 1;
        // Nested If bodies consume the same complete region authority as
        // classic loops. Switch does not acquire an iteration/Continue target.
        P::enter_body(self);
        let mut scope = LexicalScopeInstantiation::instantiate_switch(self, switch);
        let resource_capability = P::resource_source(&states)
            .map(|_| self.allocate_mixed_resource_capability("async.generator.switch.resource."));
        let mut merged_vars = before_vars.clone();
        let mut merged_globals = before_globals.clone();
        let lowered = (|| -> Option<(StatementIr, ValueKind)> {
            let mut last_function_by_name = BTreeMap::new();
            for case in switch.cases() {
                for item in case.body().statements() {
                    if let Some(function) = statement_list_item_function_declaration(item) {
                        last_function_by_name
                            .insert(function_name(self.interner, function, None), function);
                    }
                }
            }
            let lexical_declarations = last_function_by_name
                .into_values()
                .map(|function| self.lower_function_declaration(function))
                .collect::<Vec<_>>();

            let mut conditions = Vec::with_capacity(switch.cases().len());
            for (case, range) in switch.cases().iter().zip(P::selectors(&states)) {
                let selector = match (case.condition(), range) {
                    (Some(source), Some(range)) => {
                        P::set_phase(self, P::entry(*range));
                        let (prefix, mut condition) = P::lower_value(self, source)?;
                        condition.heap_shape = None;
                        let region = P::finish_region(
                            self,
                            BlockIr {
                                statements: prefix,
                                result_kind: ValueKind::Undefined,
                                lexical_environment: None,
                            },
                            *range,
                        )?;
                        Some(P::expression(region, condition))
                    }
                    (None, None) => None,
                    (Some(_), None) | (None, Some(_)) => return None,
                };
                // A later selector is reached only after preceding tests fail;
                // all their actual effects still precede its evaluation.
                conditions.push((
                    selector,
                    self.var_bindings.clone(),
                    self.global_properties.clone(),
                ));
            }
            let fallback_vars = self.var_bindings.clone();
            let fallback_globals = self.global_properties.clone();
            merged_vars = self.merge_var_bindings(&before_vars, &fallback_vars);
            merged_globals = self.merge_global_properties(&before_globals, &fallback_globals);
            let mut fallthrough_facts = None;
            let mut result_kind = None;
            let mut cases = Vec::with_capacity(switch.cases().len());
            for ((case, range), (selector, case_vars, case_globals)) in switch
                .cases()
                .iter()
                .zip(P::bodies(&states))
                .zip(conditions)
            {
                let (direct_vars, direct_globals) = if selector.is_none() {
                    (fallback_vars.clone(), fallback_globals.clone())
                } else {
                    (case_vars, case_globals)
                };
                // Entry may be a direct match or normal source-order
                // fallthrough. These facts never invent execution of a test.
                if let Some((previous_vars, previous_globals)) = fallthrough_facts {
                    self.var_bindings = self.merge_var_bindings(&direct_vars, &previous_vars);
                    self.global_properties =
                        self.merge_global_properties(&direct_globals, &previous_globals);
                } else {
                    self.var_bindings = direct_vars;
                    self.global_properties = direct_globals;
                }
                self.widen_switch_scope_value_facts();
                P::set_phase(self, P::entry(*range));
                let body = P::lower_body_items(
                    self,
                    case.body().statements(),
                    &mut scope,
                    P::resource_source(&states).zip(resource_capability.as_ref()),
                )?;
                merged_vars = self.merge_var_bindings(&merged_vars, &self.var_bindings);
                merged_globals =
                    self.merge_global_properties(&merged_globals, &self.global_properties);
                fallthrough_facts =
                    Some((self.var_bindings.clone(), self.global_properties.clone()));
                result_kind = Some(match result_kind {
                    Some(kind) if kind != body.result_kind => ValueKind::Undefined,
                    _ => body.result_kind,
                });
                let body = P::finish_body_region(
                    self,
                    body,
                    *range,
                    P::resource_source(&states).zip(resource_capability.as_ref()),
                )?;
                cases.push(P::case(selector, body)?);
            }
            let lexical_environment = self.lower_materialized_lexical_environment(
                self.analysis
                    .switch_environment_ids
                    .get(&(switch as *const AstSwitch as usize))
                    .copied(),
            );
            let value_binding =
                self.allocate_generator_switch_binding(P::VALUE_HINT, ValueInfo::undefined());
            let kind = result_kind.unwrap_or(ValueKind::Undefined);
            let kind = if switch.cases().iter().any(|case| case.condition().is_none()) {
                kind
            } else {
                self.merge_value_kinds(kind, ValueKind::Undefined)
            };
            let exit = P::exit(&states);
            let resource = P::resource_owner(
                &states,
                resource_capability,
                &cases,
                &self.generated_owned_env_bindings,
            )?;
            let plan = P::plan(
                states,
                discriminant,
                discriminant_binding,
                lexical_environment,
                lexical_declarations,
                cases,
                value_binding,
                resource,
                &self.generated_owned_env_bindings,
            )?;
            P::set_phase(self, exit);
            Some((plan, kind))
        })();
        P::leave_body(self);
        self.breakable_depth -= 1;
        scope.finish(self);
        self.var_bindings = merged_vars;
        self.global_properties = merged_globals;
        self.widen_switch_scope_value_facts();
        match lowered {
            Some(result) => result,
            None => {
                self.unsupported(P::INVALID_PLAN);
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }
    fn allocate_generator_switch_binding(
        &mut self,
        hint: &str,
        info: ValueInfo,
    ) -> OwnedEnvBindingIr {
        let name = self.alloc_suspension_owned_binding(hint, info);
        self.generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .expect("retained Switch value owns its allocated activation cell")
            .clone()
    }
}
