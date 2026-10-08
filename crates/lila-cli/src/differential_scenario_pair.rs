//! One command replays the actual relation, not two unrelated corpus entries.
use lila_test262::differential::{replay_scenario_pair, ScenarioReplayPair};

pub(super) fn run(args: &[String]) -> Result<(), String> {
    // Keep the original explicit oracle, duplicate-option and worker admission.
    let arguments = super::parse_differential_replay_args(args)?;
    let pair = ScenarioReplayPair::load(&arguments.case_path).map_err(|error| error.to_string())?;
    let runner = super::differential_worker_runner(arguments.worker_bin)?;
    let observations = replay_scenario_pair(&pair, arguments.oracle, &runner).map_err(|error| error.to_string())?;
    println!("{}", serde_json::to_string_pretty(&observations).map_err(|error| error.to_string())?);
    if observations.is_green() { Ok(()) }
    else { Err("scenario pair failed its actual differential/progress/metamorphic observations; see the JSON reports above".into()) }
}
