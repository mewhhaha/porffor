//! Every conditionally evaluated suspended operand owns one real mixed branch.
use super::*;

pub(crate) struct AsyncGeneratorOptionalChainStates {
    layouts: std::vec::IntoIter<AsyncGeneratorIfSourceStates>,
    exit: u32,
}
impl AsyncGeneratorOptionalChainStates {
    pub(crate) fn from_operands(operands: &[&Expression], entry: u32) -> Option<Self> {
        let mut cursor = entry;
        let mut layouts = Vec::new();
        for source in operands.iter().filter(|source| has_suspension(**source)) {
            let states = expression::branch_parts_states(None, None, Some(source), cursor)?;
            cursor = states.exit();
            layouts.push(states);
        }
        cursor.checked_add(1)?;
        Some(Self {
            layouts: layouts.into_iter(),
            exit: cursor,
        })
    }
    pub(crate) fn next(&mut self) -> Option<AsyncGeneratorIfSourceStates> {
        self.layouts.next()
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn finish(self) -> Option<u32> {
        self.layouts.as_slice().is_empty().then_some(self.exit)
    }
}

pub(super) fn append(
    source: &Optional,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    GeneratorOptionalChainSource::new_mixed(source)
        .ok_or(AsyncGeneratorSourceError::UnsupportedExpression)?;
    expression::append(source.target(), states)?;
    for link in source.chain() {
        let mut operand = |source: &Expression| -> Result<(), AsyncGeneratorSourceError> {
            if has_suspension(source) {
                expression::append_branches(None, None, Some(source), states)?;
            }
            Ok(())
        };
        match link.kind() {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                if let PropertyAccessField::Expr(key) = field {
                    operand(key)?;
                }
            }
            OptionalOperationKind::Call { args } => {
                for source in args.as_ref() {
                    operand(match source {
                        Expression::Spread(spread) => spread.target(),
                        source => source,
                    })?;
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { .. } => {}
        }
    }
    Ok(())
}
