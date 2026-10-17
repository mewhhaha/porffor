//! Relocations for checked P string ordinals and table lengths only.
//!
//! Ordinary JavaScript numbers and all Wasm structural indices remain code.
use crate::gc_types::{ProgramLiteralCount, ProgramLiteralIndex};
use std::borrow::Cow;

pub(crate) const SECTION_NAME: &str = "lila.native_program.literal_operands";
const VERSION: u32 = 1;
const RECORD_BYTES: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Count,
    Index,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FunctionSite {
    operator: u32,
    kind: Kind,
    expected: i32,
}
impl FunctionSite {
    pub(crate) fn count(operator: u32, value: ProgramLiteralCount) -> Self {
        Self {
            operator,
            kind: Kind::Count,
            expected: value.value() as i32,
        }
    }
    pub(crate) fn index(operator: u32, value: ProgramLiteralIndex) -> Self {
        Self {
            operator,
            kind: Kind::Index,
            expected: value.ordinal() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ModuleSite {
    body: u32,
    site: FunctionSite,
}
impl ModuleSite {
    pub(crate) fn new(body: u32, site: FunctionSite) -> Self {
        Self { body, site }
    }
}

pub(crate) fn section(sites: &[ModuleSite]) -> Option<wasm_encoder::CustomSection<'static>> {
    if sites.is_empty() {
        return None;
    }
    let mut data = Vec::with_capacity(8 + sites.len() * RECORD_BYTES);
    data.extend_from_slice(&VERSION.to_le_bytes());
    data.extend_from_slice(
        &u32::try_from(sites.len())
            .expect("literal operand count overflow")
            .to_le_bytes(),
    );
    for site in sites {
        data.extend_from_slice(&site.body.to_le_bytes());
        data.extend_from_slice(&site.site.operator.to_le_bytes());
        data.push(match site.site.kind {
            Kind::Count => 0,
            Kind::Index => 1,
        });
        data.extend_from_slice(&site.site.expected.to_le_bytes());
    }
    Some(wasm_encoder::CustomSection {
        name: Cow::Borrowed(SECTION_NAME),
        data: Cow::Owned(data),
    })
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeclaredSite {
    pub(crate) body: u32,
    pub(crate) operator: u32,
    pub(crate) kind: Kind,
    pub(crate) expected: i32,
}

pub(crate) fn read(data: &[u8]) -> Result<Vec<DeclaredSite>, &'static str> {
    if data.len() < 8 {
        return Err("truncated literal operand header");
    }
    let word = |offset| u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    if word(0) != VERSION {
        return Err("unknown literal operand version");
    }
    let count = word(4) as usize;
    if count
        .checked_mul(RECORD_BYTES)
        .and_then(|n| n.checked_add(8))
        != Some(data.len())
    {
        return Err("literal operand record length mismatch");
    }
    let mut sites = Vec::with_capacity(count);
    let mut previous = None;
    for record in data[8..].chunks_exact(RECORD_BYTES) {
        let body = u32::from_le_bytes(record[..4].try_into().unwrap());
        let operator = u32::from_le_bytes(record[4..8].try_into().unwrap());
        if previous.is_some_and(|prior| prior >= (body, operator)) {
            return Err("literal operand sites are duplicated or unordered");
        }
        previous = Some((body, operator));
        let kind = match record[8] {
            0 => Kind::Count,
            1 => Kind::Index,
            _ => return Err("unknown literal operand kind"),
        };
        let expected = i32::from_le_bytes(record[9..13].try_into().unwrap());
        sites.push(DeclaredSite {
            body,
            operator,
            kind,
            expected,
        });
    }
    Ok(sites)
}
