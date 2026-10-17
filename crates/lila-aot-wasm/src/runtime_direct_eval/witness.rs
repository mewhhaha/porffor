use super::*;
use lila_ir::{PreparedScript, PreparedScriptKind, PreparedScriptOutcome, PreparedScriptUnit};

fn root<'a>(program: &'a RuntimeDirectEvalProgram) -> Result<&'a PreparedScript, EmitError> {
    let first = program
        .program()
        .script
        .as_ref()
        .and_then(|script| script.prepared_scripts.first())
        .ok_or_else(|| EmitError::unsupported("native direct eval has no root source row"))?;
    let unit = root_unit(program, first)?;
    if unit.id != program.script_id() {
        return Err(EmitError::unsupported(
            "native direct eval root differs from its witnessed Script ID",
        ));
    }
    Ok(first)
}

fn root_unit<'a>(
    program: &RuntimeDirectEvalProgram,
    first: &'a PreparedScript,
) -> Result<&'a PreparedScriptUnit, EmitError> {
    let expected_kind = PreparedScriptKind::DirectEval(program.caller().context().clone());
    if first.kind != expected_kind {
        return Err(EmitError::unsupported(
            "native direct eval root differs from its complete caller role",
        ));
    }
    let PreparedScriptOutcome::Executable(unit) = &first.outcome else {
        return Err(EmitError::unsupported(
            "native direct eval root must be an actual executable entry",
        ));
    };
    if unit.kind != expected_kind || unit.id != program.script_id() {
        return Err(EmitError::unsupported(
            "native direct eval unit differs from its caller/root witness",
        ));
    }
    Ok(unit)
}

pub(super) fn root_source(program: &RuntimeDirectEvalProgram) -> Result<Vec<u16>, EmitError> {
    Ok(lila_front::decode_js_string_utf16(&root(program)?.source))
}

pub(super) fn require_root_source_row<'a>(
    program: &RuntimeDirectEvalProgram,
    actual: &'a PreparedScript,
) -> Result<&'a PreparedScriptUnit, EmitError> {
    if actual != root(program)? {
        return Err(EmitError::unsupported(
            "native direct eval factory differs from its exact source/caller row",
        ));
    }
    root_unit(program, actual)
}

pub(crate) fn require_factory_plan(
    program: &RuntimeDirectEvalProgram,
    functions: &crate::planning::FunctionMetaRegistry,
) -> Result<(), EmitError> {
    let actual = functions
        .prepared_scripts()
        .first()
        .ok_or_else(|| EmitError::unsupported("native direct eval has no planned factory root"))?;
    let unit = require_root_source_row(program, actual)?;
    functions
        .get(&unit.id.function_id())
        .ok_or_else(|| EmitError::unsupported("native direct eval root has no planned entry"))?
        .entry
        .prepared_script()?;
    Ok(())
}
