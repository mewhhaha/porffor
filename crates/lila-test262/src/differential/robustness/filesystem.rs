//! The genuine resolver/load boundary, confined to worker-owned fixture paths.
use super::sandbox::Sandbox;
use lila_engine::{FilesystemModuleLoader, HostModuleLoader, ModuleKey, ModuleRequestKeyIr};
use lila_ir::ImportAttributeIr;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    ResolveAndLoad,
    LoadDirect,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Referrer {
    None,
    Entry,
    Nested,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Layout {
    Plain,
    Symlinks,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attribute {
    key: String,
    value: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u32,
    operation: Operation,
    referrer: Referrer,
    layout: Layout,
    specifier: String,
    attributes: Vec<Attribute>,
}
pub(super) struct FilesystemInput {
    operation: Operation,
    referrer: Referrer,
    layout: Layout,
    request: ModuleRequestKeyIr,
}
#[derive(Debug)]
pub(super) enum SetupError {
    Failure(String),
    Unavailable(String),
}
#[derive(Debug)]
pub(super) enum ProbeError {
    Rejected(String),
    Failure(String),
}
pub(super) struct FilesystemProbe {
    sandbox: Sandbox,
    root: PathBuf,
    input: FilesystemInput,
    loader: FilesystemModuleLoader,
}
impl FilesystemInput {
    pub(super) fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > 4 * 1024 {
            return Err("filesystem probe exceeds 4 KiB".into());
        }
        let wire: Wire = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if wire.schema_version != 1 || wire.specifier.len() > 1024 || wire.attributes.len() > 8 {
            return Err(
                "filesystem schema 1 requires at most 1 KiB specifier and 8 attributes".into(),
            );
        }
        if matches!(wire.operation, Operation::LoadDirect)
            && (!wire.attributes.is_empty() || !matches!(wire.referrer, Referrer::None))
        {
            return Err("direct load has no referrer or resolution attribute operand".into());
        }
        let request = ModuleRequestKeyIr::try_new(
            wire.specifier,
            wire.attributes
                .into_iter()
                .map(|attribute| ImportAttributeIr {
                    key: attribute.key,
                    value: attribute.value,
                }),
        )
        .map_err(|error| error.to_string())?;
        Ok(Self {
            operation: wire.operation,
            referrer: wire.referrer,
            layout: wire.layout,
            request,
        })
    }
    pub(super) fn setup(self, parent: &Path) -> Result<FilesystemProbe, SetupError> {
        let sandbox = Sandbox::new_in(parent).map_err(SetupError::Failure)?;
        let root = sandbox.root().join("root");
        let outside = sandbox.root().join("outside");
        let io = |error: std::io::Error| SetupError::Failure(error.to_string());
        std::fs::create_dir_all(root.join("nested")).map_err(io)?;
        std::fs::create_dir(&outside).map_err(io)?;
        for (name, contents) in [
            ("entry.js", "export const entry = 1;"),
            ("dep.js", "export default 'inside';"),
            ("nested/entry.js", "export const nested = 1;"),
            ("nested/dep.js", "export default 'nested';"),
            ("data.json", "{\"value\":1}"),
        ] {
            std::fs::write(root.join(name), contents).map_err(io)?;
        }
        std::fs::write(outside.join("dep.js"), "export default 'outside';").map_err(io)?;
        if matches!(self.layout, Layout::Symlinks) {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(root.join("nested"), root.join("inside-link")).map_err(
                    |error| {
                        SetupError::Unavailable(format!("symlink fixture unavailable: {error}"))
                    },
                )?;
                std::os::unix::fs::symlink(&outside, root.join("outside-link")).map_err(
                    |error| {
                        SetupError::Unavailable(format!("symlink fixture unavailable: {error}"))
                    },
                )?;
            }
            #[cfg(not(unix))]
            {
                return Err(SetupError::Unavailable(
                    "symlink probe is unavailable on this platform".into(),
                ));
            }
        }
        let entry = root.join("entry.js");
        let loader = FilesystemModuleLoader::new(
            Some(&root.to_string_lossy()),
            Some(&entry.to_string_lossy()),
        )
        .map_err(|error| SetupError::Failure(error.to_string()))?;
        Ok(FilesystemProbe {
            sandbox,
            root,
            input: self,
            loader,
        })
    }
}
impl FilesystemProbe {
    fn require_owned_key(&self, key: &ModuleKey) -> Result<(), ProbeError> {
        let root = self
            .root
            .canonicalize()
            .map_err(|error| ProbeError::Failure(error.to_string()))?;
        // A successful loader key must be normalized and inside the real root,
        // not merely share its string prefix.
        let path = Path::new(key.as_str());
        if !path.is_absolute() || !path.starts_with(root) {
            return Err(ProbeError::Failure(
                "filesystem resolver returned a key outside its owned root".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn resolve(&self) -> Result<ModuleKey, ProbeError> {
        let key = match self.input.operation {
            Operation::LoadDirect => self.loader.canonical_key(
                &self
                    .root
                    .join(self.input.request.specifier())
                    .to_string_lossy(),
            ),
            Operation::ResolveAndLoad => {
                let referrer = match self.input.referrer {
                    Referrer::None => None,
                    Referrer::Entry => Some(
                        self.loader
                            .canonical_key(&self.root.join("entry.js").to_string_lossy()),
                    ),
                    Referrer::Nested => Some(
                        self.loader
                            .canonical_key(&self.root.join("nested/entry.js").to_string_lossy()),
                    ),
                };
                self.loader
                    .resolve(referrer.as_ref(), &self.input.request)
                    .map_err(|error| ProbeError::Rejected(error.to_string()))?
            }
        };
        // Direct load deliberately submits an outside key to the original load
        // confinement check. Successful resolution already owes confinement.
        if matches!(self.input.operation, Operation::ResolveAndLoad) {
            self.require_owned_key(&key)?;
        }
        Ok(key)
    }
    pub(super) fn load(&self, key: &ModuleKey) -> Result<(), ProbeError> {
        let loaded = self
            .loader
            .load(key)
            .map_err(|error| ProbeError::Rejected(error.to_string()))?;
        self.require_owned_key(&loaded.key)?;
        if loaded.key != *key {
            return Err(ProbeError::Failure(
                "load changed the resolved module identity".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<(), String> {
        self.sandbox.finish()
    }
}
