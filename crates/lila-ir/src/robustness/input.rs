use super::{IrRobustnessError, IrRobustnessInput};
use serde_json::{Map, Value};

const MAX_BYTES: usize = 16_384;
const MAX_NODES: usize = 256;
const MAX_DEPTH: usize = 16;
const MAX_STRING_BYTES: usize = 512;

impl IrRobustnessInput {
    /// Decode the small native envelope before allocating any candidate IR.
    /// The byte cap also bounds the JSON decoder's own allocation; its default
    /// recursion limit stays enabled, followed by this stricter tree bound.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IrRobustnessError> {
        if bytes.len() > MAX_BYTES {
            return Err(IrRobustnessError::ByteLimit);
        }
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| IrRobustnessError::Json(error.to_string()))?;
        let fields = fields(&value, &["schema_version", "body"])?;
        if fields["schema_version"].as_u64() != Some(1) {
            return Err(IrRobustnessError::UnsupportedSchema);
        }
        let mut nodes = 0;
        bound(&value, 0, &mut nodes)?;
        array(&fields["body"])?;
        let Value::Object(mut envelope) = value else {
            unreachable!("fields admitted an object")
        };
        let Some(Value::Array(body)) = envelope.remove("body") else {
            unreachable!("array admitted the body")
        };
        Ok(Self { body })
    }
}

fn bound(value: &Value, depth: usize, nodes: &mut usize) -> Result<(), IrRobustnessError> {
    if depth > MAX_DEPTH {
        return Err(IrRobustnessError::DepthLimit);
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(IrRobustnessError::NodeLimit);
    }
    match value {
        Value::Array(values) => {
            if values.len() > MAX_NODES - *nodes {
                return Err(IrRobustnessError::NodeLimit);
            }
            values
                .iter()
                .try_for_each(|value| bound(value, depth + 1, nodes))
        }
        Value::Object(fields) => {
            if fields.len() > MAX_NODES - *nodes {
                return Err(IrRobustnessError::NodeLimit);
            }
            for (key, value) in fields {
                bounded_string(key)?;
                bound(value, depth + 1, nodes)?;
            }
            Ok(())
        }
        Value::String(value) => bounded_string(value),
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
    }
}

fn bounded_string(value: &str) -> Result<(), IrRobustnessError> {
    if value.len() > MAX_STRING_BYTES {
        Err(IrRobustnessError::StringLimit)
    } else {
        Ok(())
    }
}

pub(super) fn fields<'a>(
    value: &'a Value,
    keys: &[&str],
) -> Result<&'a Map<String, Value>, IrRobustnessError> {
    let fields = value
        .as_object()
        .ok_or(IrRobustnessError::Wire("IR input requires an object"))?;
    if fields.len() != keys.len() || keys.iter().any(|key| !fields.contains_key(*key)) {
        return Err(IrRobustnessError::Wire("unknown or missing IR input field"));
    }
    Ok(fields)
}

pub(super) fn array(value: &Value) -> Result<&[Value], IrRobustnessError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or(IrRobustnessError::Wire("IR input requires an array"))
}

pub(super) fn string(value: &Value) -> Result<&str, IrRobustnessError> {
    value
        .as_str()
        .ok_or(IrRobustnessError::Wire("IR input requires a string"))
}

pub(super) fn boolean(value: &Value) -> Result<bool, IrRobustnessError> {
    value
        .as_bool()
        .ok_or(IrRobustnessError::Wire("IR input requires a boolean"))
}
