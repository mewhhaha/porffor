//! One actual allocator owns CaseBlock entry, lazy selectors and fallthrough.
use super::*;

pub(crate) struct AsyncGeneratorSwitchSourceStates {
    execution: ResumableRegionProtocolIr,
    discriminant: AsyncGeneratorSourceRange,
    case_block_entry: u32,
    selectors: Box<[Option<AsyncGeneratorSourceRange>]>,
    fallback_state: u32,
    bodies: Box<[AsyncGeneratorSourceRange]>,
    exit: u32,
    resource: Option<AsyncGeneratorScopedResourceSourceStates>,
    suspensions: Vec<ResumableSuspensionPointIr>,
}
impl AsyncGeneratorSwitchSourceStates {
    pub(crate) fn execution(&self) -> ResumableRegionProtocolIr {
        self.execution
    }
    pub(crate) fn discriminant(&self) -> AsyncGeneratorSourceRange {
        self.discriminant
    }
    pub(crate) fn case_block_entry(&self) -> u32 {
        self.case_block_entry
    }
    pub(crate) fn selectors(&self) -> &[Option<AsyncGeneratorSourceRange>] {
        &self.selectors
    }
    pub(crate) fn fallback_state(&self) -> u32 {
        self.fallback_state
    }
    pub(crate) fn bodies(&self) -> &[AsyncGeneratorSourceRange] {
        &self.bodies
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn resource(&self) -> Option<&AsyncGeneratorScopedResourceSourceStates> {
        self.resource.as_ref()
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AsyncGeneratorSwitchSource<'ast> {
    source: &'ast AstSwitch,
    execution: ResumableRegionProtocolIr,
}
impl<'ast> AsyncGeneratorSwitchSource<'ast> {
    pub(crate) fn new(source: &'ast AstSwitch) -> Option<Self> {
        Self::for_protocol(source, ResumableRegionProtocolIr::AsyncGenerator)
    }
    pub(crate) fn for_protocol(
        source: &'ast AstSwitch,
        execution: ResumableRegionProtocolIr,
    ) -> Option<Self> {
        match execution {
            ResumableRegionProtocolIr::AsyncGenerator => {
                region::admits(source).then_some(())?;
            }
            ResumableRegionProtocolIr::Generator => {
                if contains(source, ContainsSymbol::AwaitExpression)
                    || !has_direct_resources(source)
                {
                    return None;
                }
            }
            ResumableRegionProtocolIr::Async => {
                if contains(source, ContainsSymbol::YieldExpression)
                    || !(has_direct_resources(source) || has_async_operands(source))
                {
                    return None;
                }
            }
        }
        let owner = Self { source, execution };
        owner.states(0)?;
        Some(owner)
    }
    pub(crate) fn source(self) -> &'ast AstSwitch {
        self.source
    }
    pub(crate) fn states(self, entry: u32) -> Option<AsyncGeneratorSwitchSourceStates> {
        let mut states = ResumableStateAllocator::at(entry);
        let (discriminant, case_block_entry, selectors, fallback_state, bodies, resource) =
            append_phases(self.source, &mut states, self.execution).ok()?;
        let exit = states.current();
        let (suspensions, _) = states.into_tape();
        Some(AsyncGeneratorSwitchSourceStates {
            execution: self.execution,
            discriminant,
            case_block_entry,
            selectors,
            fallback_state,
            bodies,
            exit,
            resource,
            suspensions,
        })
    }
}

pub(super) fn append(
    source: &AstSwitch,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    if !region::admits(source) {
        return Err(AsyncGeneratorSourceError::ForeignContinuation);
    }
    append_phases(source, states, ResumableRegionProtocolIr::AsyncGenerator).map(|_| ())
}

pub(crate) fn append_for_protocol(
    source: &AstSwitch,
    execution: ResumableRegionProtocolIr,
    cursor: &mut u32,
    points: &mut Vec<ResumableSuspensionPointIr>,
) -> Option<()> {
    let owner = AsyncGeneratorSwitchSource::for_protocol(source, execution)?;
    let plan = owner.states(*cursor)?;
    *cursor = plan.exit();
    points.extend_from_slice(plan.suspensions());
    Some(())
}
pub(crate) fn has_direct_resources(source: &AstSwitch) -> bool {
    source.cases().iter().any(|case| {
        case.body()
            .statements()
            .iter()
            .any(|item| resource::declaration(item).is_some())
    })
}
/// The complete operand regions retain branch-sensitive prefixes and terminal
/// values through matching. Body-only switches keep their existing owner.
pub(crate) fn has_async_operands(source: &AstSwitch) -> bool {
    contains(source.val(), ContainsSymbol::AwaitExpression)
        || source
            .cases()
            .iter()
            .filter_map(|case| case.condition())
            .any(|selector| contains(selector, ContainsSymbol::AwaitExpression))
}

type SwitchPhases = (
    AsyncGeneratorSourceRange,
    u32,
    Box<[Option<AsyncGeneratorSourceRange>]>,
    u32,
    Box<[AsyncGeneratorSourceRange]>,
    Option<AsyncGeneratorScopedResourceSourceStates>,
);

fn append_phases(
    source: &AstSwitch,
    states: &mut ResumableStateAllocator,
    execution: ResumableRegionProtocolIr,
) -> Result<SwitchPhases, AsyncGeneratorSourceError> {
    let discriminant = phase(states, |states| {
        for_of::append_expression(source.val(), execution, states)
    })?;
    let case_block_entry = states.current();
    let (selectors, fallback_state, bodies, resource) = states.with_enclosing_scope(|states| {
        let mut selectors = Vec::with_capacity(source.cases().len());
        let mut has_default = false;
        for case in source.cases() {
            selectors.push(match case.condition() {
                Some(source) => Some(phase(states, |states| {
                    for_of::append_expression(source, execution, states)
                })?),
                None if !has_default => {
                    has_default = true;
                    None
                }
                None => return Err(AsyncGeneratorSourceError::ForeignContinuation),
            });
        }
        let fallback_state = states.current();
        states.reserve()?;
        let mut bodies = Vec::with_capacity(source.cases().len());
        let mut registrations = Vec::new();
        for case in source.cases() {
            bodies.push(phase(states, |states| {
                resource::append_control_items_for_protocol(
                    case.body().statements(),
                    states,
                    &mut registrations,
                    execution,
                )
            })?);
        }
        let body_end = bodies.last().map_or(fallback_state, |range| range.end());
        let resource = resource::finish_scoped_for_protocol(
            case_block_entry,
            body_end,
            registrations,
            states,
            execution,
        )?;
        Ok((
            selectors.into_boxed_slice(),
            fallback_state,
            bodies.into_boxed_slice(),
            resource,
        ))
    })?;
    Ok((
        discriminant,
        case_block_entry,
        selectors,
        fallback_state,
        bodies,
        resource,
    ))
}
