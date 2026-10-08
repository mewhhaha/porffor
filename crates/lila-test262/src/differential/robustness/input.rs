use super::*;
use std::io::Read;

// Hex plus the fixed request envelope must fit the existing worker IO budget.
pub(super) const MAX_INPUT_BYTES: usize = worker_process::MAX_REQUEST_BYTES / 4;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinParserTarget {
    Json,
    RegExp,
    Number,
    BigInt,
    TemporalDate,
    EncodeUri,
    EncodeUriComponent,
    DecodeUri,
    DecodeUriComponent,
}
impl BuiltinParserTarget {
    pub(super) const fn is_uri(self) -> bool {
        match self {
            Self::EncodeUri
            | Self::EncodeUriComponent
            | Self::DecodeUri
            | Self::DecodeUriComponent => true,
            Self::Json | Self::RegExp | Self::Number | Self::BigInt | Self::TemporalDate => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RobustnessTarget {
    Compiler { goal: DifferentialGoal },
    // A struct variant makes serde consume the map and enforce unknown-field
    // rejection; an internally tagged unit variant discards foreign fields.
    IrAdmission {},
    Prelude {},
    FilesystemResolver {},
    Frontmatter {},
    Snapshot {},
    Corpus {},
    ModuleGraph {},
    ReportObservation {},
    Builtin { parser: BuiltinParserTarget },
}
impl RobustnessTarget {
    pub fn name(self) -> &'static str {
        match self {
            Self::Compiler {
                goal: DifferentialGoal::Script,
            } => "script",
            Self::Compiler {
                goal: DifferentialGoal::Module,
            } => "module",
            Self::IrAdmission {} => "ir-admission",
            Self::Prelude {} => "prelude",
            Self::FilesystemResolver {} => "filesystem-resolver",
            Self::Frontmatter {} => "frontmatter",
            Self::Snapshot {} => "snapshot",
            Self::Corpus {} => "corpus",
            Self::ModuleGraph {} => "module-graph",
            Self::ReportObservation {} => "report-observation",
            Self::Builtin {
                parser: BuiltinParserTarget::Json,
            } => "json",
            Self::Builtin {
                parser: BuiltinParserTarget::RegExp,
            } => "regexp",
            Self::Builtin {
                parser: BuiltinParserTarget::Number,
            } => "number",
            Self::Builtin {
                parser: BuiltinParserTarget::BigInt,
            } => "bigint",
            Self::Builtin {
                parser: BuiltinParserTarget::TemporalDate,
            } => "temporal-date",
            Self::Builtin {
                parser: BuiltinParserTarget::EncodeUri,
            } => "encode-uri",
            Self::Builtin {
                parser: BuiltinParserTarget::EncodeUriComponent,
            } => "encode-uri-component",
            Self::Builtin {
                parser: BuiltinParserTarget::DecodeUri,
            } => "decode-uri",
            Self::Builtin {
                parser: BuiltinParserTarget::DecodeUriComponent,
            } => "decode-uri-component",
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "script" => Self::Compiler {
                goal: DifferentialGoal::Script,
            },
            "module" => Self::Compiler {
                goal: DifferentialGoal::Module,
            },
            "ir-admission" => Self::IrAdmission {},
            "prelude" => Self::Prelude {},
            "filesystem-resolver" => Self::FilesystemResolver {},
            "frontmatter" => Self::Frontmatter {},
            "snapshot" => Self::Snapshot {},
            "corpus" => Self::Corpus {},
            "module-graph" => Self::ModuleGraph {},
            "report-observation" => Self::ReportObservation {},
            "json" => Self::Builtin {
                parser: BuiltinParserTarget::Json,
            },
            "regexp" => Self::Builtin {
                parser: BuiltinParserTarget::RegExp,
            },
            "number" => Self::Builtin {
                parser: BuiltinParserTarget::Number,
            },
            "bigint" => Self::Builtin {
                parser: BuiltinParserTarget::BigInt,
            },
            "temporal-date" => Self::Builtin {
                parser: BuiltinParserTarget::TemporalDate,
            },
            "encode-uri" => Self::Builtin {
                parser: BuiltinParserTarget::EncodeUri,
            },
            "encode-uri-component" => Self::Builtin {
                parser: BuiltinParserTarget::EncodeUriComponent,
            },
            "decode-uri" => Self::Builtin {
                parser: BuiltinParserTarget::DecodeUri,
            },
            "decode-uri-component" => Self::Builtin {
                parser: BuiltinParserTarget::DecodeUriComponent,
            },
            _ => return None,
        })
    }
    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(in crate::differential) fn stages(self) -> &'static [RobustnessStage] {
        use RobustnessStage::*;
        match self {
            Self::Compiler { .. } => &[
                Decode,
                Preparation,
                Lowering,
                Emission,
                RuntimeSetup,
                Validation,
            ],
            Self::IrAdmission {} => &[
                Decode,
                IrInput,
                Preparation,
                Lowering,
                IrAdmission,
                Emission,
                RuntimeSetup,
                Validation,
            ],
            Self::Builtin { parser } if parser.is_uri() => &[
                Decode,
                BuiltinInput,
                Preparation,
                Lowering,
                Emission,
                RuntimeSetup,
                Validation,
                BuiltinExecution,
            ],
            Self::Builtin { .. } => &[
                Decode,
                Preparation,
                Lowering,
                Emission,
                RuntimeSetup,
                Validation,
                BuiltinExecution,
            ],
            Self::Frontmatter {} => &[Decode, Frontmatter],
            Self::Prelude {} => &[Decode, PreludeInput, PreludeLoad, PreludeMaterialization],
            Self::FilesystemResolver {} => &[
                Decode,
                FilesystemInput,
                FilesystemSetup,
                ModuleResolution,
                ModuleLoading,
            ],
            Self::Snapshot {} => &[Decode, Snapshot],
            Self::Corpus {} => &[Decode, Corpus],
            Self::ModuleGraph {} => &[Decode, ModuleGraph],
            Self::ReportObservation {} => &[Decode, ReportObservation],
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobustnessInput {
    id: DifferentialCaseId,
    target: RobustnessTarget,
    timeout_ms: NonZeroU64,
    bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputWire {
    schema_version: u32,
    id: String,
    target: RobustnessTarget,
    timeout_ms: u64,
    bytes_hex: String,
}
impl RobustnessInput {
    pub fn new(
        id: impl Into<String>,
        target: RobustnessTarget,
        timeout_ms: u64,
        bytes: Vec<u8>,
    ) -> Result<Self, DifferentialError> {
        if bytes.len() > MAX_INPUT_BYTES {
            return Err(DifferentialError::InvalidCorpus(
                "robustness bytes exceed the existing worker request budget".into(),
            ));
        }
        Ok(Self {
            id: DifferentialCaseId::new(id)?,
            target,
            timeout_ms: NonZeroU64::new(timeout_ms).ok_or_else(|| {
                DifferentialError::InvalidCorpus("robustness timeout must be positive".into())
            })?,
            bytes,
        })
    }
    pub fn id(&self) -> &DifferentialCaseId {
        &self.id
    }
    pub fn target(&self) -> RobustnessTarget {
        self.target
    }
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms.get()
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn from_bytes_file(
        id: impl Into<String>,
        target: RobustnessTarget,
        timeout_ms: u64,
        path: impl AsRef<Path>,
    ) -> Result<Self, DifferentialError> {
        let path = path.as_ref();
        let file = fs::File::open(path).map_err(|error| DifferentialError::ReadCorpus {
            path: path.into(),
            message: error.to_string(),
        })?;
        let mut bytes = Vec::new();
        file.take(MAX_INPUT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| DifferentialError::ReadCorpus {
                path: path.into(),
                message: error.to_string(),
            })?;
        Self::new(id, target, timeout_ms, bytes)
    }
    pub fn from_json(json: &str) -> Result<Self, DifferentialError> {
        if json.len() > worker_process::MAX_REQUEST_BYTES {
            return Err(DifferentialError::InvalidCorpus(
                "robustness request exceeds worker IO budget".into(),
            ));
        }
        let wire: InputWire = serde_json::from_str(json)
            .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
        if wire.schema_version != 1
            || wire.bytes_hex.len() > MAX_INPUT_BYTES * 2
            || wire.bytes_hex.len() % 2 != 0
        {
            return Err(DifferentialError::InvalidCorpus(
                "unsupported robustness schema or noncanonical byte length".into(),
            ));
        }
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        let bytes = wire
            .bytes_hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| Some(digit(pair[0])? * 16 + digit(pair[1])?))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                DifferentialError::InvalidCorpus(
                    "robustness bytes require canonical lowercase hex".into(),
                )
            })?;
        Self::new(wire.id, wire.target, wire.timeout_ms, bytes)
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DifferentialError> {
        let path = path.as_ref();
        let file = fs::File::open(path).map_err(|error| DifferentialError::ReadCorpus {
            path: path.into(),
            message: error.to_string(),
        })?;
        let mut json = String::new();
        file.take(worker_process::MAX_REQUEST_BYTES as u64 + 1)
            .read_to_string(&mut json)
            .map_err(|error| DifferentialError::ReadCorpus {
                path: path.into(),
                message: error.to_string(),
            })?;
        Self::from_json(&json)
    }
    pub fn to_pretty_json(&self) -> Result<String, DifferentialError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| DifferentialError::EncodeCorpus(error.to_string()))
    }
    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(super) fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(b"lila-robustness-v1");
        let timeout = self.timeout_ms().to_le_bytes();
        for field in [
            self.target.name().as_bytes(),
            timeout.as_slice(),
            self.bytes.as_slice(),
        ] {
            hash.update((field.len() as u64).to_le_bytes());
            hash.update(field);
        }
        let spelling: String = hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!("robustness-v1-{spelling}")
    }
}
impl Serialize for RobustnessInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let bytes_hex: String = self
            .bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        InputWire {
            schema_version: 1,
            id: self.id.as_str().into(),
            target: self.target,
            timeout_ms: self.timeout_ms(),
            bytes_hex,
        }
        .serialize(serializer)
    }
}
