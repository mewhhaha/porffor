//! Canonical compiler-fact transport; no executable source or runtime handles.
use super::caller::*;
use crate::{PrivateNameId, Strictness};
use scriptw_front::EvalInvocationContext;
use std::collections::BTreeMap;

const MAGIC: &[u8; 8] = b"SCRWDE01";

impl RuntimeDirectEvalCallerDescriptor {
    pub fn encode(&self) -> Vec<u8> {
        let mut output = MAGIC.to_vec();
        output.push(match self.strictness {
            Strictness::Sloppy => 0,
            Strictness::Strict => 1,
        });
        output.push(match self.invocation {
            EvalInvocationContext::Script => 0,
            EvalInvocationContext::Function => 1,
            EvalInvocationContext::Method => 2,
            EvalInvocationContext::DerivedConstructor => 3,
            EvalInvocationContext::ClassFieldInitializer => 4,
        });
        output.push(match self.derived_owner {
            RuntimeDirectEvalDerivedOwner::Absent => 0,
            RuntimeDirectEvalDerivedOwner::ExternalCaller => 1,
        });
        match self.import_origin {
            RuntimeDirectEvalImportOrigin::Absent => output.push(0),
            RuntimeDirectEvalImportOrigin::CallerScriptDiscovery => output.push(1),
            RuntimeDirectEvalImportOrigin::CallerScriptRecord(record) => {
                output.push(2);
                output.extend_from_slice(&record.to_le_bytes());
            }
        }
        output.extend_from_slice(&(self.private_names.len() as u64).to_le_bytes());
        for (name, id) in &self.private_names {
            output.extend_from_slice(&(name.len() as u64).to_le_bytes());
            output.extend_from_slice(name.as_bytes());
            output.extend_from_slice(&id.class_scope().to_le_bytes());
            output.extend_from_slice(&id.name_ordinal().to_le_bytes());
        }
        output
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RuntimeDirectEvalCallerError> {
        let mut input = Input(bytes);
        if input.take(MAGIC.len())? != MAGIC {
            return Err(RuntimeDirectEvalCallerError(
                "unknown direct-eval descriptor version",
            ));
        }
        let strictness = match input.byte()? {
            0 => Strictness::Sloppy,
            1 => Strictness::Strict,
            _ => return Err(RuntimeDirectEvalCallerError("invalid caller strictness")),
        };
        let invocation = match input.byte()? {
            0 => EvalInvocationContext::Script,
            1 => EvalInvocationContext::Function,
            2 => EvalInvocationContext::Method,
            3 => EvalInvocationContext::DerivedConstructor,
            4 => EvalInvocationContext::ClassFieldInitializer,
            _ => {
                return Err(RuntimeDirectEvalCallerError(
                    "invalid caller invocation grammar",
                ))
            }
        };
        let derived_owner = match input.byte()? {
            0 => RuntimeDirectEvalDerivedOwner::Absent,
            1 => RuntimeDirectEvalDerivedOwner::ExternalCaller,
            _ => return Err(RuntimeDirectEvalCallerError("invalid derived caller owner")),
        };
        let import_origin = match input.byte()? {
            0 => RuntimeDirectEvalImportOrigin::Absent,
            1 => RuntimeDirectEvalImportOrigin::CallerScriptDiscovery,
            2 => RuntimeDirectEvalImportOrigin::CallerScriptRecord(input.u32()?),
            _ => return Err(RuntimeDirectEvalCallerError("invalid caller import origin")),
        };
        let count = input.length()?;
        // Even an empty spelling needs its length word and two identity words.
        // Never reserve or iterate an attacker-provided count beyond the bytes
        // physically present in this immutable descriptor.
        if count > input.0.len() / 16 {
            return Err(RuntimeDirectEvalCallerError(
                "binding count exceeds descriptor bytes",
            ));
        }
        let mut private_names = BTreeMap::new();
        for _ in 0..count {
            let length = input.length()?;
            let name = std::str::from_utf8(input.take(length)?)
                .map_err(|_| RuntimeDirectEvalCallerError("caller private spelling is not UTF-8"))?
                .to_owned();
            let id = PrivateNameId::new(input.u32()?, input.u32()?);
            if private_names.insert(name, id).is_some() {
                return Err(RuntimeDirectEvalCallerError(
                    "duplicate caller private spelling",
                ));
            }
        }
        let descriptor = Self {
            strictness,
            invocation,
            private_names,
            derived_owner,
            import_origin,
        };
        if !input.0.is_empty() || descriptor.encode() != bytes {
            return Err(RuntimeDirectEvalCallerError(
                "noncanonical direct-eval descriptor",
            ));
        }
        Ok(descriptor)
    }
}

struct Input<'a>(&'a [u8]);
impl<'a> Input<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], RuntimeDirectEvalCallerError> {
        let (head, tail) = self
            .0
            .split_at_checked(count)
            .ok_or(RuntimeDirectEvalCallerError(
                "truncated direct-eval descriptor",
            ))?;
        self.0 = tail;
        Ok(head)
    }
    fn byte(&mut self) -> Result<u8, RuntimeDirectEvalCallerError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, RuntimeDirectEvalCallerError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact four-byte slice"),
        ))
    }
    fn length(&mut self) -> Result<usize, RuntimeDirectEvalCallerError> {
        let length = u64::from_le_bytes(self.take(8)?.try_into().expect("exact eight-byte slice"));
        usize::try_from(length)
            .map_err(|_| RuntimeDirectEvalCallerError("descriptor length overflows host index"))
    }
}
