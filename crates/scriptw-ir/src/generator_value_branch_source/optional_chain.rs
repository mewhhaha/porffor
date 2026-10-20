//! Complete source regions for actual optional-chain operands and destinations.
use super::*;
use boa_interner::Sym;
use branches::GeneratorValueRegionStates;

#[derive(Clone, Copy)]
enum OptionalChainSourceProtocol {
    Generator,
    Mixed,
}

/// The first link fixes whether the base is evaluated as a Value or Call
/// Reference. The actual source walk, including private acquisition, is retained.
pub(crate) struct GeneratorOptionalBaseSource<'ast> {
    source: &'ast Expression,
    plan: Option<GeneratorExpressionSourcePlan<'ast>>,
}
impl<'ast> GeneratorOptionalBaseSource<'ast> {
    fn new(source: &'ast Expression, first_is_call: bool) -> Option<Self> {
        let mut plan = GeneratorExpressionSourcePlan { steps: Vec::new() };
        if first_is_call {
            plan.invocation_reference(source, GeneratorValueBranchAdmission::OrdinaryOutsideLoops)?;
        } else {
            plan.expression(source, GeneratorValueBranchAdmission::OrdinaryOutsideLoops)?;
        }
        Some(Self {
            source,
            plan: Some(plan),
        })
    }
    fn new_mixed(source: &'ast Expression) -> Option<Self> {
        crate::async_generator_source::AsyncGeneratorExpressionSource::new(source)?;
        Some(Self { source, plan: None })
    }
    pub(crate) fn source(&self) -> &'ast Expression {
        self.source
    }
}

pub(crate) struct GeneratorOptionalOperandSource<'ast> {
    source: &'ast Expression,
}
impl<'ast> GeneratorOptionalOperandSource<'ast> {
    fn new(source: &'ast Expression) -> Option<Self> {
        GeneratorExpressionSourcePlan::new(
            source,
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
        )?;
        Some(Self { source })
    }
    fn for_protocol(
        source: &'ast Expression,
        protocol: OptionalChainSourceProtocol,
    ) -> Option<Self> {
        match protocol {
            OptionalChainSourceProtocol::Generator => Self::new(source),
            OptionalChainSourceProtocol::Mixed => {
                crate::async_generator_source::AsyncGeneratorExpressionSource::new(source)?;
                Some(Self { source })
            }
        }
    }
    pub(crate) fn source(&self) -> &'ast Expression {
        self.source
    }
    pub(crate) fn suspends(&self) -> bool {
        contains(self.source, ContainsSymbol::YieldExpression)
            || contains(self.source, ContainsSymbol::AwaitExpression)
    }
}

pub(crate) struct GeneratorOptionalChainSource<'ast> {
    base: GeneratorOptionalBaseSource<'ast>,
    links: Vec<GeneratorOptionalChainLink<'ast>>,
    protocol: OptionalChainSourceProtocol,
}

/// Grouping preserves the actual final property Reference or final Call Value.
#[must_use = "the complete grouped optional source must be consumed"]
pub(crate) enum GeneratorGroupedOptionalInvocationSource<'ast> {
    PropertyReference(GeneratorGroupedOptionalReferenceSource<'ast>),
    CallValue(GeneratorGroupedOptionalCallValueSource<'ast>),
}
pub(crate) struct GeneratorGroupedOptionalReferenceSource<'ast> {
    chain: GeneratorOptionalChainSource<'ast>,
}
pub(crate) struct GeneratorGroupedOptionalCallValueSource<'ast> {
    chain: GeneratorOptionalChainSource<'ast>,
}
impl<'ast> GeneratorGroupedOptionalInvocationSource<'ast> {
    pub(crate) fn new_mixed(source: &'ast Expression) -> Option<Self> {
        let mut source = source;
        while let Expression::Parenthesized(group) = source {
            source = group.expression();
        }
        let Expression::Optional(optional) = source else {
            return None;
        };
        match optional.chain().last()?.kind() {
            OptionalOperationKind::SimplePropertyAccess { .. } => Some(Self::PropertyReference(
                GeneratorGroupedOptionalReferenceSource {
                    chain: GeneratorOptionalChainSource::new_mixed(optional)?,
                },
            )),
            OptionalOperationKind::Call { .. } => {
                Some(Self::CallValue(GeneratorGroupedOptionalCallValueSource {
                    chain: GeneratorOptionalChainSource::new_mixed(optional)?,
                }))
            }
            OptionalOperationKind::PrivatePropertyAccess { .. } => Some(Self::PropertyReference(
                GeneratorGroupedOptionalReferenceSource {
                    chain: GeneratorOptionalChainSource::new_mixed(optional)?,
                },
            )),
        }
    }
    pub(crate) fn new(
        source: &'ast Expression,
        admission: GeneratorValueBranchAdmission,
    ) -> Option<Self> {
        if !matches!(
            admission,
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops
        ) {
            return None;
        }
        let mut source = source;
        while let Expression::Parenthesized(group) = source {
            source = group.expression();
        }
        let Expression::Optional(optional) = source else {
            return None;
        };
        match optional.chain().last()?.kind() {
            OptionalOperationKind::SimplePropertyAccess { .. } => Some(Self::PropertyReference(
                GeneratorGroupedOptionalReferenceSource {
                    chain: GeneratorOptionalChainSource::new(optional)?,
                },
            )),
            OptionalOperationKind::Call { .. } => {
                Some(Self::CallValue(GeneratorGroupedOptionalCallValueSource {
                    chain: GeneratorOptionalChainSource::new(optional)?,
                }))
            }
            OptionalOperationKind::PrivatePropertyAccess { .. } => Some(Self::PropertyReference(
                GeneratorGroupedOptionalReferenceSource {
                    chain: GeneratorOptionalChainSource::new(optional)?,
                },
            )),
        }
    }
    pub(crate) fn into_chain(self) -> GeneratorOptionalChainSource<'ast> {
        match self {
            Self::PropertyReference(source) => source.into_chain(),
            Self::CallValue(source) => source.into_chain(),
        }
    }
}
impl<'ast> GeneratorGroupedOptionalReferenceSource<'ast> {
    pub(crate) fn into_chain(self) -> GeneratorOptionalChainSource<'ast> {
        self.chain
    }
}
impl<'ast> GeneratorGroupedOptionalCallValueSource<'ast> {
    pub(crate) fn into_chain(self) -> GeneratorOptionalChainSource<'ast> {
        self.chain
    }
}

pub(crate) enum GeneratorOptionalDeleteTerminal {
    Property,
    Value,
}
/// Only the actual last source link can choose Delete's Reference/Value role.
pub(crate) struct GeneratorOptionalDeleteSource<'ast> {
    chain: GeneratorOptionalChainSource<'ast>,
    terminal: GeneratorOptionalDeleteTerminal,
}
impl<'ast> GeneratorOptionalDeleteSource<'ast> {
    pub(crate) fn new(source: &'ast Optional) -> Option<Self> {
        Self::for_protocol(source, OptionalChainSourceProtocol::Generator)
    }
    pub(crate) fn new_mixed(source: &'ast Optional) -> Option<Self> {
        Self::for_protocol(source, OptionalChainSourceProtocol::Mixed)
    }
    fn for_protocol(source: &'ast Optional, protocol: OptionalChainSourceProtocol) -> Option<Self> {
        let terminal = match source.chain().last()?.kind() {
            OptionalOperationKind::SimplePropertyAccess { .. } => {
                GeneratorOptionalDeleteTerminal::Property
            }
            OptionalOperationKind::Call { .. } => GeneratorOptionalDeleteTerminal::Value,
            OptionalOperationKind::PrivatePropertyAccess { .. } => return None,
        };
        Some(Self {
            chain: GeneratorOptionalChainSource::for_protocol(source, protocol)?,
            terminal,
        })
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        GeneratorOptionalChainSource<'ast>,
        GeneratorOptionalDeleteTerminal,
    ) {
        (self.chain, self.terminal)
    }
    pub(crate) fn into_chain(self) -> GeneratorOptionalChainSource<'ast> {
        self.chain
    }
}

pub(crate) enum GeneratorOptionalChainLink<'ast> {
    Property(GeneratorOptionalPropertyLink<'ast>),
    Private { field: PrivateName, shorted: bool },
    Call(GeneratorOptionalCallLink<'ast>),
}
pub(crate) struct GeneratorOptionalPropertyLink<'ast> {
    field: GeneratorOptionalKeySource<'ast>,
    shorted: bool,
}
pub(crate) enum GeneratorOptionalKeySource<'ast> {
    Constant(Sym),
    Computed(GeneratorOptionalOperandSource<'ast>),
}
pub(crate) struct GeneratorOptionalCallLink<'ast> {
    source: &'ast [Expression],
    arguments: Vec<GeneratorOptionalArgumentSource<'ast>>,
    shorted: bool,
}
pub(crate) enum GeneratorOptionalArgumentSource<'ast> {
    Value(GeneratorOptionalOperandSource<'ast>),
    Spread(GeneratorOptionalOperandSource<'ast>),
}
impl GeneratorOptionalChainLink<'_> {
    pub(crate) fn shorted(&self) -> bool {
        match self {
            Self::Property(link) => link.shorted,
            Self::Private { shorted, .. } => *shorted,
            Self::Call(link) => link.shorted,
        }
    }
}
impl<'ast> GeneratorOptionalPropertyLink<'ast> {
    pub(crate) fn into_parts(self) -> (GeneratorOptionalKeySource<'ast>, bool) {
        (self.field, self.shorted)
    }
}
impl<'ast> GeneratorOptionalCallLink<'ast> {
    pub(crate) fn into_parts(
        self,
    ) -> (
        &'ast [Expression],
        Vec<GeneratorOptionalArgumentSource<'ast>>,
        bool,
    ) {
        (self.source, self.arguments, self.shorted)
    }
}
impl<'ast> GeneratorOptionalArgumentSource<'ast> {
    fn operand(&self) -> &GeneratorOptionalOperandSource<'ast> {
        match self {
            Self::Value(source) | Self::Spread(source) => source,
        }
    }
}

impl<'ast> GeneratorOptionalChainSource<'ast> {
    pub(crate) fn new(source: &'ast Optional) -> Option<Self> {
        Self::for_protocol(source, OptionalChainSourceProtocol::Generator)
    }
    pub(crate) fn new_mixed(source: &'ast Optional) -> Option<Self> {
        Self::for_protocol(source, OptionalChainSourceProtocol::Mixed)
    }
    fn for_protocol(source: &'ast Optional, protocol: OptionalChainSourceProtocol) -> Option<Self> {
        match protocol {
            OptionalChainSourceProtocol::Generator
                if !contains(source, ContainsSymbol::YieldExpression)
                    || contains(source, ContainsSymbol::AwaitExpression) =>
            {
                return None
            }
            OptionalChainSourceProtocol::Mixed
                if !contains(source, ContainsSymbol::YieldExpression)
                    && !contains(source, ContainsSymbol::AwaitExpression) =>
            {
                return None
            }
            OptionalChainSourceProtocol::Generator | OptionalChainSourceProtocol::Mixed => {}
        }
        if !source.chain().first().is_some_and(|first| first.shorted()) {
            return None;
        }
        let first_is_call = matches!(
            source.chain().first()?.kind(),
            OptionalOperationKind::Call { .. }
        );
        let base = match protocol {
            OptionalChainSourceProtocol::Generator => {
                GeneratorOptionalBaseSource::new(source.target(), first_is_call)?
            }
            OptionalChainSourceProtocol::Mixed => {
                GeneratorOptionalBaseSource::new_mixed(source.target())?
            }
        };
        let mut links = Vec::with_capacity(source.chain().len());
        for operation in source.chain() {
            let shorted = operation.shorted();
            links.push(match operation.kind() {
                OptionalOperationKind::SimplePropertyAccess { field } => {
                    let field = match field {
                        PropertyAccessField::Const(name) => {
                            GeneratorOptionalKeySource::Constant(name.sym())
                        }
                        PropertyAccessField::Expr(source) => GeneratorOptionalKeySource::Computed(
                            GeneratorOptionalOperandSource::for_protocol(source, protocol)?,
                        ),
                    };
                    GeneratorOptionalChainLink::Property(GeneratorOptionalPropertyLink {
                        field,
                        shorted,
                    })
                }
                OptionalOperationKind::Call { args } => {
                    let mut arguments = Vec::with_capacity(args.len());
                    for source in args.as_ref() {
                        arguments.push(match source {
                            Expression::Spread(spread) => GeneratorOptionalArgumentSource::Spread(
                                GeneratorOptionalOperandSource::for_protocol(
                                    spread.target(),
                                    protocol,
                                )?,
                            ),
                            source => GeneratorOptionalArgumentSource::Value(
                                GeneratorOptionalOperandSource::for_protocol(source, protocol)?,
                            ),
                        });
                    }
                    GeneratorOptionalChainLink::Call(GeneratorOptionalCallLink {
                        source: args,
                        arguments,
                        shorted,
                    })
                }
                OptionalOperationKind::PrivatePropertyAccess { field } => {
                    GeneratorOptionalChainLink::Private {
                        field: *field,
                        shorted,
                    }
                }
            });
        }
        Some(Self {
            base,
            links,
            protocol,
        })
    }
    pub(crate) fn is_mixed(&self) -> bool {
        matches!(self.protocol, OptionalChainSourceProtocol::Mixed)
    }
    pub(crate) fn base_source(&self) -> &'ast Expression {
        self.base.source()
    }
    pub(crate) fn starts_with_call(&self) -> bool {
        matches!(
            self.links.first(),
            Some(GeneratorOptionalChainLink::Call(_))
        )
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        GeneratorOptionalBaseSource<'ast>,
        Vec<GeneratorOptionalChainLink<'ast>>,
    ) {
        (self.base, self.links)
    }
    pub(crate) fn states(&self, entry: u32) -> Option<GeneratorOptionalChainStates> {
        if self.is_mixed() {
            return None;
        }
        Self::link_states(&self.links, entry)
    }
    pub(crate) fn mixed_states(
        &self,
        entry: u32,
    ) -> Option<crate::async_generator_source::AsyncGeneratorOptionalChainStates> {
        if !self.is_mixed() {
            return None;
        }
        let operands = self
            .links
            .iter()
            .flat_map(|link| match link {
                GeneratorOptionalChainLink::Property(link) => match &link.field {
                    GeneratorOptionalKeySource::Constant(_) => Vec::new(),
                    GeneratorOptionalKeySource::Computed(source) => vec![source.source()],
                },
                GeneratorOptionalChainLink::Call(link) => link
                    .arguments
                    .iter()
                    .map(|source| source.operand().source())
                    .collect(),
                GeneratorOptionalChainLink::Private { .. } => Vec::new(),
            })
            .collect::<Vec<_>>();
        crate::async_generator_source::AsyncGeneratorOptionalChainStates::from_operands(
            &operands, entry,
        )
    }
    fn link_states(
        links: &[GeneratorOptionalChainLink<'ast>],
        entry: u32,
    ) -> Option<GeneratorOptionalChainStates> {
        let mut cursor = entry;
        let mut layouts = Vec::new();
        let mut reserve = |source: &GeneratorOptionalOperandSource<'ast>| -> Option<()> {
            if source.suspends() {
                let states = GeneratorValueRegionStates::new(cursor, None, Some(source.source()))?;
                cursor = states.exit();
                layouts.push(states);
            }
            Some(())
        };
        for link in links {
            match link {
                GeneratorOptionalChainLink::Property(link) => match &link.field {
                    GeneratorOptionalKeySource::Constant(_) => {}
                    GeneratorOptionalKeySource::Computed(source) => reserve(source)?,
                },
                GeneratorOptionalChainLink::Call(link) => {
                    for argument in &link.arguments {
                        reserve(argument.operand())?;
                    }
                }
                GeneratorOptionalChainLink::Private { .. } => {}
            }
        }
        cursor.checked_add(1)?;
        Some(GeneratorOptionalChainStates {
            layouts: layouts.into_iter(),
            exit: cursor,
        })
    }
    pub(super) fn append(
        self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let Self {
            base,
            links,
            protocol: _,
        } = self;
        base.plan?.append(cursor, points)?;
        let mut states = Self::link_states(&links, *cursor)?;
        while let Some(layout) = states.next() {
            layout.append_suspensions(points);
        }
        *cursor = states.finish()?;
        Some(())
    }
}

pub(crate) struct GeneratorOptionalChainStates {
    layouts: std::vec::IntoIter<GeneratorValueRegionStates>,
    exit: u32,
}
impl GeneratorOptionalChainStates {
    pub(crate) fn next(&mut self) -> Option<GeneratorValueRegionStates> {
        self.layouts.next()
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn finish(self) -> Option<u32> {
        self.layouts.as_slice().is_empty().then_some(self.exit)
    }
}
