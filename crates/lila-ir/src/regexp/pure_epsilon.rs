//! Only structural epsilon expressions have no input, capture, assertion or
//! failure effect. Quantifier bounds and preference cannot change that language.
use super::{ParsedAtom, ParsedTerm};

pub(super) fn atom_is_pure_epsilon(atom: &ParsedAtom) -> bool {
    match atom {
        ParsedAtom::NonCapture { body, .. } => alternatives_are_pure_epsilon(body),
        ParsedAtom::Instruction(_)
        | ParsedAtom::FiniteClassSet(_)
        | ParsedAtom::Capture { .. }
        | ParsedAtom::NamedBackreference { .. }
        | ParsedAtom::NumberedBackreference { .. }
        | ParsedAtom::Lookaround { .. } => false,
    }
}

pub(super) fn alternatives_are_pure_epsilon(alternatives: &[Vec<ParsedTerm>]) -> bool {
    !alternatives.is_empty()
        && alternatives.iter().all(|sequence| {
            sequence.iter().all(|term| match term {
                ParsedTerm::Quantified { atom, .. } => atom_is_pure_epsilon(atom),
                ParsedTerm::LegacyUtf16Pair { .. } => false,
            })
        })
}
