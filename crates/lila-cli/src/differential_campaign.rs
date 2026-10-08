//! CLI campaign admission shares the actual selected worker and oracle gates.
use lila_test262::differential::{
    ArithmeticCheckCount, ArithmeticExpressionDepth,
    ArithmeticGenerationPlan, ArithmeticGenerationSeed, ArithmeticGrammar,
    ArithmeticReductionLimit, GeneratedCampaignPlan, ObjectGenerationPlan, ObjectGrammar,
    ModuleGenerationPlan, ModuleGrammar, ControlFlowGenerationPlan, CONTROL_FLOW_GRAMMAR,
    NegativeGenerationPlan, NEGATIVE_SOURCE_GRAMMAR, ScenarioGenerationPlan, ScenarioGrammar,
    SpecExecOracle,
};
use std::path::PathBuf;
#[cfg(feature = "spec-exec-oracle")]
use lila_test262::differential::{run_generated_campaign, GeneratedCampaignCancellation};

struct Arguments {
    output: PathBuf,
    plan: GeneratedCampaignPlan,
    worker_bin: Option<PathBuf>,
    oracle: SpecExecOracle,
}
impl Arguments {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut output = None;
        let mut seed = None;
        let mut cases = None;
        let mut checks = None;
        let mut depth = None;
        let mut nodes = None;
        let mut properties = None;
        let mut steps = None;
        let mut modules = None;
        let mut edges = None;
        let mut max_replays = None;
        let mut grammar = None;
        let mut worker_bin = None;
        let mut oracle = None;
        let mut index = 0;
        while index < args.len() {
            let flag = &args[index];
            let value = args.get(index + 1)
                .filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| format!("{flag} needs a value"))?;
            match flag.as_str() {
                "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
                "--seed" if seed.is_none() => {
                    seed = Some(ArithmeticGenerationSeed::new(value.parse::<u64>()
                        .map_err(|_| "--seed needs a u64 integer".to_owned())?));
                }
                "--cases" if cases.is_none() => cases = Some(value.parse::<usize>()
                    .map_err(|_| "--cases needs a positive integer".to_owned())?),
                "--checks" if checks.is_none() => checks = Some(ArithmeticCheckCount::new(
                    value.parse::<usize>().map_err(|_| "--checks needs an integer".to_owned())?)
                    .map_err(|error| error.to_string())?),
                "--depth" if depth.is_none() => depth = Some(ArithmeticExpressionDepth::new(
                    value.parse::<usize>().map_err(|_| "--depth needs an integer".to_owned())?)
                    .map_err(|error| error.to_string())?),
                "--max-replays" if max_replays.is_none() => max_replays = Some(ArithmeticReductionLimit::new(
                    value.parse::<usize>().map_err(|_| "--max-replays needs an integer".to_owned())?)
                    .map_err(|error| error.to_string())?),
                "--nodes" if nodes.is_none() => nodes = Some(value.parse::<usize>()
                    .map_err(|_| "--nodes needs an integer".to_owned())?),
                "--properties" if properties.is_none() => properties = Some(value.parse::<usize>()
                    .map_err(|_| "--properties needs an integer".to_owned())?),
                "--steps" if steps.is_none() => steps = Some(value.parse::<usize>()
                    .map_err(|_| "--steps needs an integer".to_owned())?),
                "--modules" if modules.is_none() => modules = Some(value.parse::<usize>()
                    .map_err(|_| "--modules needs an integer".to_owned())?),
                "--edges" if edges.is_none() => edges = Some(value.parse::<usize>()
                    .map_err(|_| "--edges needs an integer".to_owned())?),
                "--grammar" if grammar.is_none() => grammar = Some(value.as_str()),
                "--worker-bin" if worker_bin.is_none() => worker_bin = Some(PathBuf::from(value)),
                "--oracle" if oracle.is_none() => oracle = Some(super::parse_differential_oracle(value)?),
                "--output-dir" | "--seed" | "--cases" | "--checks" | "--depth"
                | "--max-replays" | "--grammar" | "--worker-bin" | "--oracle"
                | "--nodes" | "--properties" | "--steps" | "--modules" | "--edges" => {
                    return Err(format!("{flag} may only be specified once"));
                }
                _ => return Err(format!("unknown differential campaign option: {flag}")),
            }
            index += 2;
        }
        let seed = seed.ok_or_else(|| "differential campaign needs --seed N".to_owned())?;
        let cases = cases.ok_or_else(|| "differential campaign needs --cases N".to_owned())?;
        let limit = max_replays.ok_or_else(|| "differential campaign needs --max-replays N".to_owned())?;
        let grammar = grammar.unwrap_or("integer-product-v3");
        let plan = if grammar == NEGATIVE_SOURCE_GRAMMAR {
            if checks.is_some() || nodes.is_some() || properties.is_some() || modules.is_some() || edges.is_some() {
                return Err("negative-source-v1 accepts only --steps and --depth generation budgets".into());
            }
            GeneratedCampaignPlan::for_negative_source(NegativeGenerationPlan::new(seed.get(),
                steps.ok_or_else(|| "negative-source-v1 needs --steps N".to_owned())?,
                usize::from(depth.ok_or_else(|| "negative-source-v1 needs --depth N".to_owned())?.get()))
                .map_err(|error| error.to_string())?, cases, limit)
        } else if let Some(scenario_grammar) = ScenarioGrammar::from_name(grammar) {
            if checks.is_some() || depth.is_some() || nodes.is_some() || properties.is_some() || modules.is_some() || edges.is_some() {
                return Err(format!("{grammar} accepts only the --steps generation budget"));
            }
            GeneratedCampaignPlan::for_scenarios(ScenarioGenerationPlan::new(scenario_grammar, seed.get(),
                steps.ok_or_else(|| format!("{grammar} needs --steps N"))?)
                .map_err(|error| error.to_string())?, cases, limit)
        } else if grammar == CONTROL_FLOW_GRAMMAR {
            if checks.is_some() || nodes.is_some() || properties.is_some() || modules.is_some() || edges.is_some() {
                return Err("control-flow-v1 accepts only --steps and --depth generation budgets".into());
            }
            GeneratedCampaignPlan::for_control_flow(ControlFlowGenerationPlan::new(seed.get(),
                steps.ok_or_else(|| "control-flow-v1 needs --steps N".to_owned())?,
                usize::from(depth.ok_or_else(|| "control-flow-v1 needs --depth N".to_owned())?.get()))
                .map_err(|error| error.to_string())?, cases, limit)
        } else if let Some(object_grammar) = ObjectGrammar::from_name(grammar) {
            if checks.is_some() || depth.is_some() || modules.is_some() || edges.is_some() {
                return Err(format!("{grammar} accepts only object generation budgets"));
            }
            let steps = match object_grammar {
                ObjectGrammar::StaticV1 if steps.is_some() => return Err("object-probe-v1 has no --steps budget".into()),
                ObjectGrammar::StaticV1 => 0,
                ObjectGrammar::MutationsV2 => steps.ok_or_else(|| "object-mutations-v2 needs --steps N".to_owned())?,
            };
            GeneratedCampaignPlan::for_objects(ObjectGenerationPlan::for_grammar(object_grammar, seed.get(),
                nodes.ok_or_else(|| format!("{grammar} needs --nodes N"))?,
                properties.ok_or_else(|| format!("{grammar} needs --properties N"))?, steps)
                .map_err(|error| error.to_string())?, cases, limit)
        } else if let Some(module_grammar) = ModuleGrammar::from_name(grammar) {
            if checks.is_some() || depth.is_some() || nodes.is_some() || properties.is_some() || steps.is_some() {
                return Err(format!("{grammar} accepts only --modules and --edges generation budgets"));
            }
            GeneratedCampaignPlan::for_modules(ModuleGenerationPlan::for_grammar(module_grammar, seed.get(),
                modules.ok_or_else(|| format!("{grammar} needs --modules N"))?,
                edges.ok_or_else(|| format!("{grammar} needs --edges N"))?)
                .map_err(|error| error.to_string())?, cases, limit)
        } else {
            if nodes.is_some() || properties.is_some() || modules.is_some() || edges.is_some() || steps.is_some() {
                return Err("arithmetic grammars accept only --checks and --depth generation budgets".into());
            }
            GeneratedCampaignPlan::new(ArithmeticGenerationPlan::new(
                ArithmeticGrammar::from_name(grammar).map_err(|error| error.to_string())?, seed,
                checks.ok_or_else(|| "differential campaign needs --checks N".to_owned())?,
                depth.ok_or_else(|| "differential campaign needs --depth N".to_owned())?), cases, limit)
        }.map_err(|error| error.to_string())?;
        Ok(Self {
            plan,
            output: output.ok_or_else(|| "differential campaign needs --output-dir PATH".to_owned())?,
            worker_bin,
            oracle: oracle.ok_or_else(|| "differential campaign requires explicit --oracle spec-exec".to_owned())?,
        })
    }
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let arguments = Arguments::parse(args)?;
    // Feature-off refusal precedes creating campaign output.
    #[cfg(not(feature = "spec-exec-oracle"))]
    { let _ = arguments; return Err(lila_test262::differential::DifferentialError::OracleNotLinked.to_string()); }
    #[cfg(feature = "spec-exec-oracle")]
    {
        let runner = super::differential_worker_runner(arguments.worker_bin)?;
        let report = run_generated_campaign(arguments.plan, arguments.oracle, &runner,
            &GeneratedCampaignCancellation::default(), &arguments.output)
            .map_err(|error| error.to_string())?;
        println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?);
        if report.is_green() { Ok(()) } else {
            Err(format!("generated campaign is {:?}: {} failures, {} of {} cases completed in {}",
                report.verdict(), report.failed(), report.completed(), report.total(), arguments.output.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(extra: &[&str]) -> Result<Arguments, String> {
        let mut arguments = vec!["--output-dir", "fresh", "--seed", "9", "--cases", "2",
            "--checks", "4", "--depth", "2", "--max-replays", "16"];
        arguments.extend_from_slice(extra);
        Arguments::parse(&arguments.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>())
    }
    #[test]
    fn campaign_requires_explicit_oracle_and_one_bounded_plan() {
        assert!(parse(&[]).is_err());
        assert!(parse(&["--oracle", "wasm-aot"]).is_err());
        assert_eq!(parse(&["--oracle", "spec-exec"]).unwrap().plan.count(), 2);
        for pair in [["--cases", "0"], ["--seed", "8"], ["--checks", "1"]] {
            assert!(parse(&pair).is_err());
        }
        assert!(parse(&["--oracle", "spec-exec", "--grammar", "arbitrary-source"]).is_err());
    }
    #[test]
    fn object_campaign_consumes_only_the_checked_object_budgets() {
        let parse = |extra: &[&str]| {
            let mut args = vec!["--output-dir", "fresh", "--seed", "9", "--cases", "2",
                "--max-replays", "16", "--oracle", "spec-exec", "--grammar", "object-probe-v1",
                "--nodes", "4", "--properties", "6"];
            args.extend_from_slice(extra);
            Arguments::parse(&args.into_iter().map(str::to_owned).collect::<Vec<_>>())
        };
        assert_eq!(parse(&[]).unwrap().plan.count(), 2);
        assert!(parse(&["--checks", "1"]).is_err());
        assert!(parse(&["--depth", "1"]).is_err());
        assert!(parse(&["--nodes", "2"]).is_err());
        assert!(parse(&["--properties", "17"]).is_err());
        assert!(parse(&["--steps", "0"]).is_err());
    }
    #[test]
    fn mutation_campaign_requires_its_real_schedule_and_reserves_an_observation_root() {
        let base = ["--output-dir", "fresh", "--seed", "9", "--cases", "2",
            "--max-replays", "16", "--oracle", "spec-exec", "--grammar", "object-mutations-v2"];
        let parse = |budgets: &[&str]| Arguments::parse(&base.iter().chain(budgets)
            .map(|value| (*value).to_owned()).collect::<Vec<_>>());
        assert_eq!(parse(&["--nodes", "4", "--properties", "6", "--steps", "16"]).unwrap().plan.count(), 2);
        assert!(parse(&["--nodes", "4", "--properties", "6"]).is_err());
        assert!(parse(&["--nodes", "16", "--properties", "6", "--steps", "16"]).is_err());
        assert!(parse(&["--nodes", "4", "--properties", "6", "--steps", "65"]).is_err());
        assert!(parse(&["--nodes", "4", "--properties", "6", "--steps", "16", "--edges", "1"]).is_err());
    }
    #[test]
    fn module_campaign_consumes_a_complete_graph_budget() {
        let base = ["--output-dir", "fresh", "--seed", "9", "--cases", "2",
            "--max-replays", "16", "--oracle", "spec-exec", "--grammar", "module-graph-v1"];
        let parse = |budgets: &[&str]| Arguments::parse(&base.iter().chain(budgets)
            .map(|value| (*value).to_owned()).collect::<Vec<_>>());
        assert_eq!(parse(&["--modules", "4", "--edges", "6"]).unwrap().plan.count(), 2);
        assert!(parse(&["--modules", "4", "--edges", "2"]).is_err());
        assert!(parse(&["--modules", "4", "--edges", "17"]).is_err());
        assert!(parse(&["--modules", "1", "--edges", "0", "--depth", "1"]).is_err());
    }
    #[test]
    fn async_module_campaign_uses_acyclic_edge_capacity() {
        let base = ["--output-dir", "fresh", "--seed", "9", "--cases", "2",
            "--max-replays", "16", "--oracle", "spec-exec", "--grammar", "module-graph-v2"];
        let parse = |budgets: &[&str]| Arguments::parse(&base.iter().chain(budgets)
            .map(|value| (*value).to_owned()).collect::<Vec<_>>());
        assert_eq!(parse(&["--modules", "4", "--edges", "6"]).unwrap().plan.count(), 2);
        assert!(parse(&["--modules", "4", "--edges", "7"]).is_err());
        assert!(parse(&["--modules", "4", "--edges", "2"]).is_err());
        assert!(parse(&["--modules", "2", "--edges", "1", "--checks", "1"]).is_err());
    }
    #[test]
    fn control_flow_campaign_consumes_only_the_checked_statement_tree_budgets() {
        let base = ["--output-dir", "fresh", "--seed", "9", "--cases", "4",
            "--max-replays", "16", "--oracle", "spec-exec", "--grammar", "control-flow-v1"];
        let parse = |budgets: &[&str]| Arguments::parse(&base.iter().chain(budgets)
            .map(|value| (*value).to_owned()).collect::<Vec<_>>());
        assert_eq!(parse(&["--steps", "8", "--depth", "4"]).unwrap().plan.count(), 4);
        assert!(parse(&["--steps", "0", "--depth", "4"]).is_err());
        assert!(parse(&["--steps", "33", "--depth", "4"]).is_err());
        assert!(parse(&["--steps", "8", "--depth", "5"]).is_err());
        assert!(parse(&["--steps", "8"]).is_err());
        assert!(parse(&["--steps", "8", "--depth", "4", "--checks", "1"]).is_err());
        assert!(parse(&["--steps", "8", "--depth", "4", "--edges", "1"]).is_err());
    }
    #[test]
    fn negative_and_stateful_campaigns_admit_only_their_actual_source_budgets() {
        let base = ["--output-dir", "fresh", "--seed", "9", "--cases", "2",
            "--max-replays", "16", "--oracle", "spec-exec"];
        let parse = |grammar: &str, budgets: &[&str]| {
            let mut args: Vec<String> = base.iter().map(|value| (*value).into()).collect();
            args.extend(["--grammar".into(), grammar.into()]);
            args.extend(budgets.iter().map(|value| (*value).into()));
            Arguments::parse(&args)
        };
        assert_eq!(parse("negative-source-v1", &["--steps", "4", "--depth", "3"]).unwrap().plan.count(), 2);
        assert!(parse("negative-source-v1", &["--steps", "4"]).is_err());
        assert!(parse("negative-source-v1", &["--steps", "4", "--depth", "3", "--checks", "1"]).is_err());
        for grammar in ["builtin-stateful-v1", "metamorphic-stateful-v1", "builtin-stateful-v2", "metamorphic-stateful-v2"] {
            assert_eq!(parse(grammar, &["--steps", "8"]).unwrap().plan.count(), 2);
            for invalid in [&["--steps", "0"][..], &["--steps", "33"], &["--steps", "4", "--depth", "1"], &["--steps", "4", "--nodes", "1"]] {
                assert!(parse(grammar, invalid).is_err());
            }
        }
    }
}
