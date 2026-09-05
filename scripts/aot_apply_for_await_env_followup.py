from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(text.replace(old, new, 1))


replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''        if body_suspends
            && !matches!(
                storage_without_environment,
                Some(BindingStorage::EnvSlot { .. })
            )
        {
            return Err(EmitError::unsupported(format!(
                "for-await-of binding `{name}` does not survive a suspension in the loop body"
            )));
        }
''',
    '''        if body_suspends
            && !iteration_environment_owns_binding(lexical_environment, name)
            && !matches!(
                storage_without_environment,
                Some(BindingStorage::EnvSlot { .. })
            )
        {
            return Err(EmitError::unsupported(format!(
                "for-await-of binding `{name}` does not survive a suspension in the loop body"
            )));
        }
''',
)

replace_once(
    "crates/lila-aot-wasm/src/emit.rs",
    '''            body,
            lexical_environment,
            ..
        } => {
''',
    '''            body,
            lexical_environment: _,
            ..
        } => {
''',
)
