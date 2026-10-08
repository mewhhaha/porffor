//! Immutable dependency-sealed host source ownership shared by both backends.
//!
//! This owns source bytes and exact resolution rows, not parsed ECMAScript
//! records, runtime module instances, or an alternative module graph.

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use sha2::{Digest, Sha256};

/// The JavaScript parse goal of the entry; dependencies declare their own kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedModuleGoal {
    Script,
    Module,
}

/// Untrusted constructor input; execution accepts only the completed graph.
#[derive(Debug)]
pub struct EmbeddedModuleEntryInput {
    pub goal: EmbeddedModuleGoal,
    pub identity: String,
    pub source: String,
    pub meta_url: String,
}

/// One declared Source Text Module, before identity and URL validation.
#[derive(Debug)]
pub struct EmbeddedModuleSourceInput {
    pub identity: String,
    pub source: String,
    pub meta_url: String,
}

/// The host's declared record kind, independently of the bytes' grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedModuleKind {
    SourceText,
    Json,
}

#[derive(Debug)]
pub enum EmbeddedModuleInput {
    SourceText(EmbeddedModuleSourceInput),
    Json(EmbeddedModuleSourceInput),
}

/// A referrer role is distinct from its locator. In particular a Script can
/// import a Module at the same locator without referring to its own record.
/// `Unlocated` is an explicit resolution domain for host contexts without a
/// source locator; it never falls back to the entry's identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmbeddedModuleReferrer {
    Script(String),
    Module(String),
    Unlocated,
    /// Host-defined base for ShadowRealm imports, independent of active source.
    Realm,
}

/// One untrusted, phase-free resolution row. Phases remain ECMAScript request
/// semantics; they do not select different host source bytes.
#[derive(Debug)]
pub struct EmbeddedModuleResolutionInput {
    pub referrer: EmbeddedModuleReferrer,
    pub specifier: String,
    pub attributes: Vec<(String, String)>,
    pub target: String,
}

/// A canonical exact request, including UTF-16-key-ordered import attributes.
/// Specifiers and attribute values are JS strings, not paths or provider hints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedModuleRequest {
    specifier: String,
    attributes: Vec<(String, String)>,
}

impl EmbeddedModuleRequest {
    pub fn try_new(
        specifier: impl Into<String>,
        mut attributes: Vec<(String, String)>,
    ) -> Result<Self, EmbeddedModuleGraphError> {
        attributes.sort_by(|left, right| utf16_cmp(&left.0, &right.0));
        for adjacent in attributes.windows(2) {
            if adjacent[0].0 == adjacent[1].0 {
                return Err(EmbeddedModuleGraphError::DuplicateAttribute {
                    key: adjacent[0].0.clone(),
                });
            }
        }
        Ok(Self {
            specifier: specifier.into(),
            attributes,
        })
    }

    pub fn specifier(&self) -> &str {
        &self.specifier
    }

    pub fn attributes(&self) -> &[(String, String)] {
        &self.attributes
    }

    fn cmp(&self, other: &Self) -> Ordering {
        utf16_cmp(&self.specifier, &other.specifier).then_with(|| {
            for (left, right) in self.attributes.iter().zip(&other.attributes) {
                let order = utf16_cmp(&left.0, &right.0).then_with(|| utf16_cmp(&left.1, &right.1));
                if order != Ordering::Equal {
                    return order;
                }
            }
            self.attributes.len().cmp(&other.attributes.len())
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EmbeddedModuleEntry {
    goal: EmbeddedModuleGoal,
    identity: String,
    source: String,
    meta_url: String,
}

impl EmbeddedModuleEntry {
    pub const fn goal(&self) -> EmbeddedModuleGoal {
        self.goal
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn meta_url(&self) -> &str {
        &self.meta_url
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EmbeddedModuleSource {
    kind: EmbeddedModuleKind,
    identity: String,
    source: String,
    meta_url: String,
}

impl EmbeddedModuleSource {
    pub const fn kind(&self) -> EmbeddedModuleKind {
        self.kind
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn meta_url(&self) -> &str {
        &self.meta_url
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EmbeddedModuleResolution {
    referrer: EmbeddedModuleReferrer,
    request: EmbeddedModuleRequest,
    target: String,
}

impl EmbeddedModuleResolution {
    pub fn referrer(&self) -> &EmbeddedModuleReferrer {
        &self.referrer
    }

    pub fn request(&self) -> &EmbeddedModuleRequest {
        &self.request
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

/// The only graph accepted by an embedded loading policy. Every source, URL,
/// request and target was validated together before this owner was minted.
#[derive(Debug, PartialEq, Eq)]
pub struct EmbeddedModuleGraph {
    entry: EmbeddedModuleEntry,
    modules: Vec<EmbeddedModuleSource>,
    resolutions: Vec<EmbeddedModuleResolution>,
    fingerprint: [u8; 32],
}

impl EmbeddedModuleGraph {
    pub fn try_new(
        entry: EmbeddedModuleEntryInput,
        modules: Vec<EmbeddedModuleSourceInput>,
        resolutions: Vec<EmbeddedModuleResolutionInput>,
    ) -> Result<Arc<Self>, EmbeddedModuleGraphError> {
        Self::try_new_typed(
            entry,
            modules
                .into_iter()
                .map(EmbeddedModuleInput::SourceText)
                .collect(),
            resolutions,
        )
    }

    pub fn try_new_typed(
        entry: EmbeddedModuleEntryInput,
        modules: Vec<EmbeddedModuleInput>,
        resolutions: Vec<EmbeddedModuleResolutionInput>,
    ) -> Result<Arc<Self>, EmbeddedModuleGraphError> {
        validate_identity(&entry.identity)?;
        validate_meta_url(&entry.meta_url)?;
        let entry = EmbeddedModuleEntry {
            goal: entry.goal,
            identity: entry.identity,
            source: entry.source,
            meta_url: entry.meta_url,
        };
        let mut completed_modules = Vec::with_capacity(modules.len());
        for module in modules {
            let (kind, module) = match module {
                EmbeddedModuleInput::SourceText(module) => (EmbeddedModuleKind::SourceText, module),
                EmbeddedModuleInput::Json(module) => (EmbeddedModuleKind::Json, module),
            };
            validate_identity(&module.identity)?;
            validate_meta_url(&module.meta_url)?;
            completed_modules.push(EmbeddedModuleSource {
                kind,
                identity: module.identity,
                source: module.source,
                meta_url: module.meta_url,
            });
        }
        match entry.goal {
            EmbeddedModuleGoal::Script => {}
            EmbeddedModuleGoal::Module => completed_modules.push(EmbeddedModuleSource {
                kind: EmbeddedModuleKind::SourceText,
                identity: entry.identity.clone(),
                source: entry.source.clone(),
                meta_url: entry.meta_url.clone(),
            }),
        }
        completed_modules.sort_by(|left, right| left.identity.cmp(&right.identity));
        for adjacent in completed_modules.windows(2) {
            if adjacent[0].identity == adjacent[1].identity {
                return Err(EmbeddedModuleGraphError::DuplicateModule {
                    identity: adjacent[0].identity.clone(),
                });
            }
        }

        let mut completed_resolutions = Vec::with_capacity(resolutions.len());
        for resolution in resolutions {
            let known_referrer = match &resolution.referrer {
                EmbeddedModuleReferrer::Script(identity) => {
                    validate_identity(identity)?;
                    entry.goal == EmbeddedModuleGoal::Script && identity == &entry.identity
                }
                EmbeddedModuleReferrer::Module(identity) => {
                    validate_identity(identity)?;
                    find_module(&completed_modules, identity).is_some()
                }
                EmbeddedModuleReferrer::Unlocated | EmbeddedModuleReferrer::Realm => true,
            };
            if !known_referrer {
                return Err(EmbeddedModuleGraphError::UnknownReferrer {
                    referrer: resolution.referrer,
                });
            }
            validate_identity(&resolution.target)?;
            let Some(target) = find_module(&completed_modules, &resolution.target) else {
                return Err(EmbeddedModuleGraphError::MissingTarget {
                    identity: resolution.target,
                });
            };
            let request =
                EmbeddedModuleRequest::try_new(resolution.specifier, resolution.attributes)?;
            let json =
                matches!(request.attributes(), [(key, value)] if key == "type" && value == "json");
            let declares_json = request
                .attributes()
                .iter()
                .any(|(key, value)| key == "type" && value == "json");
            if match target.kind {
                EmbeddedModuleKind::Json => !json,
                EmbeddedModuleKind::SourceText => declares_json,
            } {
                return Err(EmbeddedModuleGraphError::RecordKindMismatch {
                    identity: resolution.target,
                });
            }
            completed_resolutions.push(EmbeddedModuleResolution {
                referrer: resolution.referrer,
                request,
                target: resolution.target,
            });
        }
        completed_resolutions.sort_by(|left, right| {
            left.referrer
                .cmp(&right.referrer)
                .then_with(|| left.request.cmp(&right.request))
        });
        for adjacent in completed_resolutions.windows(2) {
            if adjacent[0].referrer == adjacent[1].referrer
                && adjacent[0].request == adjacent[1].request
            {
                return Err(EmbeddedModuleGraphError::DuplicateResolution {
                    referrer: adjacent[0].referrer.clone(),
                    specifier: adjacent[0].request.specifier.clone(),
                });
            }
        }
        let fingerprint = fingerprint(&entry, &completed_modules, &completed_resolutions)?;
        Ok(Arc::new(Self {
            entry,
            modules: completed_modules,
            resolutions: completed_resolutions,
            fingerprint,
        }))
    }

    pub fn entry(&self) -> &EmbeddedModuleEntry {
        &self.entry
    }

    /// All Module records, including the implicit entry for a Module goal.
    pub fn modules(&self) -> &[EmbeddedModuleSource] {
        &self.modules
    }

    /// Reversible wire projection: excludes the implicitly included Module
    /// entry. A Script's same-locator Module dependency remains included.
    pub fn dependency_modules(&self) -> impl Iterator<Item = &EmbeddedModuleSource> {
        self.modules.iter().filter(|module| {
            self.entry.goal == EmbeddedModuleGoal::Script || module.identity != self.entry.identity
        })
    }

    pub fn module(&self, identity: &str) -> Option<&EmbeddedModuleSource> {
        find_module(&self.modules, identity)
    }

    pub fn resolutions(&self) -> &[EmbeddedModuleResolution] {
        &self.resolutions
    }

    /// Exact host resolution only. No URL/path derivation, phase substitution,
    /// attribute dropping, or fallback can be expressed by this lookup.
    pub fn resolve(
        &self,
        referrer: &EmbeddedModuleReferrer,
        request: &EmbeddedModuleRequest,
    ) -> Option<&EmbeddedModuleSource> {
        let index = self
            .resolutions
            .binary_search_by(|resolution| {
                resolution
                    .referrer
                    .cmp(referrer)
                    .then_with(|| resolution.request.cmp(request))
            })
            .ok()?;
        self.module(&self.resolutions[index].target)
    }

    /// SHA-256 of the complete canonical, length-framed source policy. Unused
    /// declared rows and independently declared metadata URLs are included.
    pub fn fingerprint(&self) -> &[u8; 32] {
        &self.fingerprint
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddedModuleGraphError {
    InvalidIdentity {
        identity: String,
    },
    InvalidMetaUrl {
        meta_url: String,
    },
    DuplicateModule {
        identity: String,
    },
    DuplicateAttribute {
        key: String,
    },
    UnknownReferrer {
        referrer: EmbeddedModuleReferrer,
    },
    MissingTarget {
        identity: String,
    },
    RecordKindMismatch {
        identity: String,
    },
    DuplicateResolution {
        referrer: EmbeddedModuleReferrer,
        specifier: String,
    },
    FingerprintLengthOverflow,
}

impl fmt::Display for EmbeddedModuleGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentity { identity } => write!(
                formatter,
                "embedded module identity {identity:?} must be a normalized relative lowercase ASCII virtual path"
            ),
            Self::InvalidMetaUrl { meta_url } => write!(
                formatter,
                "embedded metadata URL {meta_url:?} must be lila:// followed by a normalized virtual path"
            ),
            Self::DuplicateModule { identity } => write!(
                formatter,
                "embedded module identity {identity:?} is declared more than once"
            ),
            Self::DuplicateAttribute { key } => {
                write!(formatter, "embedded import attribute {key:?} is duplicated")
            }
            Self::UnknownReferrer { referrer } => {
                write!(
                    formatter,
                    "embedded resolution referrer {referrer:?} is not declared"
                )
            }
            Self::MissingTarget { identity } => {
                write!(
                    formatter,
                    "embedded resolution target {identity:?} is not declared"
                )
            }
            Self::RecordKindMismatch { identity } => write!(formatter, "embedded import type does not match the declared record kind of {identity:?}"),
            Self::DuplicateResolution {
                referrer,
                specifier,
            } => write!(
                formatter,
                "embedded request {specifier:?} from {referrer:?} is resolved more than once"
            ),
            Self::FingerprintLengthOverflow => {
                formatter.write_str("embedded graph field length exceeds its u64 framing")
            }
        }
    }
}

impl std::error::Error for EmbeddedModuleGraphError {}

fn utf16_cmp(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

fn normalized_virtual_path(path: &str) -> bool {
    !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_/.".contains(&byte)
        })
        && path
            .split('/')
            .all(|component| !matches!(component, "" | "." | ".."))
}

fn validate_identity(identity: &str) -> Result<(), EmbeddedModuleGraphError> {
    if normalized_virtual_path(identity) {
        Ok(())
    } else {
        Err(EmbeddedModuleGraphError::InvalidIdentity {
            identity: identity.to_owned(),
        })
    }
}

fn validate_meta_url(meta_url: &str) -> Result<(), EmbeddedModuleGraphError> {
    if meta_url
        .strip_prefix("lila://")
        .is_some_and(normalized_virtual_path)
    {
        Ok(())
    } else {
        Err(EmbeddedModuleGraphError::InvalidMetaUrl {
            meta_url: meta_url.to_owned(),
        })
    }
}

fn find_module<'a>(
    modules: &'a [EmbeddedModuleSource],
    identity: &str,
) -> Option<&'a EmbeddedModuleSource> {
    let index = modules
        .binary_search_by(|module| module.identity.as_str().cmp(identity))
        .ok()?;
    Some(&modules[index])
}

fn hash_count(hash: &mut Sha256, count: usize) -> Result<(), EmbeddedModuleGraphError> {
    let count =
        u64::try_from(count).map_err(|_| EmbeddedModuleGraphError::FingerprintLengthOverflow)?;
    hash.update(count.to_le_bytes());
    Ok(())
}

fn hash_field(hash: &mut Sha256, field: &str) -> Result<(), EmbeddedModuleGraphError> {
    hash_count(hash, field.len())?;
    hash.update(field.as_bytes());
    Ok(())
}

fn fingerprint(
    entry: &EmbeddedModuleEntry,
    modules: &[EmbeddedModuleSource],
    resolutions: &[EmbeddedModuleResolution],
) -> Result<[u8; 32], EmbeddedModuleGraphError> {
    let mut hash = Sha256::new();
    hash.update(b"lila-embedded-module-graph-v2");
    hash.update([match entry.goal {
        EmbeddedModuleGoal::Script => 0,
        EmbeddedModuleGoal::Module => 1,
    }]);
    hash_field(&mut hash, &entry.identity)?;
    hash_field(&mut hash, &entry.source)?;
    hash_field(&mut hash, &entry.meta_url)?;
    hash_count(&mut hash, modules.len())?;
    for module in modules {
        hash.update([match module.kind {
            EmbeddedModuleKind::SourceText => 0,
            EmbeddedModuleKind::Json => 1,
        }]);
        hash_field(&mut hash, &module.identity)?;
        hash_field(&mut hash, &module.source)?;
        hash_field(&mut hash, &module.meta_url)?;
    }
    hash_count(&mut hash, resolutions.len())?;
    for resolution in resolutions {
        match &resolution.referrer {
            EmbeddedModuleReferrer::Script(identity) => {
                hash.update([0]);
                hash_field(&mut hash, identity)?;
            }
            EmbeddedModuleReferrer::Module(identity) => {
                hash.update([1]);
                hash_field(&mut hash, identity)?;
            }
            EmbeddedModuleReferrer::Unlocated => hash.update([2]),
            EmbeddedModuleReferrer::Realm => hash.update([3]),
        }
        hash_field(&mut hash, resolution.request.specifier())?;
        hash_count(&mut hash, resolution.request.attributes().len())?;
        for (key, value) in resolution.request.attributes() {
            hash_field(&mut hash, key)?;
            hash_field(&mut hash, value)?;
        }
        hash_field(&mut hash, &resolution.target)?;
    }
    Ok(hash.finalize().into())
}
