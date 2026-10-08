use super::super::*;
use crate::functions::OrdinaryDefaultPrototype;
use crate::gc_types::{CompletionLocals, SymbolValue, SymbolValueSchema, ValueLocals};
use lila_ir::{
    RuntimeSemanticRejection, RuntimeUnavailableCapability, WasmWeakReachabilityCapability,
    PRODUCT_WASM_WEAK_REACHABILITY,
};

/// Native weak routes are exhaustive over the selected runtime capability.
pub(super) enum WeakBuiltin {
    WeakMapConstructor,
    WeakMapPrototypeDelete,
    WeakMapPrototypeGet,
    WeakMapPrototypeGetOrInsert,
    WeakMapPrototypeGetOrInsertComputed,
    WeakMapPrototypeHas,
    WeakMapPrototypeSet,
    WeakSetConstructor,
    WeakSetPrototypeAdd,
    WeakSetPrototypeDelete,
    WeakSetPrototypeHas,
    WeakRefConstructor,
    WeakRefPrototypeDeref,
    FinalizationRegistryConstructor,
    FinalizationRegistryPrototypeRegister,
    FinalizationRegistryPrototypeUnregister,
}

#[derive(Clone, Copy)]
enum WeakFamily {
    Map,
    Set,
    Ref,
    Registry,
}

enum WeakReceiverError {
    NonObject,
    MissingInternalSlots,
}

impl WeakFamily {
    fn constructor_requires_new_message(self) -> RuntimeErrorMessage {
        match self {
            Self::Map => RuntimeErrorMessage::WEAKMAP_CONSTRUCTOR_REQUIRES_NEW,
            Self::Set => RuntimeErrorMessage::WEAKSET_CONSTRUCTOR_REQUIRES_NEW,
            Self::Ref => RuntimeErrorMessage::WEAKREF_CONSTRUCTOR_REQUIRES_NEW,
            Self::Registry => RuntimeErrorMessage::FINALIZATIONREGISTRY_CONSTRUCTOR_REQUIRES_NEW,
        }
    }

    fn prototype(self) -> OrdinaryDefaultPrototype {
        match self {
            Self::Map => OrdinaryDefaultPrototype::WeakMap,
            Self::Set => OrdinaryDefaultPrototype::WeakSet,
            Self::Ref => OrdinaryDefaultPrototype::WeakRef,
            Self::Registry => OrdinaryDefaultPrototype::FinalizationRegistry,
        }
    }

    fn receiver_error_message(self, error: WeakReceiverError) -> RuntimeErrorMessage {
        match (self, error) {
            (Self::Map, WeakReceiverError::NonObject) => {
                RuntimeErrorMessage::WEAKMAP_METHOD_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::Map, WeakReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::WEAKMAP_METHOD_RECEIVER_DOES_NOT_HAVE_WEAKMAPDATA
            }
            (Self::Set, WeakReceiverError::NonObject) => {
                RuntimeErrorMessage::WEAKSET_METHOD_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::Set, WeakReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::WEAKSET_METHOD_RECEIVER_DOES_NOT_HAVE_WEAKSETDATA
            }
            (Self::Ref, WeakReceiverError::NonObject | WeakReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::WEAKREF_PROTOTYPE_DEREF_RECEIVER_DOES_NOT_HAVE_WEAKREFTARGET
            }
            (
                Self::Registry,
                WeakReceiverError::NonObject | WeakReceiverError::MissingInternalSlots,
            ) => RuntimeErrorMessage::FINALIZATIONREGISTRY_METHOD_RECEIVER_DOES_NOT_HAVE_CELLS,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_weak_unavailable_builtin(
        &mut self,
        builtin: WeakBuiltin,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        // No record with weak internal slots can be produced in this branch.
        // A new facility variant requires implementing its real representation.
        match PRODUCT_WASM_WEAK_REACHABILITY {
            WasmWeakReachabilityCapability::Unavailable => match builtin {
                WeakBuiltin::WeakMapConstructor => {
                    self.emit_weak_constructor_preflight(WeakFamily::Map, f)
                }
                WeakBuiltin::WeakSetConstructor => {
                    self.emit_weak_constructor_preflight(WeakFamily::Set, f)
                }
                WeakBuiltin::WeakRefConstructor => {
                    self.emit_weak_constructor_preflight(WeakFamily::Ref, f)
                }
                WeakBuiltin::FinalizationRegistryConstructor => {
                    self.emit_weak_constructor_preflight(WeakFamily::Registry, f)
                }
                WeakBuiltin::WeakMapPrototypeDelete
                | WeakBuiltin::WeakMapPrototypeGet
                | WeakBuiltin::WeakMapPrototypeGetOrInsert
                | WeakBuiltin::WeakMapPrototypeGetOrInsertComputed
                | WeakBuiltin::WeakMapPrototypeHas
                | WeakBuiltin::WeakMapPrototypeSet => {
                    self.emit_unavailable_weak_receiver_error(WeakFamily::Map, f)
                }
                WeakBuiltin::WeakSetPrototypeAdd
                | WeakBuiltin::WeakSetPrototypeDelete
                | WeakBuiltin::WeakSetPrototypeHas => {
                    self.emit_unavailable_weak_receiver_error(WeakFamily::Set, f)
                }
                WeakBuiltin::WeakRefPrototypeDeref => {
                    self.emit_unavailable_weak_receiver_error(WeakFamily::Ref, f)
                }
                WeakBuiltin::FinalizationRegistryPrototypeRegister
                | WeakBuiltin::FinalizationRegistryPrototypeUnregister => {
                    self.emit_unavailable_weak_receiver_error(WeakFamily::Registry, f)
                }
            },
        }
    }

    fn emit_weak_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_weak_constructor_preflight(
        &mut self,
        family: WeakFamily,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let new_target = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        self.compile_new_target_to_locals(&new_target, f)?;
        let exit = self.open_frame(ControlFrameKind::Block, f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.emit_weak_error_if(family.constructor_requires_new_message(), &output, exit, f)?;

        // These first-argument checks precede GetPrototypeFromConstructor.
        match family {
            WeakFamily::Map | WeakFamily::Set => {}
            WeakFamily::Ref => {
                self.emit_builtin_arg_to_value(0, &argument, f);
                self.emit_whole_can_be_held_weakly_i32(&argument, f);
                f.instruction(&Instruction::I32Eqz);
                self.emit_weak_error_if(
                    RuntimeErrorMessage::WEAKREF_TARGET_CANNOT_BE_HELD_WEAKLY,
                    &output,
                    exit,
                    f,
                )?;
            }
            WeakFamily::Registry => {
                self.emit_builtin_arg_to_value(0, &argument, f);
                self.emit_is_callable_i32(&argument, f)?;
                f.instruction(&Instruction::I32Eqz);
                self.emit_weak_error_if(
                    RuntimeErrorMessage::FINALIZATIONREGISTRY_CLEANUP_CALLBACK_IS_NOT_CALLABLE,
                    &output,
                    exit,
                    f,
                )?;
            }
        }
        self.emit_get_prototype_from_constructor(&new_target, family.prototype(), &output, f)?;
        output.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(exit, f);
        self.emit_reject_runtime_semantics(
            RuntimeSemanticRejection::UnavailableCapability(
                RuntimeUnavailableCapability::WeakReachability,
            ),
            f,
        );
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        argument.clear(f);
        new_target.clear(f);
        Ok(())
    }

    fn emit_unavailable_weak_receiver_error(
        &mut self,
        family: WeakFamily,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        self.compile_this_to_locals(&receiver, f)?;
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            family.receiver_error_message(WeakReceiverError::MissingInternalSlots),
            &output,
            f,
        )?;
        f.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            family.receiver_error_message(WeakReceiverError::NonObject),
            &output,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        receiver.clear(f);
        Ok(())
    }

    fn emit_whole_can_be_held_weakly_i32(&mut self, value: &ValueLocals, f: &mut Function) {
        let s = self.runtime_schema();
        let accepted = s.reserve_i32_local(f);
        self.emit_is_heap_object_like_tag_i32(value.tag(), f);
        accepted.store(f);
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Symbol.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let symbol = s
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<SymbolValue>(s, f), f);
        s.struct_type::<SymbolValue>()
            .field(SymbolValueSchema::REGISTRY_KEY)
            .read(&symbol, s, f)
            .reference()
            .is_null(f);
        accepted.store(f);
        symbol.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        accepted.load(f);
        s.release_i32_local(accepted, f);
    }
}
