//! Bounded developer inputs for existing, self-contained IR admission domains.
//!
//! This is not a ProgramIr deserializer. Source, function, environment and
//! continuation identities remain owned by the real lowerer. The Engine caller
//! prepares an empty Script, admits this body, then uses its original emitter.
use crate::{
    AsyncResumeModeIr, BlockIr, DeleteOptionalPropertyChainIr, ExprIr, GeneratorResumeModeIr,
    InvalidDeleteOptionalPropertyChainIr, ObjectPropertyDefinitionError,
    ObjectPropertyDefinitionIr, ObjectPropertyIr, OptionalChainCallReceiverIr,
    OptionalChainOperationIr, ProgramIr, PropertyKeyIr, StatementIr, Strictness,
    SynchronousLoopBodyError, SynchronousLoopBodyIr, TypedExpr, ValueInfo, ValueKind, YieldForm,
};
use lila_front::ParseGoal;
use serde_json::Value;

mod input;
#[cfg(test)]
mod tests;

/// Syntax and resource bounds have been checked; IR admission is still pending.
/// The native tree is private so callers cannot bypass those bounds.
#[derive(Debug)]
pub struct IrRobustnessInput {
    body: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrRobustnessError {
    ByteLimit,
    NodeLimit,
    DepthLimit,
    StringLimit,
    Json(String),
    UnsupportedSchema,
    Wire(&'static str),
    NonEmptyScript,
    ObjectPropertyDefinition(ObjectPropertyDefinitionError),
    DeleteOptionalPropertyChain(InvalidDeleteOptionalPropertyChainIr),
    SynchronousBody(SynchronousLoopBodyError),
}

impl std::fmt::Display for IrRobustnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByteLimit => f.write_str("IR input exceeds 16384 bytes"),
            Self::NodeLimit => f.write_str("IR input exceeds 256 JSON nodes"),
            Self::DepthLimit => f.write_str("IR input exceeds 16 nested JSON levels"),
            Self::StringLimit => f.write_str("IR input string exceeds 512 UTF-8 bytes"),
            Self::Json(message) => write!(f, "invalid IR input JSON: {message}"),
            Self::UnsupportedSchema => f.write_str("unsupported IR admission schema"),
            Self::Wire(message) => f.write_str(message),
            Self::NonEmptyScript => f.write_str("IR admission requires an empty lowered Script"),
            Self::ObjectPropertyDefinition(reason) => {
                write!(f, "object property definition rejected: {reason:?}")
            }
            Self::DeleteOptionalPropertyChain(reason) => {
                write!(f, "optional property delete rejected: {reason:?}")
            }
            Self::SynchronousBody(reason) => {
                write!(f, "synchronous IR body rejected: {reason:?}")
            }
        }
    }
}
impl std::error::Error for IrRobustnessError {}

impl IrRobustnessInput {
    /// Consumes only a body without external identities. All checked operations
    /// and the synchronous-body proof are constructed before changing the Script;
    /// any rejection leaves the caller's original ProgramIr untouched.
    pub fn admit_into_empty_script(self, program: &mut ProgramIr) -> Result<(), IrRobustnessError> {
        if program.goal != ParseGoal::Script
            || program.source_len != 0
            || program.modules.is_some()
            || !program.diagnostics.is_empty()
        {
            return Err(IrRobustnessError::NonEmptyScript);
        }
        let script = program
            .script
            .as_mut()
            .ok_or(IrRobustnessError::NonEmptyScript)?;
        if !script.body.statements.is_empty()
            || script.body.lexical_environment.is_some()
            || !script.functions.is_empty()
            || !script.owned_env_bindings.is_empty()
            || !script.prepared_scripts.is_empty()
            || !script.prepared_dynamic_functions.is_empty()
            || script.module_prelude.is_some()
        {
            return Err(IrRobustnessError::NonEmptyScript);
        }
        let candidate = StatementIr::Block(block(&self.body)?);
        SynchronousLoopBodyIr::new(&candidate).map_err(IrRobustnessError::SynchronousBody)?;
        let StatementIr::Block(body) = candidate else {
            unreachable!("the admission candidate is the complete body above")
        };
        script.body = body;
        Ok(())
    }
}

fn block(values: &[Value]) -> Result<BlockIr, IrRobustnessError> {
    let statements = values
        .iter()
        .map(statement)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BlockIr {
        // Conservative completion information needs no invented flow analysis.
        // Exact operand kinds below still satisfy the real operation constructors.
        result_kind: if statements.is_empty() {
            ValueKind::Undefined
        } else {
            ValueKind::Dynamic
        },
        statements,
        lexical_environment: None,
    })
}

fn statement(value: &Value) -> Result<StatementIr, IrRobustnessError> {
    use input::{array, boolean, fields, string};
    let operation = value
        .get("op")
        .and_then(Value::as_str)
        .ok_or(IrRobustnessError::Wire("IR statement requires an op"))?;
    Ok(match operation {
        "empty" => {
            fields(value, &["op"])?;
            StatementIr::Empty
        }
        "value" => {
            let fields = fields(value, &["op", "value"])?;
            StatementIr::Expression(literal(&fields["value"])?)
        }
        "define_property" => {
            let fields = fields(value, &["op", "target", "key", "value"])?;
            let definition = ObjectPropertyDefinitionIr::new(
                literal(&fields["target"])?,
                ObjectPropertyIr::Data {
                    key: string(&fields["key"])?.to_owned(),
                    value: literal(&fields["value"])?,
                    is_shorthand: false,
                },
            )
            .map_err(IrRobustnessError::ObjectPropertyDefinition)?;
            StatementIr::Expression(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::ObjectPropertyDefinition(Box::new(definition)),
            ))
        }
        "delete_optional" => {
            let fields = fields(value, &["op", "target", "chain", "strict"])?;
            let chain = array(&fields["chain"])?
                .iter()
                .map(chain_operation)
                .collect::<Result<Vec<_>, _>>()?;
            let strictness = if boolean(&fields["strict"])? {
                Strictness::Strict
            } else {
                Strictness::Sloppy
            };
            let deletion =
                DeleteOptionalPropertyChainIr::new(literal(&fields["target"])?, chain, strictness)
                    .map_err(IrRobustnessError::DeleteOptionalPropertyChain)?;
            StatementIr::Expression(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Boolean),
                ExprIr::DeleteOptionalPropertyChain(Box::new(deletion)),
            ))
        }
        "block" => {
            let fields = fields(value, &["op", "body"])?;
            StatementIr::Block(block(array(&fields["body"])?)?)
        }
        "if" => {
            let fields = fields(value, &["op", "condition", "then", "else"])?;
            StatementIr::If {
                condition: literal(&fields["condition"])?,
                then_branch: Box::new(StatementIr::Block(block(array(&fields["then"])?)?)),
                else_branch: Some(Box::new(StatementIr::Block(block(array(
                    &fields["else"],
                )?)?))),
            }
        }
        "await" => {
            let fields = fields(value, &["op", "value"])?;
            // A rejection probe, never a serialized continuation certificate.
            // The complete body's real synchronous constructor rejects it.
            StatementIr::AsyncAwait {
                value: literal(&fields["value"])?,
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::Ignore,
            }
        }
        "yield" => {
            let fields = fields(value, &["op", "value"])?;
            StatementIr::GeneratorYield {
                value: literal(&fields["value"])?,
                form: YieldForm::Plain,
                suspend_state: 0,
                resume_state: 1,
                resume_mode: GeneratorResumeModeIr::Ignore,
            }
        }
        _ => {
            return Err(IrRobustnessError::Wire(
                "unsupported IR statement operation",
            ))
        }
    })
}

fn literal(value: &Value) -> Result<TypedExpr, IrRobustnessError> {
    use input::{boolean, fields, string};
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or(IrRobustnessError::Wire("IR literal requires a kind"))?;
    let (kind, expression) = match kind {
        "undefined" => {
            fields(value, &["kind"])?;
            return Ok(TypedExpr::undefined());
        }
        "null" => {
            fields(value, &["kind"])?;
            (ValueKind::Null, ExprIr::Null)
        }
        "object" => {
            fields(value, &["kind"])?;
            (ValueKind::Object, ExprIr::ObjectLiteral(Vec::new()))
        }
        "boolean" => {
            let fields = fields(value, &["kind", "value"])?;
            (
                ValueKind::Boolean,
                ExprIr::Boolean(boolean(&fields["value"])?),
            )
        }
        "number" => {
            let fields = fields(value, &["kind", "bits"])?;
            let bits = string(&fields["bits"])?;
            if bits.len() != 16
                || !bits
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(IrRobustnessError::Wire(
                    "IR number bits require 16 lowercase hex digits",
                ));
            }
            let bits = u64::from_str_radix(bits, 16)
                .map_err(|_| IrRobustnessError::Wire("invalid IR number bits"))?;
            (ValueKind::Number, ExprIr::Number(bits))
        }
        "string" => {
            let fields = fields(value, &["kind", "value"])?;
            (
                ValueKind::String,
                ExprIr::String(string(&fields["value"])?.to_owned()),
            )
        }
        _ => return Err(IrRobustnessError::Wire("unsupported IR literal kind")),
    };
    Ok(TypedExpr::from_info(ValueInfo::new(kind), expression))
}

fn chain_operation(value: &Value) -> Result<OptionalChainOperationIr, IrRobustnessError> {
    use input::{array, boolean, fields, string};
    match value.get("op").and_then(Value::as_str) {
        Some("property") => {
            let fields = fields(value, &["op", "key", "shorted"])?;
            Ok(OptionalChainOperationIr::Property {
                key: PropertyKeyIr::StaticString(string(&fields["key"])?.to_owned()),
                shorted: boolean(&fields["shorted"])?,
            })
        }
        Some("call") => {
            let fields = fields(value, &["op", "args", "shorted", "boundary_before"])?;
            Ok(OptionalChainOperationIr::Call {
                args: array(&fields["args"])?
                    .iter()
                    .map(literal)
                    .collect::<Result<_, _>>()?,
                receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                shorted: boolean(&fields["shorted"])?,
                boundary_before: boolean(&fields["boundary_before"])?,
            })
        }
        _ => Err(IrRobustnessError::Wire(
            "unsupported optional-chain operation",
        )),
    }
}
