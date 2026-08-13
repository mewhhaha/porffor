use super::super::*;
use crate::functions::NewTargetPrototypeFallback;
use crate::operations::PrimitiveToStringAbruptRoute;

macro_rules! collection_wire_domain {
    ($name:ident { $($variant:ident = $word:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $name {
            $($variant),+
        }

        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            const fn word(self) -> u64 {
                match self {
                    $(Self::$variant => $word),+
                }
            }
        }

        const _: () = {
            let all = $name::ALL;
            let mut index = 0;
            while index < all.len() {
                assert!(all[index].word() == index as u64);
                index += 1;
            }
        };
    };
}

collection_wire_domain!(MapIteratorKind {
    Key = 0,
    Value = 1,
    KeyAndValue = 2,
});

collection_wire_domain!(SetIteratorKind {
    Value = 0,
    KeyAndValue = 1,
});

// Persisted cursor lifecycle. Exhaustion is terminal even if the collection
// later grows; see `contracts/ordered-collection-cursors.md`.
collection_wire_domain!(CollectionIteratorCursorState {
    Scanning = 0,
    Exhausted = 1,
});

// Receiver representations determine whether collection or iterator
// validation may read the ordinary Object brand layout. `Dynamic` is a
// compile-time kind, never a runtime value tag.
collection_wire_domain!(CollectionReceiverRepresentation {
    ObjectTagBrandLayout = 0,
    ObjectWithoutBrandLayout = 1,
    NonObject = 2,
    NonRuntime = 3,
});

macro_rules! collection_receiver_value_kinds {
    ($($kind:ident => $representation:ident),+ $(,)?) => {
        impl CollectionReceiverRepresentation {
            const VALUE_KINDS: &'static [ValueKind] = &[$(ValueKind::$kind),+];

            const fn from_value_kind(kind: ValueKind) -> Self {
                match kind {
                    $(ValueKind::$kind => Self::$representation),+
                }
            }
        }
    };
}

collection_receiver_value_kinds! {
    Undefined => NonObject,
    Null => NonObject,
    Boolean => NonObject,
    Number => NonObject,
    String => NonObject,
    Symbol => NonObject,
    Object => ObjectTagBrandLayout,
    Array => ObjectWithoutBrandLayout,
    Function => ObjectWithoutBrandLayout,
    Arguments => ObjectWithoutBrandLayout,
    BigInt => NonObject,
    Dynamic => NonRuntime,
}

// The two iterable ordered-collection layouts that share one cursor law.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrongCollectionCursor {
    Map,
    Set,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CollectionDataReceiverKind {
    Map,
    WeakMap,
    Set,
    WeakSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CollectionReceiverRequirement {
    Data(CollectionDataReceiverKind),
    Iterator(StrongCollectionCursor),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CollectionReceiverError {
    NonObject,
    MissingInternalSlots,
}

impl CollectionDataReceiverKind {
    fn brand(self) -> u64 {
        match self {
            Self::Map => OBJECT_INTERNAL_BRAND_MAP,
            Self::WeakMap => OBJECT_INTERNAL_BRAND_WEAK_MAP,
            Self::Set => OBJECT_INTERNAL_BRAND_SET,
            Self::WeakSet => OBJECT_INTERNAL_BRAND_WEAK_SET,
        }
    }

    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {
        match (self, error) {
            (Self::Map, CollectionReceiverError::NonObject) => {
                "Map method receiver is not an object"
            }
            (Self::Map, CollectionReceiverError::MissingInternalSlots) => {
                "Map method receiver does not have [[MapData]]"
            }
            (Self::WeakMap, CollectionReceiverError::NonObject) => {
                "WeakMap method receiver is not an object"
            }
            (Self::WeakMap, CollectionReceiverError::MissingInternalSlots) => {
                "WeakMap method receiver does not have [[WeakMapData]]"
            }
            (Self::Set, CollectionReceiverError::NonObject) => {
                "Set method receiver is not an object"
            }
            (Self::Set, CollectionReceiverError::MissingInternalSlots) => {
                "Set method receiver does not have [[SetData]]"
            }
            (Self::WeakSet, CollectionReceiverError::NonObject) => {
                "WeakSet method receiver is not an object"
            }
            (Self::WeakSet, CollectionReceiverError::MissingInternalSlots) => {
                "WeakSet method receiver does not have [[WeakSetData]]"
            }
        }
    }
}

impl CollectionReceiverRequirement {
    fn brand(self) -> u64 {
        match self {
            Self::Data(kind) => kind.brand(),
            Self::Iterator(cursor) => cursor.iterator_brand(),
        }
    }

    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {
        match self {
            Self::Data(kind) => kind.receiver_error_message(error),
            Self::Iterator(cursor) => cursor.receiver_error_message(error),
        }
    }
}

impl StrongCollectionCursor {
    fn name(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Set => "Set",
        }
    }

    fn iterator_brand(self) -> u64 {
        match self {
            Self::Map => OBJECT_INTERNAL_BRAND_MAP_ITERATOR,
            Self::Set => OBJECT_INTERNAL_BRAND_SET_ITERATOR,
        }
    }

    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {
        match (self, error) {
            (Self::Map, CollectionReceiverError::NonObject) => {
                "Map Iterator.prototype.next receiver is not an object"
            }
            (Self::Map, CollectionReceiverError::MissingInternalSlots) => {
                "Map Iterator.prototype.next receiver does not have [[Map]]"
            }
            (Self::Set, CollectionReceiverError::NonObject) => {
                "Set Iterator.prototype.next receiver is not an object"
            }
            (Self::Set, CollectionReceiverError::MissingInternalSlots) => {
                "Set Iterator.prototype.next receiver does not have [[Set]]"
            }
        }
    }

    fn collection_payload_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET,
            Self::Set => HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET,
        }
    }

    fn next_index_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET,
            Self::Set => HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET,
        }
    }

    fn state_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET,
            Self::Set => HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET,
        }
    }

    fn entries_ptr_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ENTRIES_PTR_OFFSET,
            Self::Set => HEAP_SET_ENTRIES_PTR_OFFSET,
        }
    }

    fn history_len_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ENTRIES_LEN_OFFSET,
            Self::Set => HEAP_SET_ENTRIES_LEN_OFFSET,
        }
    }

    fn entry_size(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ENTRY_SIZE,
            Self::Set => HEAP_SET_ENTRY_SIZE,
        }
    }

    fn entry_present_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ENTRY_PRESENT_OFFSET,
            Self::Set => HEAP_SET_ENTRY_PRESENT_OFFSET,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GroupByResult {
    Map,
    Object,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MapCollectionKind {
    Map,
    WeakMap,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetCollectionKind {
    Set,
    WeakSet,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MapConstructorTypeError {
    RequiresNew,
    SetterNotCallable,
    IteratorMethodNotCallable,
    IteratorMethodResultNotObject,
    IteratorNextNotCallable,
    IteratorNextResultNotObject,
    IteratorValueNotObject,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetConstructorTypeError {
    RequiresNew,
    AdderNotCallable,
    IteratorMethodNotCallable,
    IteratorMethodResultNotObject,
    IteratorNextNotCallable,
    IteratorNextResultNotObject,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollectionAlgorithmTypeError {
    MapConstructor(MapCollectionKind, MapConstructorTypeError),
    SetConstructor(SetCollectionKind, SetConstructorTypeError),
    ForEachCallback(StrongCollectionCursor),
}

impl CollectionAlgorithmTypeError {
    fn message(self) -> String {
        match self {
            Self::MapConstructor(collection, error) => {
                format!(
                    "{} constructor {}",
                    collection.name(),
                    error.message_suffix()
                )
            }
            Self::SetConstructor(collection, error) => {
                format!(
                    "{} constructor {}",
                    collection.name(),
                    error.message_suffix()
                )
            }
            Self::ForEachCallback(collection) => format!(
                "{}.prototype.forEach callback must be callable",
                collection.name()
            ),
        }
    }
}

impl MapConstructorTypeError {
    fn message_suffix(self) -> &'static str {
        match self {
            Self::RequiresNew => "requires new",
            Self::SetterNotCallable => "set method is not callable",
            Self::IteratorMethodNotCallable => "iterator method is not callable",
            Self::IteratorMethodResultNotObject => "iterator method must return an object",
            Self::IteratorNextNotCallable => "iterator next method is not callable",
            Self::IteratorNextResultNotObject => "iterator next result must be an object",
            Self::IteratorValueNotObject => "iterator value must be an object",
        }
    }
}

impl SetConstructorTypeError {
    fn message_suffix(self) -> &'static str {
        match self {
            Self::RequiresNew => "requires new",
            Self::AdderNotCallable => "add method is not callable",
            Self::IteratorMethodNotCallable => "iterator method is not callable",
            Self::IteratorMethodResultNotObject => "iterator method must return an object",
            Self::IteratorNextNotCallable => "iterator next method is not callable",
            Self::IteratorNextResultNotObject => "iterator next result must be an object",
        }
    }
}

impl SetCollectionKind {
    fn name(self) -> &'static str {
        match self {
            Self::Set => "Set",
            Self::WeakSet => "WeakSet",
        }
    }

    fn receiver_kind(self) -> CollectionDataReceiverKind {
        match self {
            Self::Set => CollectionDataReceiverKind::Set,
            Self::WeakSet => CollectionDataReceiverKind::WeakSet,
        }
    }

    fn prototype_global_index(self) -> u32 {
        match self {
            Self::Set => SET_PROTOTYPE_GLOBAL_INDEX,
            Self::WeakSet => WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
        }
    }

    fn realm_prototype_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET,
            Self::WeakSet => HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,
        }
    }

    fn brand(self) -> u64 {
        self.receiver_kind().brand()
    }

    fn record_size(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_RECORD_SIZE,
            Self::WeakSet => HEAP_WEAK_SET_RECORD_SIZE,
        }
    }

    fn entry_size(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRY_SIZE,
            Self::WeakSet => HEAP_WEAK_SET_ENTRY_SIZE,
        }
    }

    fn entries_ptr_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRIES_PTR_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRIES_PTR_OFFSET,
        }
    }

    fn entries_len_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRIES_LEN_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRIES_LEN_OFFSET,
        }
    }

    fn entries_cap_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRIES_CAP_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRIES_CAP_OFFSET,
        }
    }

    fn live_count_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_LIVE_COUNT_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_LIVE_COUNT_OFFSET,
        }
    }

    fn entry_present_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRY_PRESENT_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET,
        }
    }

    fn entry_value_tag_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET,
        }
    }

    fn entry_value_payload_offset(self) -> u64 {
        match self {
            Self::Set => HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            Self::WeakSet => HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
        }
    }
}

impl MapCollectionKind {
    fn name(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::WeakMap => "WeakMap",
        }
    }

    fn receiver_kind(self) -> CollectionDataReceiverKind {
        match self {
            Self::Map => CollectionDataReceiverKind::Map,
            Self::WeakMap => CollectionDataReceiverKind::WeakMap,
        }
    }

    fn prototype_global_index(self) -> u32 {
        match self {
            Self::Map => MAP_PROTOTYPE_GLOBAL_INDEX,
            Self::WeakMap => WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
        }
    }

    fn realm_prototype_offset(self) -> u64 {
        match self {
            Self::Map => HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET,
            Self::WeakMap => HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,
        }
    }

    fn record_size(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_RECORD_SIZE,
            Self::WeakMap => HEAP_WEAK_MAP_RECORD_SIZE,
        }
    }

    fn entry_size(self) -> u64 {
        match self {
            Self::Map => HEAP_MAP_ENTRY_SIZE,
            Self::WeakMap => HEAP_WEAK_MAP_ENTRY_SIZE,
        }
    }

    fn brand(self) -> u64 {
        self.receiver_kind().brand()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetAlgebraOperation {
    Difference,
    Intersection,
    SymmetricDifference,
    Union,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetPredicateOperation {
    IsDisjointFrom,
    IsSubsetOf,
    IsSupersetOf,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_collection_record_from_receiver(
        &mut self,
        requirement: CollectionReceiverRequirement,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let receiver_representation_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;

        function.instruction(&Instruction::I64Const(
            CollectionReceiverRepresentation::NonObject.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(receiver_representation_local));
        for kind in CollectionReceiverRepresentation::VALUE_KINDS
            .iter()
            .copied()
        {
            let representation = CollectionReceiverRepresentation::from_value_kind(kind);
            match representation {
                CollectionReceiverRepresentation::NonObject => {}
                CollectionReceiverRepresentation::ObjectTagBrandLayout
                | CollectionReceiverRepresentation::ObjectWithoutBrandLayout
                | CollectionReceiverRepresentation::NonRuntime => {
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(kind.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(representation.word() as i64));
                    function.instruction(&Instruction::LocalSet(receiver_representation_local));
                    function.instruction(&Instruction::End);
                }
            }
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        for representation in CollectionReceiverRepresentation::ALL.iter().copied() {
            function.instruction(&Instruction::LocalGet(receiver_representation_local));
            function.instruction(&Instruction::I64Const(representation.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match representation {
                CollectionReceiverRepresentation::ObjectTagBrandLayout => {
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                        receiver_brand_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(receiver_brand_local));
                    function.instruction(&Instruction::I64Const(requirement.brand() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                        record_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    self.emit_throw_current_function_realm_type_error(
                        requirement
                            .receiver_error_message(CollectionReceiverError::MissingInternalSlots),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                }
                CollectionReceiverRepresentation::ObjectWithoutBrandLayout => {
                    self.emit_throw_current_function_realm_type_error(
                        requirement
                            .receiver_error_message(CollectionReceiverError::MissingInternalSlots),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
                CollectionReceiverRepresentation::NonObject => {
                    self.emit_throw_current_function_realm_type_error(
                        requirement.receiver_error_message(CollectionReceiverError::NonObject),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
                CollectionReceiverRepresentation::NonRuntime => {
                    function.instruction(&Instruction::Unreachable);
                }
            }
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(receiver_representation_local);
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_strong_collection_iterator_record_from_receiver(
        &mut self,
        cursor: StrongCollectionCursor,
        iterator_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_collection_record_from_receiver(
            CollectionReceiverRequirement::Iterator(cursor),
            iterator_record_local,
            function,
        )
    }

    fn emit_map_record_from_receiver(
        &mut self,
        map_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_record_from_receiver(
            MapCollectionKind::Map,
            map_record_local,
            function,
        )
    }

    fn emit_map_collection_record_from_receiver(
        &mut self,
        collection_kind: MapCollectionKind,
        map_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_collection_record_from_receiver(
            CollectionReceiverRequirement::Data(collection_kind.receiver_kind()),
            map_record_local,
            function,
        )
    }

    fn emit_find_map_entry(
        &mut self,
        map_record_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        found_entry_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let stored_key_tag_local = self.reserve_temp_local();
        let stored_key_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            stored_key_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            stored_key_payload_local,
            function,
        );
        self.emit_tagged_payload_same_value_zero_i32(
            stored_key_tag_local,
            stored_key_payload_local,
            key_tag_local,
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::LocalSet(found_entry_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_key_payload_local);
        self.release_temp_local(stored_key_tag_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    fn emit_ensure_map_capacity(
        &mut self,
        map_record_local: u32,
        entries_ptr_local: u32,
        entries_len_local: u32,
        entries_cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_entries_ptr_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();
        let copied_value_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_ptr_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        for (entries_base_local, entry_local) in [
            (entries_ptr_local, old_entry_local),
            (new_entries_ptr_local, new_entry_local),
        ] {
            function.instruction(&Instruction::LocalGet(entries_base_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
        }
        for offset in [
            HEAP_MAP_ENTRY_PRESENT_OFFSET,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(
                old_entry_local,
                offset,
                copied_value_local,
                function,
            );
            self.store_i64_local_at_offset(new_entry_local, offset, copied_value_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            new_entries_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_CAP_OFFSET,
            new_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(new_entries_ptr_local));
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(entries_cap_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(copied_value_local);
        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_entries_ptr_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_can_be_held_weakly_i32(
        &mut self,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        let can_be_held_weakly_local = self.reserve_temp_local();
        let registry_key_payload_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(key_tag_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(can_be_held_weakly_local));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(can_be_held_weakly_local));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            key_payload_local,
            HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
            registry_key_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(registry_key_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(can_be_held_weakly_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(can_be_held_weakly_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);

        self.release_temp_local(registry_key_payload_local);
        self.release_temp_local(can_be_held_weakly_local);
    }

    fn emit_require_weak_key(
        &mut self,
        key_payload_local: u32,
        key_tag_local: u32,
        message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_collection_algorithm_type_error(
        &mut self,
        error: CollectionAlgorithmTypeError,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let message = error.message();
        self.emit_throw_current_function_realm_type_error(
            &message,
            self.result_local,
            self.result_tag_local,
            function,
        )
    }

    pub(crate) fn emit_map_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_constructor(MapCollectionKind::Map, function)
    }

    pub(crate) fn emit_weak_map_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_constructor(MapCollectionKind::WeakMap, function)
    }

    fn emit_map_collection_constructor(
        &mut self,
        collection_kind: MapCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let map_payload_local = self.reserve_temp_local();
        let map_tag_local = self.reserve_temp_local();
        let map_record_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let entry_payload_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::RequiresNew,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_new_target_prototype_to_locals(
            collection_kind.prototype_global_index(),
            NewTargetPrototypeFallback::RealmIntrinsic(collection_kind.realm_prototype_offset()),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(map_payload_local));
        self.emit_heap_alloc_const(collection_kind.record_size(), function)?;
        function.instruction(&Instruction::LocalSet(map_record_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * collection_kind.entry_size(), function)?;
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(map_record_local, HEAP_MAP_ENTRIES_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_CAP_OFFSET,
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(map_record_local, HEAP_MAP_LIVE_COUNT_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            map_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            collection_kind.brand(),
            function,
        );
        self.store_i64_const_at_offset(
            map_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            map_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            map_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            map_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(map_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(map_tag_local));

        self.emit_builtin_arg_to_locals(0, iterable_payload_local, iterable_tag_local, function);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            map_payload_local,
            map_tag_local,
            map_payload_local,
            map_tag_local,
            property_key_local,
            setter_payload_local,
            setter_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::SetterNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            iterable_payload_local,
            iterable_tag_local,
            iterable_object_payload_local,
            iterable_object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            property_key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::IteratorMethodNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::IteratorMethodResultNotObject,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            property_key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::IteratorNextNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let iterator_close = IteratorCloseOnThrowLocals {
            iterator_payload_local,
            iterator_tag_local,
            key_local: property_key_local,
            return_payload_local: iterator_method_payload_local,
            return_tag_local: iterator_method_tag_local,
            result_payload_local: call_result_payload_local,
            result_tag_local: call_result_tag_local,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::IteratorNextResultNotObject,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            property_key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            property_key_local,
            entry_payload_local,
            entry_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(entry_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::MapConstructor(
                collection_kind,
                MapConstructorTypeError::IteratorValueNotObject,
            ),
            function,
        )?;
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read_without_throw_propagation(
            entry_payload_local,
            entry_tag_local,
            entry_payload_local,
            entry_tag_local,
            property_key_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read_without_throw_propagation(
            entry_payload_local,
            entry_tag_local,
            entry_payload_local,
            entry_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_function_or_proxy_call_leave_throw_completion(
            setter_payload_local,
            setter_tag_local,
            map_payload_local,
            map_tag_local,
            &[
                (key_payload_local, key_tag_local),
                (value_payload_local, value_tag_local),
            ],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(map_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(map_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(next_result_tag_local);
        self.release_temp_local(next_result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(iterator_method_tag_local);
        self.release_temp_local(iterator_method_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(property_key_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(map_record_local);
        self.release_temp_local(map_tag_local);
        self.release_temp_local(map_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(iterable_object_tag_local);
        self.release_temp_local(iterable_object_payload_local);
        self.release_temp_local(iterable_tag_local);
        self.release_temp_local(iterable_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_from_entries(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let result_object_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let property_key_tag_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let entry_payload_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let primitive_key_payload_local = self.reserve_temp_local();
        let primitive_key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, iterable_payload_local, iterable_tag_local, function);
        self.emit_value_to_current_function_realm_object_locals(
            iterable_payload_local,
            iterable_tag_local,
            iterable_object_payload_local,
            iterable_object_tag_local,
            function,
        )?;
        self.emit_load_function_defining_realm_object_prototype(
            self.current_env_local,
            object_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(result_object_local));

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            property_key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.fromEntries iterator method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.fromEntries iterator method must return an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            property_key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.fromEntries iterator next method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let iterator_close = IteratorCloseOnThrowLocals {
            iterator_payload_local,
            iterator_tag_local,
            key_local: property_key_local,
            return_payload_local: iterator_method_payload_local,
            return_tag_local: iterator_method_tag_local,
            result_payload_local: close_result_payload_local,
            result_tag_local: close_result_tag_local,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.fromEntries iterator next result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            property_key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            property_key_local,
            entry_payload_local,
            entry_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(entry_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.fromEntries iterator value must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, payload_local, tag_local) in [
            ("0", key_payload_local, key_tag_local),
            ("1", value_payload_local, value_tag_local),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(index)));
            function.instruction(&Instruction::LocalSet(property_key_local));
            self.emit_object_read_without_throw_propagation(
                entry_payload_local,
                entry_tag_local,
                entry_payload_local,
                entry_tag_local,
                property_key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::String,
            key_payload_local,
            key_tag_local,
            primitive_key_payload_local,
            primitive_key_tag_local,
            ToPrimitiveAbruptRoute::IteratorCloseAndReturn(iterator_close),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(primitive_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A symbol property key is stored under the marked internal payload.
        function.instruction(&Instruction::LocalGet(primitive_key_payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(property_key_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_string_payload(
            primitive_key_payload_local,
            primitive_key_tag_local,
            PrimitiveToStringAbruptRoute::IteratorCloseAndReturn(iterator_close),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(property_key_tag_local));
        function.instruction(&Instruction::End);
        self.emit_object_define_enumerable_data(
            result_object_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            close_saved_aux_local,
            close_saved_completion_local,
            close_saved_tag_local,
            close_saved_payload_local,
            close_result_tag_local,
            close_result_payload_local,
            value_tag_local,
            value_payload_local,
            primitive_key_tag_local,
            primitive_key_payload_local,
            key_tag_local,
            key_payload_local,
            entry_tag_local,
            entry_payload_local,
            done_tag_local,
            done_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_tag_local,
            next_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_method_tag_local,
            iterator_method_payload_local,
            property_key_tag_local,
            property_key_local,
            result_object_local,
            object_prototype_local,
            iterable_object_tag_local,
            iterable_object_payload_local,
            iterable_tag_local,
            iterable_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_map_group_by(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_group_by(GroupByResult::Map, function)
    }

    pub(crate) fn emit_object_group_by(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_group_by(GroupByResult::Object, function)
    }

    fn emit_group_by(
        &mut self,
        result_kind: GroupByResult,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let nullish_items_message = match result_kind {
            GroupByResult::Map => "Map.groupBy items cannot be null or undefined",
            GroupByResult::Object => "Object.groupBy items cannot be null or undefined",
        };
        let callback_message = match result_kind {
            GroupByResult::Map => "Map.groupBy callback must be callable",
            GroupByResult::Object => "Object.groupBy callback must be callable",
        };
        let iterator_method_message = match result_kind {
            GroupByResult::Map => "Map.groupBy iterator method must be callable",
            GroupByResult::Object => "Object.groupBy iterator method must be callable",
        };
        let iterator_result_message = match result_kind {
            GroupByResult::Map => "Map.groupBy iterator method must return an object",
            GroupByResult::Object => "Object.groupBy iterator method must return an object",
        };
        let next_method_message = match result_kind {
            GroupByResult::Map => "Map.groupBy iterator next method must be callable",
            GroupByResult::Object => "Object.groupBy iterator next method must be callable",
        };
        let too_many_values_message = match result_kind {
            GroupByResult::Map => "Map.groupBy iterator produced too many values",
            GroupByResult::Object => "Object.groupBy iterator produced too many values",
        };
        let next_result_message = match result_kind {
            GroupByResult::Map => "Map.groupBy iterator next result must be an object",
            GroupByResult::Object => "Object.groupBy iterator next result must be an object",
        };
        let items_payload_local = self.reserve_temp_local();
        let items_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let group_key_payload_local = self.reserve_temp_local();
        let group_key_tag_local = self.reserve_temp_local();
        let primitive_key_payload_local = self.reserve_temp_local();
        let primitive_key_tag_local = self.reserve_temp_local();
        let callback_index_local = self.reserve_temp_local();
        let callback_index_payload_local = self.reserve_temp_local();
        let callback_index_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let map_prototype_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        let map_payload_local = self.reserve_temp_local();
        let map_tag_local = self.reserve_temp_local();
        let map_record_local = self.reserve_temp_local();
        let initial_entries_ptr_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let group_array_payload_local = self.reserve_temp_local();
        let group_array_tag_local = self.reserve_temp_local();
        let group_array_index_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();
        let stored_group_key_local = self.reserve_temp_local();
        let close_return_payload_local = self.reserve_temp_local();
        let close_return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, items_payload_local, items_tag_local, function);
        function.instruction(&Instruction::LocalGet(items_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(items_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            nullish_items_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(1, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            callback_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            items_payload_local,
            items_tag_local,
            iterable_object_payload_local,
            iterable_object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterable_object_payload_local,
            iterable_object_tag_local,
            items_payload_local,
            items_tag_local,
            key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            iterator_method_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            items_payload_local,
            items_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            iterator_result_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            next_method_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            function_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        if result_kind == GroupByResult::Map {
            self.emit_load_realm_intrinsic_prototype_or_global(
                function_realm_local,
                HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET,
                MAP_PROTOTYPE_GLOBAL_INDEX,
                map_prototype_local,
                function,
            );
        }
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            array_prototype_local,
            function,
        );

        if result_kind == GroupByResult::Map {
            self.emit_alloc_plain_object_with_prototype(Some(map_prototype_local), None, function)?;
            function.instruction(&Instruction::LocalSet(map_payload_local));
            self.emit_heap_alloc_const(HEAP_MAP_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(map_record_local));
            self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_MAP_ENTRY_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(initial_entries_ptr_local));
            self.store_i64_local_at_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_PTR_OFFSET,
                initial_entries_ptr_local,
                function,
            );
            self.store_i64_const_at_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_LEN_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_CAP_OFFSET,
                MIN_HEAP_CAPACITY,
                function,
            );
            self.store_i64_const_at_offset(
                map_record_local,
                HEAP_MAP_LIVE_COUNT_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                map_payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                CollectionDataReceiverKind::Map.brand(),
                function,
            );
            self.store_i64_const_at_offset(
                map_payload_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                BOXED_PRIMITIVE_KIND_NONE,
                function,
            );
            self.store_i64_const_at_offset(
                map_payload_local,
                HEAP_OBJECT_BOXED_TAG_OFFSET,
                ValueKind::Object.tag() as u64,
                function,
            );
            self.store_i64_local_at_offset(
                map_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                map_record_local,
                function,
            );
        } else {
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(map_payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(map_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(callback_index_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(group_array_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(callback_index_local));

        let iterator_close = IteratorCloseOnThrowLocals {
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local: close_return_payload_local,
            return_tag_local: close_return_tag_local,
            result_payload_local: close_result_payload_local,
            result_tag_local: close_result_tag_local,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(callback_index_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            too_many_values_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            next_result_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(callback_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(callback_index_payload_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (callback_index_payload_local, callback_index_tag_local),
            ],
            group_key_payload_local,
            group_key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if result_kind == GroupByResult::Object {
            self.emit_tagged_to_primitive_locals(
                ToPrimitiveHint::String,
                group_key_payload_local,
                group_key_tag_local,
                primitive_key_payload_local,
                primitive_key_tag_local,
                ToPrimitiveAbruptRoute::IteratorCloseAndReturn(iterator_close),
                function,
            )?;

            function.instruction(&Instruction::LocalGet(primitive_key_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            // A symbol property key is stored under the marked internal payload.
            function.instruction(&Instruction::LocalGet(primitive_key_payload_local));
            function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(group_key_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::LocalSet(group_key_tag_local));
            function.instruction(&Instruction::Else);
            self.emit_primitive_to_string_payload(
                primitive_key_payload_local,
                primitive_key_tag_local,
                PrimitiveToStringAbruptRoute::IteratorCloseAndReturn(iterator_close),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(group_key_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(group_key_tag_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(group_key_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(group_key_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(0.0.into()));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(group_key_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        if result_kind == GroupByResult::Map {
            self.emit_find_map_entry(
                map_record_local,
                group_key_payload_local,
                group_key_tag_local,
                found_entry_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(found_entry_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                found_entry_local,
                HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
                group_array_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(group_array_index_local));
            self.emit_alloc_array_payload_with_length(
                group_array_index_local,
                group_array_payload_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                group_array_payload_local,
                HEAP_PROTOTYPE_OFFSET,
                array_prototype_local,
                function,
            );
            self.store_i64_const_at_offset(
                group_array_payload_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                ValueKind::Array.tag() as u64,
                function,
            );

            self.load_i64_to_local_from_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_PTR_OFFSET,
                entries_ptr_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_LEN_OFFSET,
                entries_len_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_CAP_OFFSET,
                entries_cap_local,
                function,
            );
            self.emit_ensure_map_capacity(
                map_record_local,
                entries_ptr_local,
                entries_len_local,
                entries_cap_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(entries_ptr_local));
            function.instruction(&Instruction::LocalGet(entries_len_local));
            function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_const_at_offset(entry_local, HEAP_MAP_ENTRY_PRESENT_OFFSET, 1, function);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
                group_key_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
                group_key_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
                group_array_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
                group_array_payload_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(entries_len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entries_len_local));
            self.store_i64_local_at_offset(
                map_record_local,
                HEAP_MAP_ENTRIES_LEN_OFFSET,
                entries_len_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                map_record_local,
                HEAP_MAP_LIVE_COUNT_OFFSET,
                live_count_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(live_count_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(live_count_local));
            self.store_i64_local_at_offset(
                map_record_local,
                HEAP_MAP_LIVE_COUNT_OFFSET,
                live_count_local,
                function,
            );
            function.instruction(&Instruction::End);
        } else {
            self.load_i64_to_local_from_offset(
                map_payload_local,
                HEAP_PTR_OFFSET,
                entries_ptr_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                map_payload_local,
                HEAP_LEN_OFFSET,
                entries_len_local,
                function,
            );
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_entry_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(entries_cap_local));
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(entries_cap_local));
            function.instruction(&Instruction::LocalGet(entries_len_local));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(entries_ptr_local));
            function.instruction(&Instruction::LocalGet(entries_cap_local));
            function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_KEY_OFFSET,
                stored_group_key_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(group_key_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            function.instruction(&Instruction::LocalGet(stored_group_key_local));
            function.instruction(&Instruction::LocalGet(group_key_payload_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::Else);
            self.emit_string_payload_equality_i32(
                stored_group_key_local,
                group_key_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(entry_local));
            function.instruction(&Instruction::LocalSet(found_entry_local));
            function.instruction(&Instruction::Br(2));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(entries_cap_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entries_cap_local));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(found_entry_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(group_array_index_local));
            self.emit_alloc_array_payload_with_length(
                group_array_index_local,
                group_array_payload_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                group_array_payload_local,
                HEAP_PROTOTYPE_OFFSET,
                array_prototype_local,
                function,
            );
            self.store_i64_const_at_offset(
                group_array_payload_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                ValueKind::Array.tag() as u64,
                function,
            );
            self.emit_object_define_enumerable_data(
                map_payload_local,
                group_key_payload_local,
                group_array_payload_local,
                group_array_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.load_i64_to_local_from_offset(
                found_entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                group_array_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        self.load_i64_to_local_from_offset(
            group_array_payload_local,
            HEAP_LEN_OFFSET,
            group_array_index_local,
            function,
        );
        self.emit_array_write(
            group_array_payload_local,
            group_array_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(callback_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(callback_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(map_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(map_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(close_result_tag_local);
        self.release_temp_local(close_result_payload_local);
        self.release_temp_local(close_return_tag_local);
        self.release_temp_local(close_return_payload_local);
        self.release_temp_local(stored_group_key_local);
        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(group_array_index_local);
        self.release_temp_local(group_array_tag_local);
        self.release_temp_local(group_array_payload_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(initial_entries_ptr_local);
        self.release_temp_local(map_record_local);
        self.release_temp_local(map_tag_local);
        self.release_temp_local(map_payload_local);
        self.release_temp_local(array_prototype_local);
        self.release_temp_local(map_prototype_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(callback_index_tag_local);
        self.release_temp_local(callback_index_payload_local);
        self.release_temp_local(callback_index_local);
        self.release_temp_local(primitive_key_tag_local);
        self.release_temp_local(primitive_key_payload_local);
        self.release_temp_local(group_key_tag_local);
        self.release_temp_local(group_key_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(next_result_tag_local);
        self.release_temp_local(next_result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(iterator_method_tag_local);
        self.release_temp_local(iterator_method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(iterable_object_tag_local);
        self.release_temp_local(iterable_object_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(items_tag_local);
        self.release_temp_local(items_payload_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_clear(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.emit_map_record_from_receiver(map_record_local, function)?;
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for offset in [
            HEAP_MAP_ENTRY_PRESENT_OFFSET,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(entry_local, offset, 0, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(map_record_local, HEAP_MAP_LIVE_COUNT_OFFSET, 0, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_delete(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_delete(MapCollectionKind::Map, function)
    }

    pub(crate) fn emit_weak_map_prototype_delete(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_delete(MapCollectionKind::WeakMap, function)
    }

    fn emit_map_collection_prototype_delete(
        &mut self,
        collection_kind: MapCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        if collection_kind == MapCollectionKind::WeakMap {
            self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for offset in [
            HEAP_MAP_ENTRY_PRESENT_OFFSET,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(found_entry_local, offset, 0, function);
        }
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_get(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_get(MapCollectionKind::Map, function)
    }

    pub(crate) fn emit_weak_map_prototype_get(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_get(MapCollectionKind::WeakMap, function)
    }

    fn emit_map_collection_prototype_get(
        &mut self,
        collection_kind: MapCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();

        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        if collection_kind == MapCollectionKind::WeakMap {
            self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(found_entry_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_get_or_insert(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(MapCollectionKind::Map, false, function)
    }

    pub(crate) fn emit_map_prototype_get_or_insert_computed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(MapCollectionKind::Map, true, function)
    }

    pub(crate) fn emit_weak_map_prototype_get_or_insert(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(MapCollectionKind::WeakMap, false, function)
    }

    pub(crate) fn emit_weak_map_prototype_get_or_insert_computed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(MapCollectionKind::WeakMap, true, function)
    }

    fn emit_map_prototype_get_or_insert_inner(
        &mut self,
        collection_kind: MapCollectionKind,
        computed: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        if collection_kind == MapCollectionKind::WeakMap && !computed {
            self.emit_require_weak_key(
                key_payload_local,
                key_tag_local,
                "WeakMap key must be an object or unregistered symbol",
                function,
            )?;
        }
        if computed {
            self.emit_builtin_arg_to_locals(
                1,
                callback_payload_local,
                callback_tag_local,
                function,
            );
            self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                match collection_kind {
                    MapCollectionKind::Map => {
                        "Map.prototype.getOrInsertComputed callback must be callable"
                    }
                    MapCollectionKind::WeakMap => {
                        "WeakMap.prototype.getOrInsertComputed callback must be callable"
                    }
                },
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            if collection_kind == MapCollectionKind::WeakMap {
                self.emit_require_weak_key(
                    key_payload_local,
                    key_tag_local,
                    "WeakMap key must be an object or unregistered symbol",
                    function,
                )?;
            }
        } else {
            self.emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function);
        }

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);

        if computed {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(undefined_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(undefined_tag_local));
            self.emit_function_or_proxy_call_leave_throw_completion(
                callback_payload_local,
                callback_tag_local,
                undefined_payload_local,
                undefined_tag_local,
                &[(key_payload_local, key_tag_local)],
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
        }

        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        self.emit_ensure_map_capacity(
            map_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(entry_local, HEAP_MAP_ENTRY_PRESENT_OFFSET, 1, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_has(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_has(MapCollectionKind::Map, function)
    }

    pub(crate) fn emit_weak_map_prototype_has(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_has(MapCollectionKind::WeakMap, function)
    }

    fn emit_map_collection_prototype_has(
        &mut self,
        collection_kind: MapCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();

        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        if collection_kind == MapCollectionKind::WeakMap {
            self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(found_entry_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_for_each(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_map_record_from_receiver(map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::ForEachCallback(StrongCollectionCursor::Map),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_function_or_proxy_call_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (key_payload_local, key_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_set(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_set(MapCollectionKind::Map, function)
    }

    pub(crate) fn emit_weak_map_prototype_set(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_collection_prototype_set(MapCollectionKind::WeakMap, function)
    }

    fn emit_map_collection_prototype_set(
        &mut self,
        collection_kind: MapCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function);
        if collection_kind == MapCollectionKind::WeakMap {
            self.emit_require_weak_key(
                key_payload_local,
                key_tag_local,
                "WeakMap key must be an object or unregistered symbol",
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        self.emit_ensure_map_capacity(
            map_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(entry_local, HEAP_MAP_ENTRY_PRESENT_OFFSET, 1, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_size_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_map_record_from_receiver(map_record_local, function)?;
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    fn emit_exhausted_collection_iterator_result(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_advance_strong_collection_cursor(
        &mut self,
        cursor: StrongCollectionCursor,
        iterator_record_local: u32,
        collection_record_local: u32,
        entry_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let history_len_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            iterator_record_local,
            cursor.state_offset(),
            state_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        for state in CollectionIteratorCursorState::ALL.iter().copied() {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match state {
                CollectionIteratorCursorState::Scanning => {
                    function.instruction(&Instruction::Br(1));
                }
                CollectionIteratorCursorState::Exhausted => {
                    self.emit_exhausted_collection_iterator_result(
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            iterator_record_local,
            cursor.collection_payload_offset(),
            collection_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            cursor.next_index_offset(),
            index_local,
            function,
        );

        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            collection_record_local,
            cursor.history_len_offset(),
            history_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(history_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            iterator_record_local,
            cursor.state_offset(),
            CollectionIteratorCursorState::Exhausted.word(),
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            cursor.collection_payload_offset(),
            0,
            function,
        );
        self.emit_exhausted_collection_iterator_result(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            collection_record_local,
            cursor.entries_ptr_offset(),
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(cursor.entry_size() as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.store_i64_local_at_offset(
            iterator_record_local,
            cursor.next_index_offset(),
            index_local,
            function,
        );

        self.load_i64_to_local_from_offset(
            entry_local,
            cursor.entry_present_offset(),
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(present_local);
        self.release_temp_local(history_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(index_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    fn emit_map_iterator_create(
        &mut self,
        kind: MapIteratorKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_record_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        self.emit_map_record_from_receiver(map_record_local, function)?;
        function.instruction(&Instruction::GlobalGet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_map_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(iterator_payload_local));
        self.emit_heap_alloc_const(HEAP_MAP_ITERATOR_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(iterator_record_local));
        self.store_i64_local_at_offset(
            iterator_record_local,
            HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET,
            map_record_local,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_MAP_ITERATOR_KIND_OFFSET,
            kind.word(),
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET,
            CollectionIteratorCursorState::Scanning.word(),
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_MAP_ITERATOR,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            iterator_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(iterator_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_local);
        self.release_temp_local(iterator_record_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }

    pub(crate) fn emit_map_prototype_keys(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_iterator_create(MapIteratorKind::Key, function)
    }

    pub(crate) fn emit_map_prototype_values(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_iterator_create(MapIteratorKind::Value, function)
    }

    pub(crate) fn emit_map_prototype_entries(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_iterator_create(MapIteratorKind::KeyAndValue, function)
    }

    fn emit_map_iterator_value(
        &mut self,
        kind: MapIteratorKind,
        entry_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            MapIteratorKind::Key => {
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
                    value_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
                    value_tag_local,
                    function,
                );
            }
            MapIteratorKind::Value => {
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
                    value_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
                    value_tag_local,
                    function,
                );
            }
            MapIteratorKind::KeyAndValue => {
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
                    value_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
                    value_tag_local,
                    function,
                );

                let pair_local = self.reserve_temp_local();
                let pair_index_local = self.reserve_temp_local();
                let pair_value_payload_local = self.reserve_temp_local();
                let pair_value_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::LocalSet(pair_index_local));
                self.emit_alloc_array_payload_with_length(pair_index_local, pair_local, function)?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(pair_index_local));
                self.emit_array_write(
                    pair_local,
                    pair_index_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
                    pair_value_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
                    pair_value_tag_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(pair_index_local));
                self.emit_array_write(
                    pair_local,
                    pair_index_local,
                    pair_value_payload_local,
                    pair_value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(pair_local));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.release_temp_local(pair_value_tag_local);
                self.release_temp_local(pair_value_payload_local);
                self.release_temp_local(pair_index_local);
                self.release_temp_local(pair_local);
            }
        }
        Ok(())
    }

    pub(crate) fn emit_map_iterator_next(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_record_local = self.reserve_temp_local();
        let map_record_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_strong_collection_iterator_record_from_receiver(
            StrongCollectionCursor::Map,
            iterator_record_local,
            function,
        )?;

        self.emit_advance_strong_collection_cursor(
            StrongCollectionCursor::Map,
            iterator_record_local,
            map_record_local,
            entry_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_MAP_ITERATOR_KIND_OFFSET,
            kind_local,
            function,
        );

        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in MapIteratorKind::ALL.iter().copied() {
            function.instruction(&Instruction::LocalGet(kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_map_iterator_value(
                kind,
                entry_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(kind_local);
        self.release_temp_local(map_record_local);
        self.release_temp_local(iterator_record_local);
        Ok(())
    }

    fn emit_set_record_from_receiver(
        &mut self,
        set_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_record_from_receiver(
            SetCollectionKind::Set,
            set_record_local,
            function,
        )
    }

    fn emit_set_collection_record_from_receiver(
        &mut self,
        collection_kind: SetCollectionKind,
        set_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_collection_record_from_receiver(
            CollectionReceiverRequirement::Data(collection_kind.receiver_kind()),
            set_record_local,
            function,
        )
    }

    fn emit_find_set_entry(
        &mut self,
        set_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        found_entry_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_find_set_collection_entry(
            SetCollectionKind::Set,
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )
    }

    fn emit_find_set_collection_entry(
        &mut self,
        collection_kind: SetCollectionKind,
        set_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        found_entry_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let stored_value_tag_local = self.reserve_temp_local();
        let stored_value_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.entries_ptr_offset(),
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.entries_len_offset(),
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(collection_kind.entry_size() as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            collection_kind.entry_present_offset(),
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            collection_kind.entry_value_tag_offset(),
            stored_value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            collection_kind.entry_value_payload_offset(),
            stored_value_payload_local,
            function,
        );
        self.emit_tagged_payload_same_value_zero_i32(
            stored_value_tag_local,
            stored_value_payload_local,
            value_tag_local,
            value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::LocalSet(found_entry_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_value_payload_local);
        self.release_temp_local(stored_value_tag_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    fn emit_ensure_set_capacity(
        &mut self,
        set_record_local: u32,
        entries_ptr_local: u32,
        entries_len_local: u32,
        entries_cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ensure_set_collection_capacity(
            SetCollectionKind::Set,
            set_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )
    }

    fn emit_ensure_set_collection_capacity(
        &mut self,
        collection_kind: SetCollectionKind,
        set_record_local: u32,
        entries_ptr_local: u32,
        entries_len_local: u32,
        entries_cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_entries_ptr_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();
        let copied_value_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(collection_kind.entry_size() as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_ptr_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        for (entries_base_local, entry_local) in [
            (entries_ptr_local, old_entry_local),
            (new_entries_ptr_local, new_entry_local),
        ] {
            function.instruction(&Instruction::LocalGet(entries_base_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(collection_kind.entry_size() as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
        }
        for offset in [
            collection_kind.entry_present_offset(),
            collection_kind.entry_value_tag_offset(),
            collection_kind.entry_value_payload_offset(),
        ] {
            self.load_i64_to_local_from_offset(
                old_entry_local,
                offset,
                copied_value_local,
                function,
            );
            self.store_i64_local_at_offset(new_entry_local, offset, copied_value_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.entries_ptr_offset(),
            new_entries_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.entries_cap_offset(),
            new_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(new_entries_ptr_local));
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(entries_cap_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(copied_value_local);
        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_entries_ptr_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    fn emit_normalize_set_value_zero(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_add_set_record_value(
        &mut self,
        set_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let found_entry_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_normalize_set_value_zero(value_payload_local, value_tag_local, function);
        self.emit_find_set_entry(
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        self.emit_ensure_set_capacity(
            set_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(entry_local, HEAP_SET_ENTRY_PRESENT_OFFSET, 1, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            set_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            set_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(found_entry_local);
        Ok(())
    }

    fn emit_delete_set_record_value(
        &mut self,
        set_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let found_entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_find_set_entry(
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for offset in [
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(found_entry_local, offset, 0, function);
        }
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            set_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(live_count_local);
        self.release_temp_local(found_entry_local);
        Ok(())
    }

    fn emit_load_current_function_realm_set_prototype(
        &mut self,
        prototype_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(SET_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    fn emit_alloc_intrinsic_set(
        &mut self,
        set_payload_local: u32,
        set_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();

        self.emit_load_current_function_realm_set_prototype(prototype_local, function);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(set_payload_local));
        self.emit_heap_alloc_const(HEAP_SET_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(set_record_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_SET_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        self.store_i64_local_at_offset(
            set_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(set_record_local, HEAP_SET_ENTRIES_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            set_record_local,
            HEAP_SET_ENTRIES_CAP_OFFSET,
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(set_record_local, HEAP_SET_LIVE_COUNT_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            CollectionDataReceiverKind::Set.brand(),
            function,
        );
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            set_record_local,
            function,
        );

        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    fn emit_copy_set_record(
        &mut self,
        source_record_local: u32,
        result_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_add_set_record_value(
            result_record_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    pub(crate) fn emit_set_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_constructor(SetCollectionKind::Set, function)
    }

    pub(crate) fn emit_weak_set_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_constructor(SetCollectionKind::WeakSet, function)
    }

    fn emit_set_collection_constructor(
        &mut self,
        collection_kind: SetCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let set_payload_local = self.reserve_temp_local();
        let set_tag_local = self.reserve_temp_local();
        let set_record_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let adder_payload_local = self.reserve_temp_local();
        let adder_tag_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::RequiresNew,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_new_target_prototype_to_locals(
            collection_kind.prototype_global_index(),
            NewTargetPrototypeFallback::RealmIntrinsic(collection_kind.realm_prototype_offset()),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(set_payload_local));
        self.emit_heap_alloc_const(collection_kind.record_size(), function)?;
        function.instruction(&Instruction::LocalSet(set_record_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * collection_kind.entry_size(), function)?;
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.entries_ptr_offset(),
            entries_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(
            set_record_local,
            collection_kind.entries_len_offset(),
            0,
            function,
        );
        self.store_i64_const_at_offset(
            set_record_local,
            collection_kind.entries_cap_offset(),
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(
            set_record_local,
            collection_kind.live_count_offset(),
            0,
            function,
        );
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            collection_kind.brand(),
            function,
        );
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            set_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            set_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(set_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(set_tag_local));

        self.emit_builtin_arg_to_locals(0, iterable_payload_local, iterable_tag_local, function);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("add")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            set_payload_local,
            set_tag_local,
            set_payload_local,
            set_tag_local,
            key_local,
            adder_payload_local,
            adder_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(adder_tag_local, adder_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::AdderNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            iterable_payload_local,
            iterable_tag_local,
            iterable_object_payload_local,
            iterable_object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::IteratorMethodNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::IteratorMethodResultNotObject,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::IteratorNextNotCallable,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::SetConstructor(
                collection_kind,
                SetConstructorTypeError::IteratorNextResultNotObject,
            ),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            adder_payload_local,
            adder_tag_local,
            set_payload_local,
            set_tag_local,
            &[(value_payload_local, value_tag_local)],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local,
                iterator_tag_local,
                key_local,
                return_payload_local: iterator_method_payload_local,
                return_tag_local: iterator_method_tag_local,
                result_payload_local: call_result_payload_local,
                result_tag_local: call_result_tag_local,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(set_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(set_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(next_result_tag_local);
        self.release_temp_local(next_result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(iterator_method_tag_local);
        self.release_temp_local(iterator_method_payload_local);
        self.release_temp_local(adder_tag_local);
        self.release_temp_local(adder_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(set_record_local);
        self.release_temp_local(set_tag_local);
        self.release_temp_local(set_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(iterable_object_tag_local);
        self.release_temp_local(iterable_object_payload_local);
        self.release_temp_local(iterable_tag_local);
        self.release_temp_local(iterable_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_add(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_add(SetCollectionKind::Set, function)
    }

    pub(crate) fn emit_weak_set_prototype_add(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_add(SetCollectionKind::WeakSet, function)
    }

    fn emit_set_collection_prototype_add(
        &mut self,
        collection_kind: SetCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_set_collection_record_from_receiver(collection_kind, set_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        if collection_kind == SetCollectionKind::WeakSet {
            self.emit_require_weak_key(
                value_payload_local,
                value_tag_local,
                "WeakSet value must be an object or unregistered symbol",
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_find_set_collection_entry(
            collection_kind,
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.entries_ptr_offset(),
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.entries_len_offset(),
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.entries_cap_offset(),
            entries_cap_local,
            function,
        );
        self.emit_ensure_set_collection_capacity(
            collection_kind,
            set_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(collection_kind.entry_size() as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(
            entry_local,
            collection_kind.entry_present_offset(),
            1,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            collection_kind.entry_value_tag_offset(),
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            collection_kind.entry_value_payload_offset(),
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.entries_len_offset(),
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.live_count_offset(),
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.live_count_offset(),
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_clear(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.emit_set_record_from_receiver(set_record_local, function)?;
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for offset in [
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(entry_local, offset, 0, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(set_record_local, HEAP_SET_LIVE_COUNT_OFFSET, 0, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_delete(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_delete(SetCollectionKind::Set, function)
    }

    pub(crate) fn emit_weak_set_prototype_delete(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_delete(SetCollectionKind::WeakSet, function)
    }

    fn emit_set_collection_prototype_delete(
        &mut self,
        collection_kind: SetCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_set_collection_record_from_receiver(collection_kind, set_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        if collection_kind == SetCollectionKind::WeakSet {
            self.emit_can_be_held_weakly_i32(value_payload_local, value_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_find_set_collection_entry(
            collection_kind,
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        for offset in [
            collection_kind.entry_present_offset(),
            collection_kind.entry_value_tag_offset(),
            collection_kind.entry_value_payload_offset(),
        ] {
            self.store_i64_const_at_offset(found_entry_local, offset, 0, function);
        }
        self.load_i64_to_local_from_offset(
            set_record_local,
            collection_kind.live_count_offset(),
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            set_record_local,
            collection_kind.live_count_offset(),
            live_count_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_has(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_has(SetCollectionKind::Set, function)
    }

    pub(crate) fn emit_weak_set_prototype_has(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_collection_prototype_has(SetCollectionKind::WeakSet, function)
    }

    fn emit_set_collection_prototype_has(
        &mut self,
        collection_kind: SetCollectionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();

        self.emit_set_collection_record_from_receiver(collection_kind, set_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        if collection_kind == SetCollectionKind::WeakSet {
            self.emit_can_be_held_weakly_i32(value_payload_local, value_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_find_set_collection_entry(
            collection_kind,
            set_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    fn emit_get_set_record(
        &mut self,
        other_payload_local: u32,
        other_tag_local: u32,
        size_payload_local: u32,
        has_payload_local: u32,
        has_tag_local: u32,
        keys_payload_local: u32,
        keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let size_tag_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(other_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set method argument is not a set-like object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("size")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            other_payload_local,
            other_tag_local,
            other_payload_local,
            other_tag_local,
            key_local,
            size_payload_local,
            size_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_number_payload(size_tag_local, size_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(size_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(size_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(size_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like size is NaN",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            size_payload_local,
            size_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(size_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Set-like size is negative",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("has")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            other_payload_local,
            other_tag_local,
            other_payload_local,
            other_tag_local,
            key_local,
            has_payload_local,
            has_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(has_tag_local, has_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like has method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("keys")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            other_payload_local,
            other_tag_local,
            other_payload_local,
            other_tag_local,
            key_local,
            keys_payload_local,
            keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(keys_tag_local, keys_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like keys method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(size_tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_set_predicate_iterate_receiver(
        &mut self,
        operation: SetPredicateOperation,
        receiver_record_local: u32,
        other_payload_local: u32,
        other_tag_local: u32,
        has_payload_local: u32,
        has_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();

        debug_assert!(matches!(
            operation,
            SetPredicateOperation::IsDisjointFrom | SetPredicateOperation::IsSubsetOf
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_function_or_proxy_call_leave_throw_completion(
            has_payload_local,
            has_tag_local,
            other_payload_local,
            other_tag_local,
            &[(value_payload_local, value_tag_local)],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(call_result_tag_local, call_result_payload_local, function)?;
        if operation == SetPredicateOperation::IsSubsetOf {
            function.instruction(&Instruction::I32Eqz);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    fn emit_set_predicate_iterate_other(
        &mut self,
        operation: SetPredicateOperation,
        receiver_record_local: u32,
        other_payload_local: u32,
        other_tag_local: u32,
        keys_payload_local: u32,
        keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let return_payload_local = self.reserve_temp_local();
        let return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();

        debug_assert!(matches!(
            operation,
            SetPredicateOperation::IsDisjointFrom | SetPredicateOperation::IsSupersetOf
        ));
        self.emit_function_or_proxy_call_leave_throw_completion(
            keys_payload_local,
            keys_tag_local,
            other_payload_local,
            other_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like keys method must return an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like iterator next method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like iterator next result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_normalize_set_value_zero(value_payload_local, value_tag_local, function);
        self.emit_find_set_entry(
            receiver_record_local,
            value_payload_local,
            value_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        if operation == SetPredicateOperation::IsDisjointFrom {
            function.instruction(&Instruction::I64Ne);
        } else {
            function.instruction(&Instruction::I64Eq);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local,
            return_tag_local,
            close_result_payload_local,
            close_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(close_result_tag_local);
        self.release_temp_local(close_result_payload_local);
        self.release_temp_local(return_tag_local);
        self.release_temp_local(return_payload_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(next_result_tag_local);
        self.release_temp_local(next_result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_is_disjoint_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_predicate(SetPredicateOperation::IsDisjointFrom, function)
    }

    pub(crate) fn emit_set_prototype_is_subset_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_predicate(SetPredicateOperation::IsSubsetOf, function)
    }

    pub(crate) fn emit_set_prototype_is_superset_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_predicate(SetPredicateOperation::IsSupersetOf, function)
    }

    fn emit_set_predicate(
        &mut self,
        operation: SetPredicateOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_size_payload_local = self.reserve_temp_local();
        let has_payload_local = self.reserve_temp_local();
        let has_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let receiver_size_local = self.reserve_temp_local();

        self.emit_set_record_from_receiver(receiver_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, other_payload_local, other_tag_local, function);
        self.emit_get_set_record(
            other_payload_local,
            other_tag_local,
            other_size_payload_local,
            has_payload_local,
            has_tag_local,
            keys_payload_local,
            keys_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            receiver_size_local,
            function,
        );

        match operation {
            SetPredicateOperation::IsDisjointFrom => {
                function.instruction(&Instruction::LocalGet(receiver_size_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::LocalGet(other_size_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Le);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_set_predicate_iterate_receiver(
                    operation,
                    receiver_record_local,
                    other_payload_local,
                    other_tag_local,
                    has_payload_local,
                    has_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_set_predicate_iterate_other(
                    operation,
                    receiver_record_local,
                    other_payload_local,
                    other_tag_local,
                    keys_payload_local,
                    keys_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            SetPredicateOperation::IsSubsetOf => {
                function.instruction(&Instruction::LocalGet(receiver_size_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::LocalGet(other_size_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_set_predicate_iterate_receiver(
                    operation,
                    receiver_record_local,
                    other_payload_local,
                    other_tag_local,
                    has_payload_local,
                    has_tag_local,
                    function,
                )?;
            }
            SetPredicateOperation::IsSupersetOf => {
                function.instruction(&Instruction::LocalGet(receiver_size_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::LocalGet(other_size_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Lt);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_set_predicate_iterate_other(
                    operation,
                    receiver_record_local,
                    other_payload_local,
                    other_tag_local,
                    keys_payload_local,
                    keys_tag_local,
                    function,
                )?;
            }
        }

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(receiver_size_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(has_tag_local);
        self.release_temp_local(has_payload_local);
        self.release_temp_local(other_size_payload_local);
        self.release_temp_local(other_tag_local);
        self.release_temp_local(other_payload_local);
        self.release_temp_local(receiver_record_local);
        Ok(())
    }

    fn emit_set_algebra_iterate_receiver(
        &mut self,
        operation: SetAlgebraOperation,
        receiver_record_local: u32,
        result_record_local: u32,
        other_payload_local: u32,
        other_tag_local: u32,
        has_payload_local: u32,
        has_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_function_or_proxy_call_leave_throw_completion(
            has_payload_local,
            has_tag_local,
            other_payload_local,
            other_tag_local,
            &[(value_payload_local, value_tag_local)],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(call_result_tag_local, call_result_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        match operation {
            SetAlgebraOperation::Difference => self.emit_delete_set_record_value(
                result_record_local,
                value_payload_local,
                value_tag_local,
                function,
            )?,
            SetAlgebraOperation::Intersection => self.emit_add_set_record_value(
                result_record_local,
                value_payload_local,
                value_tag_local,
                function,
            )?,
            SetAlgebraOperation::SymmetricDifference | SetAlgebraOperation::Union => {
                unreachable!("receiver iteration is only used by difference and intersection")
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    fn emit_set_algebra_iterate_other(
        &mut self,
        operation: SetAlgebraOperation,
        receiver_record_local: u32,
        result_record_local: u32,
        other_payload_local: u32,
        other_tag_local: u32,
        keys_payload_local: u32,
        keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();

        self.emit_function_or_proxy_call_leave_throw_completion(
            keys_payload_local,
            keys_tag_local,
            other_payload_local,
            other_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like keys method must return an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like iterator next method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Set-like iterator next result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_normalize_set_value_zero(value_payload_local, value_tag_local, function);

        match operation {
            SetAlgebraOperation::Difference => self.emit_delete_set_record_value(
                result_record_local,
                value_payload_local,
                value_tag_local,
                function,
            )?,
            SetAlgebraOperation::Intersection => {
                self.emit_find_set_entry(
                    receiver_record_local,
                    value_payload_local,
                    value_tag_local,
                    found_entry_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(found_entry_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_add_set_record_value(
                    result_record_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            SetAlgebraOperation::SymmetricDifference => {
                self.emit_find_set_entry(
                    receiver_record_local,
                    value_payload_local,
                    value_tag_local,
                    found_entry_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(found_entry_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_delete_set_record_value(
                    result_record_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_add_set_record_value(
                    result_record_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            SetAlgebraOperation::Union => self.emit_add_set_record_value(
                result_record_local,
                value_payload_local,
                value_tag_local,
                function,
            )?,
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_entry_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(next_result_tag_local);
        self.release_temp_local(next_result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_difference(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_algebra(SetAlgebraOperation::Difference, function)
    }

    pub(crate) fn emit_set_prototype_intersection(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_algebra(SetAlgebraOperation::Intersection, function)
    }

    pub(crate) fn emit_set_prototype_symmetric_difference(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_algebra(SetAlgebraOperation::SymmetricDifference, function)
    }

    pub(crate) fn emit_set_prototype_union(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_algebra(SetAlgebraOperation::Union, function)
    }

    fn emit_set_algebra(
        &mut self,
        operation: SetAlgebraOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_size_payload_local = self.reserve_temp_local();
        let has_payload_local = self.reserve_temp_local();
        let has_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_record_local = self.reserve_temp_local();
        let receiver_size_local = self.reserve_temp_local();

        self.emit_set_record_from_receiver(receiver_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, other_payload_local, other_tag_local, function);
        self.emit_get_set_record(
            other_payload_local,
            other_tag_local,
            other_size_payload_local,
            has_payload_local,
            has_tag_local,
            keys_payload_local,
            keys_tag_local,
            function,
        )?;
        self.emit_alloc_intrinsic_set(result_payload_local, result_record_local, function)?;
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            receiver_size_local,
            function,
        );

        if operation != SetAlgebraOperation::Intersection {
            self.emit_copy_set_record(receiver_record_local, result_record_local, function)?;
        }

        match operation {
            SetAlgebraOperation::Difference | SetAlgebraOperation::Intersection => {
                function.instruction(&Instruction::LocalGet(receiver_size_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::LocalGet(other_size_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Le);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_set_algebra_iterate_receiver(
                    operation,
                    receiver_record_local,
                    result_record_local,
                    other_payload_local,
                    other_tag_local,
                    has_payload_local,
                    has_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_set_algebra_iterate_other(
                    operation,
                    receiver_record_local,
                    result_record_local,
                    other_payload_local,
                    other_tag_local,
                    keys_payload_local,
                    keys_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            SetAlgebraOperation::SymmetricDifference | SetAlgebraOperation::Union => {
                self.emit_set_algebra_iterate_other(
                    operation,
                    receiver_record_local,
                    result_record_local,
                    other_payload_local,
                    other_tag_local,
                    keys_payload_local,
                    keys_tag_local,
                    function,
                )?;
            }
        }

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(receiver_size_local);
        self.release_temp_local(result_record_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(has_tag_local);
        self.release_temp_local(has_payload_local);
        self.release_temp_local(other_size_payload_local);
        self.release_temp_local(other_tag_local);
        self.release_temp_local(other_payload_local);
        self.release_temp_local(receiver_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_for_each(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_set_record_from_receiver(set_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_collection_algorithm_type_error(
            CollectionAlgorithmTypeError::ForEachCallback(StrongCollectionCursor::Set),
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_SET_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_function_or_proxy_call_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (value_payload_local, value_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_size_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_set_record_from_receiver(set_record_local, function)?;
        self.load_i64_to_local_from_offset(
            set_record_local,
            HEAP_SET_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(live_count_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    fn emit_set_iterator_create(
        &mut self,
        kind: SetIteratorKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let set_record_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_record_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        self.emit_set_record_from_receiver(set_record_local, function)?;
        function.instruction(&Instruction::GlobalGet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_set_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(iterator_payload_local));
        self.emit_heap_alloc_const(HEAP_SET_ITERATOR_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(iterator_record_local));
        self.store_i64_local_at_offset(
            iterator_record_local,
            HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET,
            set_record_local,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_SET_ITERATOR_KIND_OFFSET,
            kind.word(),
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET,
            CollectionIteratorCursorState::Scanning.word(),
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_SET_ITERATOR,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            iterator_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(iterator_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_local);
        self.release_temp_local(iterator_record_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(set_record_local);
        Ok(())
    }

    pub(crate) fn emit_set_prototype_values(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_iterator_create(SetIteratorKind::Value, function)
    }

    pub(crate) fn emit_set_prototype_entries(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_iterator_create(SetIteratorKind::KeyAndValue, function)
    }

    fn emit_set_iterator_value(
        &mut self,
        kind: SetIteratorKind,
        entry_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );

        match kind {
            SetIteratorKind::Value => {}
            SetIteratorKind::KeyAndValue => {
                let pair_local = self.reserve_temp_local();
                let pair_index_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::LocalSet(pair_index_local));
                self.emit_alloc_array_payload_with_length(pair_index_local, pair_local, function)?;
                for pair_index in [0, 1] {
                    function.instruction(&Instruction::I64Const(pair_index));
                    function.instruction(&Instruction::LocalSet(pair_index_local));
                    self.emit_array_write(
                        pair_local,
                        pair_index_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::LocalGet(pair_local));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.release_temp_local(pair_index_local);
                self.release_temp_local(pair_local);
            }
        }
        Ok(())
    }

    pub(crate) fn emit_set_iterator_next(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_record_local = self.reserve_temp_local();
        let set_record_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_strong_collection_iterator_record_from_receiver(
            StrongCollectionCursor::Set,
            iterator_record_local,
            function,
        )?;

        self.emit_advance_strong_collection_cursor(
            StrongCollectionCursor::Set,
            iterator_record_local,
            set_record_local,
            entry_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_SET_ITERATOR_KIND_OFFSET,
            kind_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in SetIteratorKind::ALL.iter().copied() {
            function.instruction(&Instruction::LocalGet(kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_set_iterator_value(
                kind,
                entry_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(kind_local);
        self.release_temp_local(set_record_local);
        self.release_temp_local(iterator_record_local);
        Ok(())
    }
}
