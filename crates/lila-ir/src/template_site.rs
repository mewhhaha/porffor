//! Compiler ownership of actual parsed template sites.

use boa_ast::{
    expression::TaggedTemplate,
    visitor::{VisitWith, Visitor},
};
use lila_front::{ParsedScript, ParsedScriptIdentity};
use std::ops::ControlFlow;

/// One actual parsed Script allocation in a joined AOT compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateSourceId(u32);

impl TemplateSourceId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A parsed source with a completed census of its template parse sites.
/// Only the parsed-owner registry constructs this plan; cache allocation and
/// site lowering both require the same completed plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateSourceIr {
    id: TemplateSourceId,
    site_count: u32,
}

impl TemplateSourceIr {
    #[must_use]
    pub const fn id(self) -> TemplateSourceId {
        self.id
    }
    #[must_use]
    pub const fn site_count(self) -> u32 {
        self.site_count
    }
    pub(crate) fn site(self, local_site: u64) -> TemplateSiteId {
        let ordinal = u32::try_from(local_site >> 32).expect("parser template ordinal fits u32");
        assert!(
            ordinal > 0 && ordinal <= self.site_count,
            "site belongs to completed source census"
        );
        TemplateSiteId {
            source: self.id,
            local_site,
            cache_slot: ordinal - 1,
        }
    }
}

/// A template parse node qualified by its real retained parsed Script owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateSiteId {
    source: TemplateSourceId,
    local_site: u64,
    cache_slot: u32,
}

impl TemplateSiteId {
    #[must_use]
    pub const fn source(self) -> TemplateSourceId {
        self.source
    }
    #[must_use]
    pub const fn local_site(self) -> u64 {
        self.local_site
    }
    #[must_use]
    pub const fn cache_slot(self) -> u32 {
        self.cache_slot
    }
}

#[derive(Default)]
struct TemplateCensus {
    site_count: u32,
}
impl<'ast> Visitor<'ast> for TemplateCensus {
    type BreakTy = ();
    fn visit_tagged_template(&mut self, template: &'ast TaggedTemplate) -> ControlFlow<()> {
        let ordinal =
            u32::try_from(template.identifier() >> 32).expect("parser template ordinal fits u32");
        self.site_count = self.site_count.max(ordinal);
        template.visit_with(self)
    }
}

/// Retaining the actual syntax allocation makes cloned parsed Scripts share
/// an owner while separately parsed equal text stays distinct, without hashes
/// or a global counter.
#[derive(Debug, Clone, Default)]
pub(crate) struct TemplateSourceOwners {
    parsed: Vec<(ParsedScriptIdentity, Option<TemplateSourceIr>)>,
}

impl TemplateSourceOwners {
    pub(crate) fn for_parsed(&mut self, source: &ParsedScript) -> Option<TemplateSourceIr> {
        let identity = source.syntax_identity();
        if let Some((_, plan)) = self.parsed.iter().find(|(owner, _)| *owner == identity) {
            return *plan;
        }
        let mut census = TemplateCensus::default();
        source.with_compiler_session(|script, _| {
            let _ = census.visit_script(script);
        });
        let plan = (census.site_count > 0).then(|| TemplateSourceIr {
            id: TemplateSourceId(
                u32::try_from(self.parsed.len()).expect("parsed template source space exhausted"),
            ),
            site_count: census.site_count,
        });
        self.parsed.push((identity, plan));
        plan
    }
}
