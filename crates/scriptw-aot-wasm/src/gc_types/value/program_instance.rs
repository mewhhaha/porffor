//! Frame-local code, hook and immutable data ownership for compiled programs.

use super::*;
mod native_payload;
use crate::data::{PooledStringIndex, StringPool};
use crate::program_hooks::ProvidedHooks;
use crate::runtime_artifact::ModuleKind;
use crate::runtime_helpers::RuntimeHelperFunctionBase;
use crate::EmitError;

mod language_features;
mod literal_index;
mod metadata;
pub(crate) use metadata::ProgramMetadataView;

/// The final P pool, checked against its actual imported runtime boundary.
/// No scalar linear-memory address is accepted as a metadata image.
pub(crate) struct ProgramInstanceImage<'pool> {
    strings: &'pool StringPool,
    literal_base: u32,
    literal_count: u32,
    metadata_bytes: u32,
    language_features: scriptw_ir::LanguageFeaturePolicy,
}

impl<'pool> ProgramInstanceImage<'pool> {
    pub(crate) fn from_program_pool(
        strings: &'pool StringPool,
        kind: ModuleKind<'_>,
        language_features: scriptw_ir::LanguageFeaturePolicy,
    ) -> Result<Self, EmitError> {
        let ModuleKind::Program(runtime) = kind else {
            return Err(EmitError::unsupported(
                "a ProgramInstance image requires an actual linked P module",
            ));
        };
        runtime.layout().require_pool(strings)?;
        let boundary = strings.compiler_owned_boundary();
        let literal_base = boundary.strings();
        let literal_count = strings
            .pooled_string_count()
            .checked_sub(literal_base)
            .ok_or_else(|| EmitError::unsupported("P literals do not retain the runtime prefix"))?;
        let metadata_bytes = strings
            .bytes
            .len()
            .checked_sub(boundary.static_len())
            .and_then(|length| u32::try_from(length).ok())
            .ok_or_else(|| EmitError::unsupported("P metadata exceeds its GC byte image"))?;
        Ok(Self {
            strings,
            literal_base,
            literal_count,
            metadata_bytes,
            language_features,
        })
    }

    pub(crate) fn literal_count(&self) -> u32 {
        self.literal_count
    }

    pub(crate) fn literal_count_operand(&self) -> ProgramLiteralCount {
        ProgramLiteralCount(self.literal_count)
    }

    pub(crate) fn literal_index(
        &self,
        index: PooledStringIndex,
    ) -> Result<ProgramLiteralIndex, EmitError> {
        ProgramLiteralIndex::for_pool(self.strings, index)
    }
}

/// A table length minted only by the checked P image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProgramLiteralCount(u32);
impl ProgramLiteralCount {
    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

/// A P-local ordinal. Runtime/compiler literals use their separate R pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProgramLiteralIndex(u32);

/// The producer hands over its sole table root. Publication verifies complete
/// P-only coverage, and this wrapper exposes no mutable table capability.
pub(crate) struct ProgramLiteralRoots<'image, 'pool> {
    image: &'image ProgramInstanceImage<'pool>,
    table: GcLocal<PooledStringTable>,
}

impl<'image, 'pool> ProgramLiteralRoots<'image, 'pool> {
    pub(crate) fn admit_complete_table(
        schema: &RuntimeSchema,
        image: &'image ProgramInstanceImage<'pool>,
        table: GcLocal<PooledStringTable>,
        function: &mut Function,
    ) -> Self {
        schema
            .array_type::<PooledStringTable>()
            .length(&table, schema, function);
        function.program_literal_count(image.literal_count_operand());
        function.instruction(&Instruction::I32Ne);
        trap_if_true(function);
        let row = schema.reserve_i32_local(function);
        row.set_constant(0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row.load(function);
        function.program_literal_count(image.literal_count_operand());
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<PooledStringTable>()
            .read(&table, row, schema, function)
            .reference()
            .is_null(function);
        trap_if_true(function);
        row.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        row.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(row, function);
        Self { image, table }
    }
}

/// Copies the P passive metadata segment into the existing read-only GC byte
/// owner. Contained records must be P-relative before consumers migrate; this
/// allocation does not turn old absolute-pointer records into safe readers.
pub(crate) struct ProgramMetadataBytes<'image, 'pool> {
    image: &'image ProgramInstanceImage<'pool>,
    bytes: GcLocal<ImmutableByteArray>,
}

impl<'image, 'pool> ProgramMetadataBytes<'image, 'pool> {
    pub(crate) fn from_program_segment(
        schema: &RuntimeSchema,
        image: &'image ProgramInstanceImage<'pool>,
        function: &mut Function,
    ) -> Self {
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Const(image.metadata_bytes as i32));
        function.instruction(&Instruction::ArrayNewData {
            array_type_index: schema.array_type::<ImmutableByteArray>().type_index().raw(),
            array_data_index: StringPool::PROGRAM_STATIC_DATA_SEGMENT,
        });
        Self {
            image,
            bytes: schema
                .reserve_gc_local(function)
                .initialize(GcStackReference::new(), function),
        }
    }
}

/// Each absent hook is a null of that exact helper type. The constructor uses
/// the existing ProvidedHooks authority and helper index plan for this P.
pub(crate) struct ProgramInstanceHooks<'image, 'pool> {
    image: &'image ProgramInstanceImage<'pool>,
    dynamic_function: FunctionLocal<PreparedDynamicFunctionHook, Nullable>,
    script: FunctionLocal<PreparedScriptHook, Nullable>,
    realm_module_import: FunctionLocal<RealmModuleImportHook, Nullable>,
    module_body_reaction: FunctionLocal<ModuleBodyReactionHook, Nullable>,
    regexp_candidate: FunctionLocal<RegExpProgramCandidateHook, Nullable>,
}

impl<'image, 'pool> ProgramInstanceHooks<'image, 'pool> {
    pub(crate) fn from_program(
        schema: &RuntimeSchema,
        image: &'image ProgramInstanceImage<'pool>,
        functions: &crate::planning::FunctionMetaRegistry,
        base: RuntimeHelperFunctionBase,
        function: &mut Function,
    ) -> Self {
        let provided = ProvidedHooks::of(functions, image.strings);
        Self {
            image,
            dynamic_function: bind_hook(schema, provided, base, function),
            script: bind_hook(schema, provided, base, function),
            realm_module_import: bind_hook(schema, provided, base, function),
            module_body_reaction: bind_hook(schema, provided, base, function),
            regexp_candidate: bind_hook(schema, provided, base, function),
        }
    }
}

fn bind_hook<R: GcProgramHookRole>(
    schema: &RuntimeSchema,
    provided: ProvidedHooks,
    base: RuntimeHelperFunctionBase,
    function: &mut Function,
) -> FunctionLocal<R, Nullable> {
    if provided.provides(R::HOOK) {
        function.instruction(&Instruction::RefFunc(R::HOOK.helper().index(base)));
    } else {
        function.instruction(&Instruction::RefNull(HeapType::Concrete(
            R::SIGNATURE.type_index(),
        )));
    }
    schema
        .reserve_function_local(function)
        .initialize(FunctionStackReference { shape: PhantomData }, function)
}

/// One explicit live code/data owner. It is a frame-local GC root, not a
/// process/global current-P selector and not the execution Realm.
#[must_use = "retain the program owner through its frame/continuation and clear the local on exit"]
pub(crate) struct ProgramInstanceLocals {
    owner: GcLocal<ProgramInstance>,
}

impl ProgramInstanceLocals {
    pub(crate) fn publish(
        schema: &RuntimeSchema,
        image: &ProgramInstanceImage<'_>,
        hooks: ProgramInstanceHooks<'_, '_>,
        literals: ProgramLiteralRoots<'_, '_>,
        metadata: ProgramMetadataBytes<'_, '_>,
        function: &mut Function,
    ) -> Result<Self, EmitError> {
        if !std::ptr::eq(image, hooks.image)
            || !std::ptr::eq(image, literals.image)
            || !std::ptr::eq(image, metadata.image)
        {
            return Err(EmitError::unsupported(
                "ProgramInstance resources belong to different P images",
            ));
        }
        let source_payload = Self::fresh_native_payload_slot(schema, function);
        let owner = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ProgramInstance>().construct(
                (
                    GcOperand::nullable_function_reference(&hooks.dynamic_function),
                    GcOperand::nullable_function_reference(&hooks.script),
                    GcOperand::nullable_function_reference(&hooks.realm_module_import),
                    GcOperand::nullable_function_reference(&hooks.module_body_reaction),
                    GcOperand::nullable_function_reference(&hooks.regexp_candidate),
                    GcOperand::reference(&literals.table, schema),
                    GcOperand::i32(image.literal_base as i32),
                    GcOperand::program_literal_count(image.literal_count_operand()),
                    GcOperand::reference(&metadata.bytes, schema),
                    GcOperand::constant(image.language_features),
                    GcOperand::reference(&source_payload, schema),
                    GcOperand::null(schema),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        source_payload.clear(function);
        hooks.dynamic_function.clear(function);
        hooks.script.clear(function);
        hooks.realm_module_import.clear(function);
        hooks.module_body_reaction.clear(function);
        hooks.regexp_candidate.clear(function);
        literals.table.clear(function);
        metadata.bytes.clear(function);
        Ok(Self { owner })
    }

    pub(crate) fn compact_shapes(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcStackReference<CompactDataShape, Nullable> {
        schema
            .field(ProgramInstanceSchema::COMPACT_SHAPES)
            .read(&self.owner, schema, function)
            .reference()
    }
    pub(crate) fn publish_compact_shape(
        &self,
        shape: &GcLocal<CompactDataShape>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        schema.field(ProgramInstanceSchema::COMPACT_SHAPES).write(
            &self.owner,
            GcOperand::nullable_reference(shape, schema),
            schema,
            function,
        );
    }
    pub(crate) fn own_data_read_caches(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcStackReference<OwnDataReadCache, Nullable> {
        schema
            .field(ProgramInstanceSchema::OWN_DATA_READ_CACHES)
            .read(&self.owner, schema, function)
            .reference()
    }
    pub(crate) fn publish_own_data_read_cache(
        &self,
        cache: &GcLocal<OwnDataReadCache>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        schema
            .field(ProgramInstanceSchema::OWN_DATA_READ_CACHES)
            .write(
                &self.owner,
                GcOperand::nullable_reference(cache, schema),
                schema,
                function,
            );
    }

    pub(crate) fn bind(
        schema: &RuntimeSchema,
        reference: GcStackReference<ProgramInstance>,
        function: &mut Function,
    ) -> Self {
        Self {
            owner: schema
                .reserve_gc_local(function)
                .initialize(reference, function),
        }
    }

    /// A source function enters its defining P. An R-native function has no
    /// defining P and inherits the caller operand, independently of its Realm.
    pub(crate) fn for_function_entry(
        schema: &RuntimeSchema,
        context: &GcLocal<FunctionContext>,
        calling_program: &Self,
        function: &mut Function,
    ) -> Self {
        let defining = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::DEFINING_PROGRAM)
                .read(context, schema, function)
                .reference(),
            function,
        );
        let effective = Self::bind(
            schema,
            calling_program.owner.load(schema, function),
            function,
        );
        defining.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        effective.owner.replace(
            defining.load(schema, function).require_non_null(function),
            function,
        );
        function.instruction(&Instruction::End);
        defining.clear(function);
        effective
    }

    pub(crate) fn reference(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcStackReference<ProgramInstance> {
        self.owner.load(schema, function)
    }

    pub(crate) fn hook<R: GcProgramHookRole>(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> FunctionLocal<R, Nullable> {
        schema.reserve_function_local(function).initialize(
            schema
                .struct_type::<ProgramInstance>()
                .field(R::field())
                .read(&self.owner, schema, function)
                .callable(),
            function,
        )
    }

    pub(crate) fn literal(
        &self,
        schema: &RuntimeSchema,
        index: ProgramLiteralIndex,
        function: &mut Function,
    ) -> GcStackReference<StringValue> {
        let table = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ProgramInstance>()
                .field(ProgramInstanceSchema::LITERAL_STRINGS)
                .read(&self.owner, schema, function)
                .reference(),
            function,
        );
        let ordinal = schema.reserve_i32_local(function);
        function.program_literal_index(index);
        ordinal.store(function);
        let string = schema
            .array_type::<PooledStringTable>()
            .read(&table, ordinal, schema, function)
            .reference()
            .require_non_null(function);
        schema.release_i32_local(ordinal, function);
        table.clear(function);
        string
    }

    pub(crate) fn metadata(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcStackReference<ImmutableByteArray> {
        schema
            .struct_type::<ProgramInstance>()
            .field(ProgramInstanceSchema::METADATA)
            .read(&self.owner, schema, function)
            .reference()
    }

    pub(crate) fn clear(self, function: &mut Function) {
        self.owner.clear(function);
    }
}

/// Required FunctionContext capture input; absence means native inheritance,
/// never "infer the program from the Realm". The allocation owner must check
/// this mode against the actual executable kind before emitting the capture.
pub(crate) enum FunctionProgramOwner<'a> {
    Defining(&'a ProgramInstanceLocals),
    InheritCaller,
}

impl FunctionProgramOwner<'_> {
    pub(crate) fn capture_operand<'a>(
        &'a self,
        kind: ExecutableCodeKind,
        schema: &RuntimeSchema,
    ) -> Result<GcOperand<'a, GcRef<ProgramInstance>, Nullable>, EmitError> {
        match (kind, self) {
            (ExecutableCodeKind::JavaScript, Self::Defining(program)) => {
                Ok(GcOperand::nullable_reference(&program.owner, schema))
            }
            (
                ExecutableCodeKind::StandardBuiltin | ExecutableCodeKind::HostBuiltin,
                Self::InheritCaller,
            ) => Ok(GcOperand::null(schema)),
            (ExecutableCodeKind::JavaScript, Self::InheritCaller)
            | (
                ExecutableCodeKind::StandardBuiltin | ExecutableCodeKind::HostBuiltin,
                Self::Defining(_),
            ) => Err(EmitError::unsupported(
                "FunctionContext program capture differs from its executable kind",
            )),
        }
    }
}

fn trap_if_true(function: &mut Function) {
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}
