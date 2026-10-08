use super::*;
use std::path::Path;

pub const SCENARIO_PAIR_SCHEMA_VERSION: u32 = 1;

/// A reduced relation is replayable only with both exact admitted variants.
/// The closed program certificate prevents arbitrary source from claiming a
/// lawful rename/block/loop relation or invented progress checkpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioReplayPair {
    program: Program,
    plan: ScenarioGenerationPlan,
    cases: GeneratedScenarioCases,
    transformation: MetamorphicTransformation,
}

#[derive(Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanWire {
    grammar: String,
    seed: u64,
    steps: u8,
}
#[derive(Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgramWire {
    family: ScenarioFamily,
    strict: bool,
    actions: Vec<Action>,
}
#[derive(Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PairWire {
    schema_version: u32,
    plan: PlanWire,
    transformation: MetamorphicTransformation,
    expected_steps: u8,
    program: ProgramWire,
    baseline: serde_json::Value,
    transformed: serde_json::Value,
}
impl ScenarioReplayPair {
    pub(super) fn from_program(
        program: Program,
        plan: ScenarioGenerationPlan,
    ) -> Result<Self, DifferentialError> {
        let transformation = plan.transformation().ok_or_else(|| {
            DifferentialError::InvalidGeneration(
                "paired replay requires the metamorphic grammar".into(),
            )
        })?;
        let cases = program.cases(plan)?;
        Ok(Self {
            program,
            plan,
            cases,
            transformation,
        })
    }
    pub fn generate(plan: ScenarioGenerationPlan) -> Result<Self, DifferentialError> {
        Self::from_program(generate(plan)?, plan)
    }
    pub fn plan(&self) -> ScenarioGenerationPlan {
        self.plan
    }
    pub fn cases(&self) -> &GeneratedScenarioCases {
        &self.cases
    }
    pub fn transformation(&self) -> MetamorphicTransformation {
        self.transformation
    }
    pub fn expected_steps(&self) -> usize {
        self.program.actions.len()
    }
    pub fn from_json(json: &str) -> Result<Self, DifferentialError> {
        if json.len() > worker_process::MAX_REQUEST_BYTES {
            return Err(DifferentialError::InvalidCorpus(
                "scenario pair exceeds the existing worker request budget".into(),
            ));
        }
        let wire: PairWire = serde_json::from_str(json)
            .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
        if wire.schema_version != SCENARIO_PAIR_SCHEMA_VERSION {
            return Err(DifferentialError::InvalidCorpus(
                "unsupported scenario pair schema version".into(),
            ));
        }
        let grammar = ScenarioGrammar::from_name(&wire.plan.grammar).ok_or_else(|| {
            DifferentialError::InvalidCorpus("unknown scenario pair grammar".into())
        })?;
        let plan =
            ScenarioGenerationPlan::new(grammar, wire.plan.seed, usize::from(wire.plan.steps))?;
        let program = Program::new(
            wire.program.family,
            wire.program.strict,
            wire.program.actions,
        )
        .ok_or_else(|| {
            DifferentialError::InvalidCorpus(
                "scenario pair has invalid action domains or bounds".into(),
            )
        })?;
        if usize::from(wire.expected_steps) != program.actions.len() {
            return Err(DifferentialError::InvalidCorpus(
                "scenario pair progress count does not match its checked actions".into(),
            ));
        }
        let pair = Self::from_program(program, plan)?;
        if wire.transformation != pair.transformation {
            return Err(DifferentialError::InvalidCorpus(
                "scenario pair transformation differs from its checked seed".into(),
            ));
        }
        let decode = |value| {
            DifferentialReplayInput::from_json(
                &serde_json::to_string(&value)
                    .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?,
            )
        };
        let baseline = decode(wire.baseline)?;
        let transformed = decode(wire.transformed)?;
        if pair.cases.baseline() != &baseline || pair.cases.transformed() != Some(&transformed) {
            return Err(DifferentialError::InvalidCorpus(
                "scenario pair sources/protocols differ from their checked producer".into(),
            ));
        }
        Ok(pair)
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DifferentialError> {
        use std::io::Read as _;
        let path = path.as_ref();
        let file = std::fs::File::open(path).map_err(|error| DifferentialError::ReadCorpus {
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
    pub(super) fn replay_with(
        &self,
        replay: &mut impl FnMut(
            &DifferentialReplayInput,
        ) -> Result<DifferentialReport, DifferentialError>,
    ) -> Result<ScenarioObservations, DifferentialError> {
        replay_program(&self.program, self.plan, replay)
    }
}
impl Serialize for ScenarioReplayPair {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let encode = |case: &DifferentialReplayInput| -> Result<serde_json::Value, S::Error> {
            let json = case
                .to_pretty_json()
                .map_err(<S::Error as serde::ser::Error>::custom)?;
            serde_json::from_str(&json).map_err(<S::Error as serde::ser::Error>::custom)
        };
        let transformed = self.cases.transformed().ok_or_else(|| {
            <S::Error as serde::ser::Error>::custom(
                "checked scenario pair lost its transformed case",
            )
        })?;
        PairWire {
            schema_version: SCENARIO_PAIR_SCHEMA_VERSION,
            plan: PlanWire {
                grammar: self.plan.grammar().name().into(),
                seed: self.plan.seed(),
                steps: self.plan.steps(),
            },
            transformation: self.transformation,
            expected_steps: self.program.actions.len() as u8,
            program: ProgramWire {
                family: self.program.family,
                strict: self.program.strict,
                actions: self.program.actions.clone(),
            },
            baseline: encode(self.cases.baseline())?,
            transformed: encode(transformed)?,
        }
        .serialize(serializer)
    }
}
