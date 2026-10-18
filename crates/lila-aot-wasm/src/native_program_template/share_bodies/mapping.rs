use super::*;
use std::collections::{btree_map::Entry, BTreeSet};
use wasmparser::{KnownCustom, Name, Parser, Payload, TypeRef};

pub(super) struct Mapping {
    pub functions: Vec<u32>,
    pub keep: Vec<bool>,
    imported: u32,
}
impl Mapping {
    pub fn retained(&self, function: u32) -> bool {
        function < self.imported
            || self
                .keep
                .get((function - self.imported) as usize)
                .copied()
                .unwrap_or(false)
    }
    pub fn read(
        bytes: &[u8],
        owners: &BTreeMap<u32, Owner>,
    ) -> Result<Option<Self>, NativeProgramTemplateError> {
        let mut imported = 0;
        let mut types = Vec::new();
        let mut bodies = Vec::new();
        let mut scripts = BTreeSet::new();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.map_err(invalid)? {
                Payload::ImportSection(reader) => {
                    for import in reader.into_imports() {
                        if matches!(
                            import.map_err(invalid)?.ty,
                            TypeRef::Func(_) | TypeRef::FuncExact(_)
                        ) {
                            imported += 1;
                        }
                    }
                }
                Payload::FunctionSection(reader) => {
                    for ty in reader {
                        types.push(ty.map_err(invalid)?);
                    }
                }
                Payload::CodeSectionEntry(body) => bodies.push(body),
                Payload::CustomSection(section) => {
                    match section.as_known() {
                        KnownCustom::Name(reader) => {
                            for name in reader {
                                if let Name::Function(reader) = name.map_err(invalid)? {
                                    for row in reader {
                                        let row = row.map_err(invalid)?;
                                        if row.name.starts_with("js::") {
                                            scripts.insert(row.index);
                                        }
                                    }
                                }
                            }
                        }
                        // This byte is a compilation policy, with no code offsets.
                        _ if section.name()
                            == crate::RUNTIME_SOURCE_COMPILATION_PROFILE_CUSTOM_SECTION => {}
                        // Unknown debug/relocation formats may contain indices or
                        // offsets. Preserve their whole module until supported.
                        _ => return Ok(None),
                    }
                }
                _ => {}
            }
        }
        if types.len() != bodies.len() {
            return Err(invalid("shared body declaration count mismatch"));
        }
        let mut functions = (0..imported).collect::<Vec<_>>();
        let mut keep = Vec::new();
        let mut next = imported;
        let mut seen = BTreeMap::new();
        for (ordinal, (ty, body)) in types.into_iter().zip(bodies).enumerate() {
            let old = imported + ordinal as u32;
            let owner = owners.get(&old).filter(|_| scripts.contains(&old));
            let canonical = if let Some(owner) = owner {
                // Equality is exact after all source/literal rewrites. Numeric
                // SOURCE_ID constants in creator bodies are never renumbered.
                match seen.entry((ty, *owner, body.as_bytes().to_vec())) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        entry.insert(next);
                        next
                    }
                }
            } else {
                next
            };
            keep.push(canonical == next);
            if canonical == next {
                next += 1;
            }
            functions.push(canonical);
        }
        if keep.iter().all(|keep| *keep) {
            return Ok(None);
        }
        Ok(Some(Self {
            functions,
            keep,
            imported,
        }))
    }
}
