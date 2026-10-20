use super::*;

/// Keep each strong collection brand's two receiver errors inseparable.
#[derive(Clone, Copy)]
pub(super) enum StrongCollectionReceiverKind {
    MapData,
    SetData,
    MapIterator,
    SetIterator,
}

enum CollectionReceiverError {
    NonObject,
    MissingInternalSlots,
}

impl StrongCollectionReceiverKind {
    fn error_message(self, error: CollectionReceiverError) -> RuntimeErrorMessage {
        match (self, error) {
            (Self::MapData, CollectionReceiverError::NonObject) => {
                RuntimeErrorMessage::MAP_METHOD_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::MapData, CollectionReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::MAP_METHOD_RECEIVER_DOES_NOT_HAVE_MAPDATA
            }
            (Self::SetData, CollectionReceiverError::NonObject) => {
                RuntimeErrorMessage::SET_METHOD_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::SetData, CollectionReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::SET_METHOD_RECEIVER_DOES_NOT_HAVE_SETDATA
            }
            (Self::MapIterator, CollectionReceiverError::NonObject) => {
                RuntimeErrorMessage::MAP_ITERATOR_PROTOTYPE_NEXT_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::MapIterator, CollectionReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::MAP_ITERATOR_PROTOTYPE_NEXT_RECEIVER_DOES_NOT_HAVE_MAP
            }
            (Self::SetIterator, CollectionReceiverError::NonObject) => {
                RuntimeErrorMessage::SET_ITERATOR_PROTOTYPE_NEXT_RECEIVER_IS_NOT_AN_OBJECT
            }
            (Self::SetIterator, CollectionReceiverError::MissingInternalSlots) => {
                RuntimeErrorMessage::SET_ITERATOR_PROTOTYPE_NEXT_RECEIVER_DOES_NOT_HAVE_SET
            }
        }
    }
}

impl FunctionBuilder<'_> {
    /// Called after a failed GC brand check. Do not coerce the receiver,
    /// inspect a prototype, unwrap a Proxy or observe a Proxy trap.
    pub(super) fn emit_collection_receiver_type_error(
        &mut self,
        receiver: &ValueLocals,
        kind: StrongCollectionReceiverKind,
        result: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            kind.error_message(CollectionReceiverError::MissingInternalSlots),
            result,
            f,
        )?;
        f.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            kind.error_message(CollectionReceiverError::NonObject),
            result,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
}
