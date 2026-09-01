use std::borrow::Cow;

use wasm_encoder::{ElementSection, Elements, RefType, TableSection, TableType};

use super::*;

/// The one type section and the typed indices assigned while constructing it.
pub(crate) struct ModuleTypeRegistry {
    section: TypeSection,
    runtime: RuntimeModuleTypes,
}

impl ModuleTypeRegistry {
    pub(crate) fn new() -> Self {
        let mut types = ModuleTypeSectionBuilder::new();
        types.function([], [ValType::I64]);
        types.function(
            function_param_types(),
            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        );
        types.function([ValType::I64], [ValType::I64]);
        types.function(
            [
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
            ],
            [],
        );
        types.function(
            [
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
            ],
            [],
        );
        types.function(
            [
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
                ValType::I64,
            ],
            [ValType::I64],
        );
        types.function([ValType::I64, ValType::I64], [ValType::I64]);
        types.function([ValType::I64], [ValType::I64, ValType::I64]);
        types.function([ValType::I32, ValType::I32], []);
        types.function([ValType::F64, ValType::F64], [ValType::F64]);
        types.function([], [ValType::I32]);
        types.function([], [ValType::I64]);
        types.function([ValType::I64], []);
        types.function([ValType::I64, ValType::I64, ValType::I64], [ValType::I64]);
        types.function([], [ValType::F64]);

        let runtime = RuntimeModuleTypes::register(&mut types.section);

        Self {
            section: types.finish(),
            runtime,
        }
    }

    /// Consumes every scalar/dynamic global and returns the sealed section with
    /// the runtime schema derived from its actual final scalar index.
    pub(crate) fn finalize_globals(
        self,
        globals: ModuleGlobalSectionBuilder,
    ) -> FinalizedModuleSections {
        let Self { section, runtime } = self;
        FinalizedModuleSections {
            types: section,
            globals: globals.finish(runtime),
        }
    }
}

/// The type and global sections finalized as one consume-once package.
///
/// Consuming [`ModuleTypeRegistry`] prevents a second global package from being
/// finalized against the same typed registry. The only next transition accepts
/// the emitter's closed main-compilation plan, compiles it against this exact
/// package and starts package-owned code with that main body.
pub(crate) struct FinalizedModuleSections {
    types: TypeSection,
    globals: FinalizedModuleGlobals,
}

impl FinalizedModuleSections {
    pub(crate) fn compile_main(
        self,
        compilation: MainFunctionCompilation<'_>,
    ) -> Result<CompiledModulePackage, EmitError> {
        let mut code = ModuleCode::new(compilation.first_wasm_index());
        let main_emitted_local_count = compilation.compile_into(&self.globals, &mut code)?;
        Ok(CompiledModulePackage {
            types: self.types,
            globals: self.globals,
            code,
            main_emitted_local_count,
        })
    }
}

/// Type, rooted globals and code whose first body is main compiled against that
/// exact root, sealed together behind one consuming assembly operation.
///
/// None of the three Wasm sections has an independent append method. Remaining
/// bodies extend package-owned code by mutable borrow; only the final assembly
/// transition consumes the package. Normal Rust code therefore cannot combine
/// A's main with B's types or globals or reuse either package for another
/// module.
pub(crate) struct CompiledModulePackage {
    types: TypeSection,
    globals: FinalizedModuleGlobals,
    code: ModuleCode,
    main_emitted_local_count: u32,
}

impl CompiledModulePackage {
    pub(crate) fn append_remaining_functions(&mut self, remaining_functions: Vec<EmittedFunction>) {
        for function in remaining_functions {
            self.code.push(function);
        }
    }

    pub(crate) const fn main_emitted_local_count(&self) -> u32 {
        self.main_emitted_local_count
    }

    pub(crate) fn append_to_module(
        self,
        module: &mut Module,
        sections: ModuleAssemblySections,
    ) -> ModuleFunctionTable {
        let Self {
            types,
            globals,
            code,
            main_emitted_local_count: _,
        } = self;
        let (code, function_table) = code.finish();
        let ModuleAssemblySections {
            imports,
            functions,
            callable_function_table,
            memories,
            exports,
            data,
        } = sections;
        let CallableFunctionTableSections { tables, elements } = callable_function_table;

        module.section(&types);
        module.section(&imports);
        module.section(&functions);
        module.section(&tables);
        if let Some(memories) = memories {
            module.section(&memories);
        }
        module.section(&globals);
        module.section(&exports);
        module.section(&elements);
        module.section(&code);
        if let Some(data) = data {
            module.section(&data);
        }

        function_table
    }
}

/// The table section and its active element segment, constructed from one
/// callable-function range.
///
/// Keeping both encoder sections private prevents module assembly callers from
/// supplying an empty section or pairing entries from different ranges.
struct CallableFunctionTableSections {
    tables: TableSection,
    elements: ElementSection,
}

impl CallableFunctionTableSections {
    fn new(first_callable_wasm_index: u32, callable_function_count: usize) -> Self {
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: callable_function_count as u64,
            maximum: Some(callable_function_count as u64),
            table64: false,
            shared: false,
        });

        let mut elements = ElementSection::new();
        let function_indexes = (first_callable_wasm_index
            ..first_callable_wasm_index + callable_function_count as u32)
            .collect::<Vec<_>>();
        elements.active(
            Some(0),
            &ConstExpr::i32_const(0),
            Elements::Functions(Cow::Owned(function_indexes)),
        );

        Self { tables, elements }
    }
}

/// The non-runtime core sections that must be interleaved with one compiled
/// runtime package according to Wasm's canonical section order.
pub(crate) struct ModuleAssemblySections {
    imports: ImportSection,
    functions: FunctionSection,
    callable_function_table: CallableFunctionTableSections,
    memories: Option<MemorySection>,
    exports: ExportSection,
    data: Option<DataSection>,
}

impl ModuleAssemblySections {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        imports: ImportSection,
        functions: FunctionSection,
        first_callable_wasm_index: u32,
        callable_function_count: usize,
        memories: Option<MemorySection>,
        exports: ExportSection,
        data: Option<DataSection>,
    ) -> Self {
        Self {
            imports,
            functions,
            callable_function_table: CallableFunctionTableSections::new(
                first_callable_wasm_index,
                callable_function_count,
            ),
            memories,
            exports,
            data,
        }
    }
}

// These function-pointer assignments are compile-time lifecycle gates. Main
// compilation and module assembly must consume their package states, while
// adding the already-compiled internal bodies may only borrow the one compiled
// package. A future edit that weakens those ownership transitions no longer
// matches these function types and fails to compile.
const _: for<'a> fn(
    FinalizedModuleSections,
    MainFunctionCompilation<'a>,
) -> Result<CompiledModulePackage, EmitError> = FinalizedModuleSections::compile_main;
const _: fn() -> ModuleTypeRegistry = ModuleTypeRegistry::new;
const _: fn(
    ImportSection,
    FunctionSection,
    u32,
    usize,
    Option<MemorySection>,
    ExportSection,
    Option<DataSection>,
) -> ModuleAssemblySections = ModuleAssemblySections::new;
const _: fn(&mut CompiledModulePackage, Vec<EmittedFunction>) =
    CompiledModulePackage::append_remaining_functions;
const _: fn(CompiledModulePackage, &mut Module, ModuleAssemblySections) -> ModuleFunctionTable =
    CompiledModulePackage::append_to_module;

/// Single-use owner of the module type section.
///
/// Function signatures are appended here. The opaque runtime-GC registration
/// operation borrows the same section once, so GC types follow those signatures
/// without exposing their assigned indices back to module assembly.
struct ModuleTypeSectionBuilder {
    section: TypeSection,
}

impl ModuleTypeSectionBuilder {
    fn new() -> Self {
        Self {
            section: TypeSection::new(),
        }
    }

    fn function<P, R>(&mut self, params: P, results: R)
    where
        P: IntoIterator<Item = ValType>,
        P::IntoIter: ExactSizeIterator,
        R: IntoIterator<Item = ValType>,
        R::IntoIter: ExactSizeIterator,
    {
        self.section.ty().function(params, results);
    }

    fn finish(self) -> TypeSection {
        self.section
    }
}

/// The sole construction path for the module global section.
///
/// The inner encoder is private and this wrapper is not a Wasm section. A
/// caller must finish it through the type registry before the result can be
/// attached to a module. Finalization both appends the GC root and creates the
/// only matching runtime schema, which makes omission or a separately planned
/// root index a compile error rather than an out-of-band convention.
pub(crate) struct ModuleGlobalSectionBuilder {
    section: GlobalSection,
}

impl ModuleGlobalSectionBuilder {
    pub(crate) fn new() -> Self {
        Self {
            section: GlobalSection::new(),
        }
    }

    pub(crate) fn global(&mut self, global_type: GlobalType, init_expr: &ConstExpr) -> &mut Self {
        self.section.global(global_type, init_expr);
        self
    }

    fn finish(self, runtime: RuntimeModuleTypes) -> FinalizedModuleGlobals {
        runtime.finalize_globals(self.section)
    }
}
