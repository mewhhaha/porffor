//! Closed stateful source plans and execution-backed metamorphic relations.
use super::*;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub const BUILTIN_STATEFUL_GRAMMAR: &str = "builtin-stateful-v1";
pub const METAMORPHIC_STATEFUL_GRAMMAR: &str = "metamorphic-stateful-v1";
pub const BUILTIN_STATEFUL_GRAMMAR_V2: &str = "builtin-stateful-v2";
pub const METAMORPHIC_STATEFUL_GRAMMAR_V2: &str = "metamorphic-stateful-v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioGrammar {
    StatefulV1,
    MetamorphicV1,
    StatefulV2,
    MetamorphicV2,
}
impl ScenarioGrammar {
    pub const fn name(self) -> &'static str {
        match self {
            Self::StatefulV1 => BUILTIN_STATEFUL_GRAMMAR,
            Self::MetamorphicV1 => METAMORPHIC_STATEFUL_GRAMMAR,
            Self::StatefulV2 => BUILTIN_STATEFUL_GRAMMAR_V2,
            Self::MetamorphicV2 => METAMORPHIC_STATEFUL_GRAMMAR_V2,
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            BUILTIN_STATEFUL_GRAMMAR => Some(Self::StatefulV1),
            METAMORPHIC_STATEFUL_GRAMMAR => Some(Self::MetamorphicV1),
            BUILTIN_STATEFUL_GRAMMAR_V2 => Some(Self::StatefulV2),
            METAMORPHIC_STATEFUL_GRAMMAR_V2 => Some(Self::MetamorphicV2),
            _ => None,
        }
    }
    const fn family_count(self) -> u64 {
        match self {
            Self::StatefulV1 | Self::MetamorphicV1 => 6,
            Self::StatefulV2 | Self::MetamorphicV2 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioFamily {
    ArrayAndBuffer,
    IteratorClose,
    Collections,
    ProxyAndCoercion,
    SubclassAndSpecies,
    PromiseJobs,
    CrossRealm,
    Temporal,
}
impl ScenarioFamily {
    fn for_plan(plan: ScenarioGenerationPlan) -> Self {
        [
            Self::ArrayAndBuffer,
            Self::IteratorClose,
            Self::Collections,
            Self::ProxyAndCoercion,
            Self::SubclassAndSpecies,
            Self::PromiseJobs,
            Self::CrossRealm,
            Self::Temporal,
        ][(plan.seed % plan.grammar.family_count()) as usize]
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::ArrayAndBuffer => "array-buffer",
            Self::IteratorClose => "iterator-close",
            Self::Collections => "collections",
            Self::ProxyAndCoercion => "proxy-coercion",
            Self::SubclassAndSpecies => "subclass-species",
            Self::PromiseJobs => "promise-jobs",
            Self::CrossRealm => "cross-realm",
            Self::Temporal => "temporal",
        }
    }
    fn protocol(self) -> DifferentialProtocol {
        match self {
            Self::CrossRealm => DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            Self::ArrayAndBuffer
            | Self::IteratorClose
            | Self::Collections
            | Self::ProxyAndCoercion
            | Self::SubclassAndSpecies
            | Self::PromiseJobs
            | Self::Temporal => DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetamorphicTransformation {
    BindingRename,
    NeutralBlocks,
    EquivalentFiniteLoop,
}
impl MetamorphicTransformation {
    fn for_plan(plan: ScenarioGenerationPlan) -> Self {
        [
            Self::BindingRename,
            Self::NeutralBlocks,
            Self::EquivalentFiniteLoop,
        ][((plan.seed / plan.grammar.family_count()) % 3) as usize]
    }
}

/// Reuses the established control-flow step budget; no second workload ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioGenerationPlan {
    grammar: ScenarioGrammar,
    seed: u64,
    steps: u8,
}
impl ScenarioGenerationPlan {
    pub fn new(
        grammar: ScenarioGrammar,
        seed: u64,
        steps: usize,
    ) -> Result<Self, DifferentialError> {
        let steps = u8::try_from(steps)
            .ok()
            .filter(|steps| *steps > 0 && usize::from(*steps) <= MAX_CONTROL_FLOW_STEPS)
            .ok_or_else(|| {
                DifferentialError::InvalidGeneration(format!(
                    "stateful scenario steps must be in 1..={MAX_CONTROL_FLOW_STEPS}"
                ))
            })?;
        Ok(Self {
            grammar,
            seed,
            steps,
        })
    }
    pub const fn grammar(self) -> ScenarioGrammar {
        self.grammar
    }
    pub const fn seed(self) -> u64 {
        self.seed
    }
    pub const fn steps(self) -> u8 {
        self.steps
    }
    pub fn family(self) -> ScenarioFamily {
        ScenarioFamily::for_plan(self)
    }
    pub fn transformation(self) -> Option<MetamorphicTransformation> {
        match self.grammar {
            ScenarioGrammar::StatefulV1 | ScenarioGrammar::StatefulV2 => None,
            ScenarioGrammar::MetamorphicV1 | ScenarioGrammar::MetamorphicV2 => {
                Some(MetamorphicTransformation::for_plan(self))
            }
        }
    }
    fn strict(self) -> bool {
        self.seed / (self.grammar.family_count() * 3) % 2 == 1
    }
    pub(super) fn with_seed(self, seed: u64) -> Self {
        Self { seed, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
struct SmallValue(i8);
impl SmallValue {
    fn valid(self) -> bool {
        (-16..=16).contains(&self.0)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
struct Slot(u8);
impl Slot {
    fn valid(self) -> bool {
        self.0 < 8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArrayOperation {
    Push,
    Pop,
    Splice,
    CopyWithin,
    SetLength,
    TypedWrite,
    ViewWrite,
    Observe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum IteratorOperation {
    Break,
    Throw,
    Pattern,
    TargetThrow,
    ReturnThrows,
    GetterReturn,
    ContinueThenBreak,
    NestedClose,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CollectionOperation {
    MapSet,
    MapDelete,
    SetAdd,
    SetDelete,
    MapForEach,
    SetForEach,
    Clear,
    Observe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProxyOperation {
    Get,
    Put,
    Compound,
    Has,
    Define,
    Delete,
    Keys,
    Coerce,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpeciesOperation {
    Map,
    Filter,
    Slice,
    Splice,
    Concat,
    TypedMap,
    TypedSlice,
    Observe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum PromiseOperation {
    Then,
    Thenable,
    Recover,
    Finally,
    NestedJobs,
    Await,
    All,
    Race,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum RealmOperation {
    ArrayMap,
    ArraySpecies,
    BoxPrimitive,
    ErrorRealm,
    MapBrand,
    TypedArray,
    ReflectConstruct,
    AbruptIdentity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum TemporalOperation {
    DateAdd,
    DateWith,
    InstantAdd,
    ZonedAdd,
    DurationRound,
    DurationTotal,
    ConversionOrder,
    AbruptIdentity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}
const CHOICES: [Choice; 8] = [
    Choice::First,
    Choice::Second,
    Choice::Third,
    Choice::Fourth,
    Choice::Fifth,
    Choice::Sixth,
    Choice::Seventh,
    Choice::Eighth,
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    Array(ArrayOperation),
    Iterator(IteratorOperation),
    Collection(CollectionOperation),
    Proxy(ProxyOperation),
    Species(SpeciesOperation),
    Promise(PromiseOperation),
    Realm(RealmOperation),
    Temporal(TemporalOperation),
}
impl Operation {
    fn family(self) -> ScenarioFamily {
        match self {
            Self::Array(_) => ScenarioFamily::ArrayAndBuffer,
            Self::Iterator(_) => ScenarioFamily::IteratorClose,
            Self::Collection(_) => ScenarioFamily::Collections,
            Self::Proxy(_) => ScenarioFamily::ProxyAndCoercion,
            Self::Species(_) => ScenarioFamily::SubclassAndSpecies,
            Self::Promise(_) => ScenarioFamily::PromiseJobs,
            Self::Realm(_) => ScenarioFamily::CrossRealm,
            Self::Temporal(_) => ScenarioFamily::Temporal,
        }
    }
    fn selected(family: ScenarioFamily, choice: Choice) -> Self {
        // The actual source action is closed; source text is never supplied by a seed.
        match family {
            ScenarioFamily::ArrayAndBuffer => Self::Array(match choice {
                Choice::First => ArrayOperation::Push,
                Choice::Second => ArrayOperation::Pop,
                Choice::Third => ArrayOperation::Splice,
                Choice::Fourth => ArrayOperation::CopyWithin,
                Choice::Fifth => ArrayOperation::SetLength,
                Choice::Sixth => ArrayOperation::TypedWrite,
                Choice::Seventh => ArrayOperation::ViewWrite,
                Choice::Eighth => ArrayOperation::Observe,
            }),
            ScenarioFamily::IteratorClose => Self::Iterator(match choice {
                Choice::First => IteratorOperation::Break,
                Choice::Second => IteratorOperation::Throw,
                Choice::Third => IteratorOperation::Pattern,
                Choice::Fourth => IteratorOperation::TargetThrow,
                Choice::Fifth => IteratorOperation::ReturnThrows,
                Choice::Sixth => IteratorOperation::GetterReturn,
                Choice::Seventh => IteratorOperation::ContinueThenBreak,
                Choice::Eighth => IteratorOperation::NestedClose,
            }),
            ScenarioFamily::Collections => Self::Collection(match choice {
                Choice::First => CollectionOperation::MapSet,
                Choice::Second => CollectionOperation::MapDelete,
                Choice::Third => CollectionOperation::SetAdd,
                Choice::Fourth => CollectionOperation::SetDelete,
                Choice::Fifth => CollectionOperation::MapForEach,
                Choice::Sixth => CollectionOperation::SetForEach,
                Choice::Seventh => CollectionOperation::Clear,
                Choice::Eighth => CollectionOperation::Observe,
            }),
            ScenarioFamily::ProxyAndCoercion => Self::Proxy(match choice {
                Choice::First => ProxyOperation::Get,
                Choice::Second => ProxyOperation::Put,
                Choice::Third => ProxyOperation::Compound,
                Choice::Fourth => ProxyOperation::Has,
                Choice::Fifth => ProxyOperation::Define,
                Choice::Sixth => ProxyOperation::Delete,
                Choice::Seventh => ProxyOperation::Keys,
                Choice::Eighth => ProxyOperation::Coerce,
            }),
            ScenarioFamily::SubclassAndSpecies => Self::Species(match choice {
                Choice::First => SpeciesOperation::Map,
                Choice::Second => SpeciesOperation::Filter,
                Choice::Third => SpeciesOperation::Slice,
                Choice::Fourth => SpeciesOperation::Splice,
                Choice::Fifth => SpeciesOperation::Concat,
                Choice::Sixth => SpeciesOperation::TypedMap,
                Choice::Seventh => SpeciesOperation::TypedSlice,
                Choice::Eighth => SpeciesOperation::Observe,
            }),
            ScenarioFamily::PromiseJobs => Self::Promise(match choice {
                Choice::First => PromiseOperation::Then,
                Choice::Second => PromiseOperation::Thenable,
                Choice::Third => PromiseOperation::Recover,
                Choice::Fourth => PromiseOperation::Finally,
                Choice::Fifth => PromiseOperation::NestedJobs,
                Choice::Sixth => PromiseOperation::Await,
                Choice::Seventh => PromiseOperation::All,
                Choice::Eighth => PromiseOperation::Race,
            }),
            ScenarioFamily::CrossRealm => Self::Realm(match choice {
                Choice::First => RealmOperation::ArrayMap,
                Choice::Second => RealmOperation::ArraySpecies,
                Choice::Third => RealmOperation::BoxPrimitive,
                Choice::Fourth => RealmOperation::ErrorRealm,
                Choice::Fifth => RealmOperation::MapBrand,
                Choice::Sixth => RealmOperation::TypedArray,
                Choice::Seventh => RealmOperation::ReflectConstruct,
                Choice::Eighth => RealmOperation::AbruptIdentity,
            }),
            ScenarioFamily::Temporal => Self::Temporal(match choice {
                Choice::First => TemporalOperation::DateAdd,
                Choice::Second => TemporalOperation::DateWith,
                Choice::Third => TemporalOperation::InstantAdd,
                Choice::Fourth => TemporalOperation::ZonedAdd,
                Choice::Fifth => TemporalOperation::DurationRound,
                Choice::Sixth => TemporalOperation::DurationTotal,
                Choice::Seventh => TemporalOperation::ConversionOrder,
                Choice::Eighth => TemporalOperation::AbruptIdentity,
            }),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    operation: Operation,
    slot: Slot,
    value: SmallValue,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    family: ScenarioFamily,
    strict: bool,
    actions: Vec<Action>,
}
impl Program {
    fn new(family: ScenarioFamily, strict: bool, actions: Vec<Action>) -> Option<Self> {
        (!actions.is_empty()
            && actions.len() <= MAX_CONTROL_FLOW_STEPS
            && actions.iter().all(|action| {
                action.operation.family() == family && action.slot.valid() && action.value.valid()
            }))
        .then_some(Self {
            family,
            strict,
            actions,
        })
    }
    fn complexity(&self) -> (usize, usize) {
        (
            self.actions.len(),
            self.actions
                .iter()
                .map(|action| {
                    usize::from(action.value.0.unsigned_abs()) + usize::from(action.slot.0)
                })
                .sum(),
        )
    }
    fn reductions(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut push = |actions| {
            if let Some(candidate) = Self::new(self.family, self.strict, actions) {
                if candidate.complexity() < self.complexity() && !result.contains(&candidate) {
                    result.push(candidate);
                }
            }
        };
        for index in 0..self.actions.len() {
            if self.actions.len() > 1 {
                let mut actions = self.actions.clone();
                actions.remove(index);
                push(actions);
            }
            if self.actions[index].value.0 != 0 {
                let mut actions = self.actions.clone();
                actions[index].value = SmallValue(0);
                push(actions);
            }
            if self.actions[index].slot.0 != 0 {
                let mut actions = self.actions.clone();
                actions[index].slot = Slot(0);
                push(actions);
            }
        }
        result
    }
    fn cases(
        &self,
        plan: ScenarioGenerationPlan,
    ) -> Result<GeneratedScenarioCases, DifferentialError> {
        if self.family != plan.family()
            || self.strict != plan.strict()
            || self.actions.len() > usize::from(plan.steps)
        {
            return Err(DifferentialError::GeneratorInvariant(
                "scenario reduction changed its family, strictness or source budget".into(),
            ));
        }
        let case = |variant, transformation| {
            let stem = format!(
                "seed-{:016x}-{}-steps-{:02}-{variant}",
                plan.seed,
                self.family.name(),
                plan.steps
            );
            DifferentialReplayInput::new_script(
                format!("t25/generated/{}/{stem}", plan.grammar.name()),
                self.family.protocol(),
                format!(
                    "differential/v{}/generated/{}/{stem}.js",
                    self.family.protocol().schema_version(),
                    plan.grammar.name()
                ),
                5_000,
                render::source(self, transformation),
            )
        };
        Ok(GeneratedScenarioCases {
            baseline: case("baseline", None)?,
            transformed: plan
                .transformation()
                .map(|transformation| case("transformed", Some(transformation)))
                .transpose()?,
            transformation: plan.transformation(),
        })
    }
}
fn generate(plan: ScenarioGenerationPlan) -> Result<Program, DifferentialError> {
    let mut random = plan.seed ^ 0x9e3779b97f4a7c15;
    let mut next = || {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        random
    };
    let actions = (0..plan.steps)
        .map(|_| Action {
            operation: Operation::selected(plan.family(), CHOICES[(next() % 8) as usize]),
            slot: Slot((next() % 8) as u8),
            value: SmallValue((next() % 33) as i8 - 16),
        })
        .collect();
    Program::new(plan.family(), plan.strict(), actions).ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "scenario generator escaped its closed action plan".into(),
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedScenarioCases {
    baseline: DifferentialReplayInput,
    transformed: Option<DifferentialReplayInput>,
    transformation: Option<MetamorphicTransformation>,
}
impl GeneratedScenarioCases {
    pub fn baseline(&self) -> &DifferentialReplayInput {
        &self.baseline
    }
    pub fn transformed(&self) -> Option<&DifferentialReplayInput> {
        self.transformed.as_ref()
    }
    pub fn transformation(&self) -> Option<MetamorphicTransformation> {
        self.transformation
    }
}
pub fn generate_scenario_cases(
    plan: ScenarioGenerationPlan,
) -> Result<GeneratedScenarioCases, DifferentialError> {
    generate(plan)?.cases(plan)
}

/// These are observed dimensions, never a prediction of what a program returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ScenarioDifference {
    pub completion: bool,
    pub primitive_value: bool,
    pub print: bool,
    pub baseline_kind: CompletionKindObservation,
    pub transformed_kind: CompletionKindObservation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum MetamorphicVerdict {
    NotRequested,
    ObservationsMatch,
    Mismatch {
        wasm_aot: ScenarioDifference,
        spec_exec: ScenarioDifference,
    },
    Unavailable,
}
#[derive(Debug)]
pub struct ScenarioObservation {
    case: DifferentialReplayInput,
    report: DifferentialReport,
    expected_steps: usize,
}
impl ScenarioObservation {
    pub fn case(&self) -> &DifferentialReplayInput {
        &self.case
    }
    pub fn report(&self) -> &DifferentialReport {
        &self.report
    }
}
impl Serialize for ScenarioObservation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // The validated replay request remains the sole wire producer. The
        // case record retains its exact source alongside the untouched report.
        let case_json = self
            .case
            .to_pretty_json()
            .map_err(<S::Error as serde::ser::Error>::custom)?;
        let case: serde_json::Value =
            serde_json::from_str(&case_json).map_err(<S::Error as serde::ser::Error>::custom)?;
        let mut record = serializer.serialize_struct("ScenarioObservation", 2)?;
        record.serialize_field("case", &case)?;
        record.serialize_field("report", &self.report)?;
        record.end()
    }
}
#[derive(Debug)]
struct PendingScenarioInput(DifferentialReplayInput);
impl Serialize for PendingScenarioInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let json = self
            .0
            .to_pretty_json()
            .map_err(<S::Error as serde::ser::Error>::custom)?;
        let case: serde_json::Value =
            serde_json::from_str(&json).map_err(<S::Error as serde::ser::Error>::custom)?;
        case.serialize(serializer)
    }
}
#[derive(Debug, Serialize)]
pub struct ScenarioObservations {
    baseline: ScenarioObservation,
    transformed: Option<ScenarioObservation>,
    transformation: Option<MetamorphicTransformation>,
    metamorphic: MetamorphicVerdict,
    pending_transformed: Option<PendingScenarioInput>,
    #[serde(skip)]
    replay_pair: Option<ScenarioReplayPair>,
}
impl ScenarioObservations {
    pub fn baseline(&self) -> &ScenarioObservation {
        &self.baseline
    }
    pub fn transformed(&self) -> Option<&ScenarioObservation> {
        self.transformed.as_ref()
    }
    pub fn transformation(&self) -> Option<MetamorphicTransformation> {
        self.transformation
    }
    pub fn metamorphic(&self) -> MetamorphicVerdict {
        self.metamorphic
    }
    pub fn pending_transformed(&self) -> Option<&DifferentialReplayInput> {
        self.pending_transformed.as_ref().map(|pending| &pending.0)
    }
    pub fn replay_pair(&self) -> Option<&ScenarioReplayPair> {
        self.replay_pair.as_ref()
    }
    pub fn is_green(&self) -> bool {
        let relation_shape = matches!(
            (&self.transformed, self.transformation, self.metamorphic),
            (None, None, MetamorphicVerdict::NotRequested)
                | (Some(_), Some(_), MetamorphicVerdict::ObservationsMatch)
        );
        relation_shape
            && self.pending_transformed.is_none()
            && self.metamorphic
                == metamorphic(
                    &self.baseline.report,
                    self.transformed
                        .as_ref()
                        .map(|observation| &observation.report),
                )
            && self.baseline.report.verdict()
                == DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
            && self.transformed.as_ref().is_none_or(|observation| {
                observation.report.verdict()
                    == DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
            })
            && finished(&self.baseline)
            && self.transformed.as_ref().is_none_or(finished)
    }
}
fn primitive(
    observation: &BackendObservation,
) -> Option<(&PrimitiveCompletionObservation, &[String])> {
    let (
        ExecutionObservation::PrimitiveCompletion { completion, .. },
        OutputEventsObservation::Captured { events },
    ) = (&observation.execution, &observation.output_events)
    else {
        return None;
    };
    Some((completion, events))
}
fn primitive_value(completion: &PrimitiveCompletionObservation) -> &PrimitiveValueObservation {
    match completion {
        PrimitiveCompletionObservation::Normal { value }
        | PrimitiveCompletionObservation::Throw { value } => value,
    }
}
fn difference(
    baseline: &BackendObservation,
    transformed: &BackendObservation,
) -> Option<ScenarioDifference> {
    if baseline.backend != transformed.backend
        || baseline.worker_identity != transformed.worker_identity
    {
        return None;
    }
    let (left, a) = primitive(baseline)?;
    let (right, b) = primitive(transformed)?;
    Some(ScenarioDifference {
        completion: left != right,
        primitive_value: primitive_value(left) != primitive_value(right),
        print: a != b,
        baseline_kind: left.kind(),
        transformed_kind: right.kind(),
    })
}
fn metamorphic(
    baseline: &DifferentialReport,
    transformed: Option<&DifferentialReport>,
) -> MetamorphicVerdict {
    let Some(transformed) = transformed else {
        return MetamorphicVerdict::NotRequested;
    };
    let (Some(wasm_aot), Some(spec_exec)) = (
        difference(baseline.wasm_aot(), transformed.wasm_aot()),
        difference(baseline.spec_exec(), transformed.spec_exec()),
    ) else {
        return MetamorphicVerdict::Unavailable;
    };
    if !wasm_aot.completion && !wasm_aot.print && !spec_exec.completion && !spec_exec.print {
        MetamorphicVerdict::ObservationsMatch
    } else {
        MetamorphicVerdict::Mismatch {
            wasm_aot,
            spec_exec,
        }
    }
}
fn finished(observation: &ScenarioObservation) -> bool {
    // Check only harness progress, not any expected value or hook trace.
    let source_steps = observation.expected_steps;
    let check = |backend| {
        let Some((completion, events)) = primitive(backend) else {
            return false;
        };
        completion.kind() == CompletionKindObservation::Normal
            && events
                .last()
                .is_some_and(|event| event.starts_with("scenario-done:"))
            && events
                .iter()
                .filter(|event| event.starts_with("scenario-step:"))
                .eq((0..source_steps)
                    .map(|index| format!("scenario-step:{index}"))
                    .collect::<Vec<_>>()
                    .iter())
    };
    source_steps > 0
        && check(observation.report.wasm_aot())
        && check(observation.report.spec_exec())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DifferentialWitness {
    Match {
        kind: CompletionKindObservation,
    },
    Compared {
        completion: bool,
        primitive_value: bool,
        print: bool,
        wasm_kind: CompletionKindObservation,
        spec_kind: CompletionKindObservation,
    },
    WasmFailed {
        phase: FailurePhase,
        spec_kind: CompletionKindObservation,
    },
    SpecFailed {
        phase: FailurePhase,
        wasm_kind: CompletionKindObservation,
    },
}
impl DifferentialWitness {
    fn from_report(report: &DifferentialReport) -> Option<Self> {
        match report.protocol() {
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript => {}
            DifferentialProtocol::V1SelfCheckingNoOutput
            | DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V5SelectedObjectProbePrintTranscript
            | DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript => {
                return None
            }
        }
        match report.verdict() {
            DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch => Some(Self::Match {
                kind: primitive(report.wasm_aot())?.0.kind(),
            }),
            DifferentialVerdict::Mismatch => {
                match (&report.wasm_aot().execution, &report.spec_exec().execution) {
                    (
                        ExecutionObservation::PrimitiveCompletion {
                            completion: left, ..
                        },
                        ExecutionObservation::PrimitiveCompletion {
                            completion: right, ..
                        },
                    ) => {
                        let (_, a) = primitive(report.wasm_aot())?;
                        let (_, b) = primitive(report.spec_exec())?;
                        Some(Self::Compared {
                            completion: left != right,
                            primitive_value: primitive_value(left) != primitive_value(right),
                            print: a != b,
                            wasm_kind: left.kind(),
                            spec_kind: right.kind(),
                        })
                    }
                    (
                        ExecutionObservation::EngineFailure { phase, .. },
                        ExecutionObservation::PrimitiveCompletion { completion, .. },
                    ) => Some(Self::WasmFailed {
                        phase: *phase,
                        spec_kind: completion.kind(),
                    }),
                    (
                        ExecutionObservation::PrimitiveCompletion { completion, .. },
                        ExecutionObservation::EngineFailure { phase, .. },
                    ) => Some(Self::SpecFailed {
                        phase: *phase,
                        wasm_kind: completion.kind(),
                    }),
                    (
                        ExecutionObservation::EngineFailure { .. },
                        ExecutionObservation::EngineFailure { .. },
                    ) => None,
                    (
                        ExecutionObservation::Normal { .. }
                        | ExecutionObservation::Error { .. }
                        | ExecutionObservation::SelectedObjectProbe { .. }
                        | ExecutionObservation::RootedCompletionGraph { .. }
                        | ExecutionObservation::ObservationRejected { .. }
                        | ExecutionObservation::UnsupportedCompletion { .. }
                        | ExecutionObservation::WorkerFailure { .. },
                        _,
                    )
                    | (
                        _,
                        ExecutionObservation::Normal { .. }
                        | ExecutionObservation::Error { .. }
                        | ExecutionObservation::SelectedObjectProbe { .. }
                        | ExecutionObservation::RootedCompletionGraph { .. }
                        | ExecutionObservation::ObservationRejected { .. }
                        | ExecutionObservation::UnsupportedCompletion { .. }
                        | ExecutionObservation::WorkerFailure { .. },
                    ) => None,
                }
            }
            DifferentialVerdict::BothCompleted
            | DifferentialVerdict::PrimitiveCompletionsMatch
            | DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
            | DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch
            | DifferentialVerdict::BothFailed
            | DifferentialVerdict::ObservationContractViolated
            | DifferentialVerdict::WorkerFailure => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Witness {
    baseline: DifferentialWitness,
    transformed: Option<DifferentialWitness>,
    metamorphic: MetamorphicVerdict,
}
impl Witness {
    fn from_observations(observations: &ScenarioObservations) -> Option<Self> {
        let baseline = DifferentialWitness::from_report(&observations.baseline.report)?;
        let transformed = match observations.transformed.as_ref() {
            Some(observation) => Some(DifferentialWitness::from_report(&observation.report)?),
            None => None,
        };
        if matches!(baseline, DifferentialWitness::Match { .. })
            && transformed
                .is_none_or(|witness| matches!(witness, DifferentialWitness::Match { .. }))
            && !matches!(
                observations.metamorphic,
                MetamorphicVerdict::Mismatch { .. }
            )
        {
            return None;
        }
        Some(Self {
            baseline,
            transformed,
            metamorphic: observations.metamorphic,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ScenarioReductionSummary {
    attempted_replays: u16,
    accepted_reductions: u16,
    stop: ArithmeticReductionStop,
}
impl ScenarioReductionSummary {
    pub const fn attempted_replays(self) -> u16 {
        self.attempted_replays
    }
    pub const fn accepted_reductions(self) -> u16 {
        self.accepted_reductions
    }
    pub const fn stop(self) -> ArithmeticReductionStop {
        self.stop
    }
}
pub(super) enum Outcome {
    Verified {
        observations: ScenarioObservations,
    },
    ReducedMismatch {
        observations: ScenarioObservations,
        reduction: ScenarioReductionSummary,
    },
    Rejected {
        observations: ScenarioObservations,
    },
}
fn replay_program(
    program: &Program,
    plan: ScenarioGenerationPlan,
    replay: &mut impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<ScenarioObservations, DifferentialError> {
    let cases = program.cases(plan)?;
    let replay_pair = if plan.transformation().is_some() {
        Some(ScenarioReplayPair::from_program(program.clone(), plan)?)
    } else {
        None
    };
    let observe = |case: DifferentialReplayInput,
                   replay: &mut dyn FnMut(
        &DifferentialReplayInput,
    ) -> Result<DifferentialReport, DifferentialError>| {
        let report = replay(&case)?;
        if report.case_id() != case.id()
            || report.case_fingerprint != input_fingerprint(&case)
            || report.protocol() != case.protocol()
        {
            return Err(DifferentialError::GeneratorInvariant(
                "scenario replay returned a foreign source/protocol observation".into(),
            ));
        }
        Ok(ScenarioObservation {
            case,
            report,
            expected_steps: program.actions.len(),
        })
    };
    let baseline = observe(cases.baseline, replay)?;
    if baseline.report.verdict() == DifferentialVerdict::WorkerFailure {
        return Ok(ScenarioObservations {
            baseline,
            transformed: None,
            pending_transformed: cases.transformed.map(PendingScenarioInput),
            transformation: cases.transformation,
            metamorphic: if cases.transformation.is_some() {
                MetamorphicVerdict::Unavailable
            } else {
                MetamorphicVerdict::NotRequested
            },
            replay_pair,
        });
    }
    let transformed = cases
        .transformed
        .map(|case| observe(case, replay))
        .transpose()?;
    let metamorphic = metamorphic(
        &baseline.report,
        transformed.as_ref().map(|observation| &observation.report),
    );
    Ok(ScenarioObservations {
        baseline,
        transformed,
        transformation: cases.transformation,
        metamorphic,
        pending_transformed: None,
        replay_pair,
    })
}
pub(super) fn run_with_replay(
    plan: ScenarioGenerationPlan,
    limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<Outcome, DifferentialError> {
    let mut program = generate(plan)?;
    let mut observations = replay_program(&program, plan, &mut replay)?;
    if observations.is_green() {
        return Ok(Outcome::Verified { observations });
    }
    let Some(witness) = Witness::from_observations(&observations) else {
        return Ok(Outcome::Rejected { observations });
    };
    let mut attempted_replays = 0u16;
    let mut accepted_reductions = 0u16;
    let candidate_cost = if plan.transformation().is_some() {
        2
    } else {
        1
    };
    let stop = 'reductions: loop {
        let mut accepted = false;
        for candidate in program.reductions() {
            if limit.get() - attempted_replays < candidate_cost {
                break 'reductions ArithmeticReductionStop::ReplayLimitReached;
            }
            // The replay callback checks cancellation and journals each actual
            // source before invoking the existing selected SDK worker pair.
            let candidate_observations = replay_program(&candidate, plan, &mut |case| {
                attempted_replays += 1;
                replay(case)
            })?;
            if candidate_observations.baseline.report.verdict()
                == DifferentialVerdict::WorkerFailure
                || candidate_observations
                    .transformed
                    .as_ref()
                    .is_some_and(|observation| {
                        observation.report.verdict() == DifferentialVerdict::WorkerFailure
                    })
            {
                return Ok(Outcome::Rejected {
                    observations: candidate_observations,
                });
            }
            if Witness::from_observations(&candidate_observations) == Some(witness) {
                program = candidate;
                observations = candidate_observations;
                accepted_reductions += 1;
                accepted = true;
                break;
            }
        }
        if !accepted {
            break ArithmeticReductionStop::FixedPoint;
        }
    };
    Ok(Outcome::ReducedMismatch {
        observations,
        reduction: ScenarioReductionSummary {
            attempted_replays,
            accepted_reductions,
            stop,
        },
    })
}

mod pair;
mod render;
pub use pair::{ScenarioReplayPair, SCENARIO_PAIR_SCHEMA_VERSION};
pub fn replay_scenario_pair(
    pair: &ScenarioReplayPair,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
) -> Result<ScenarioObservations, DifferentialError> {
    pair.replay_with(&mut |case| replay_case(case, oracle, runner))
}
#[cfg(test)]
mod tests;
