use std::borrow::Cow;

use wasm_encoder::{ElementSection, Elements, RefType, TableSection, TableType};

use super::*;
use crate::abi::CallAbi;

/// The one type section and the typed indices assigned while constructing it.
pub(crate) struct ModuleTypeRegistry {
    section: TypeSection,
    runtime: RuntimeModuleTypes,
}

impl ModuleTypeRegistry {
    pub(crate) fn new() -> Self {
        let mut types = ModuleTypeSectionBuilder::new();
        types.function([], [ValType::I64]);
        types.arg_vector();
        assert_eq!(
            types.function(CallAbi::Js.parameter_types(), CallAbi::Js.result_types()),
            CallAbi::Js.type_index(),
            "JS call ABI type index changed without updating the function section",
        );
        assert_eq!(
            types.function(CallAbi::Raw.parameter_types(), CallAbi::Raw.result_types()),
            CallAbi::Raw.type_index(),
            "raw helper ABI type index changed without updating the function section",
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

        assert_eq!(
            types.function(
                CallAbi::Dispatch.parameter_types(),
                CallAbi::Dispatch.result_types()
            ),
            CallAbi::Dispatch.type_index(),
            "call dispatcher ABI type index changed without updating the function section",
        );

        assert_eq!(
            types.function(
                CallAbi::PreparedScript.parameter_types(),
                CallAbi::PreparedScript.result_types(),
            ),
            CallAbi::PreparedScript.type_index(),
            "prepared-script call ABI type index changed without updating the function section",
        );

        // Stack-guard wrappers call this leaf import before entering the
        // relocated body.
        assert_eq!(
            types.function([ValType::I32, ValType::I32], [ValType::I32]),
            STACK_GUARD_IMPORT_TYPE_INDEX,
            "stack-guard import type index changed without updating its declaration",
        );

        let runtime = RuntimeModuleTypes::register(&mut types.section);
        crate::gc_types::legacy_arguments::register(&mut types.section);
        assert_eq!(types.section.len(), HOST_GC_IMPORT_TYPE_INDEX);
        types.section.ty().function([], []);

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

    /// Replace the original `main` entry with its guard wrapper and append the
    /// compiled body after all remaining original and relocated functions.
    pub(crate) fn wrap_main_and_relocate(
        &mut self,
        wrapper: EmittedFunction,
        active_realm_global_index: u32,
    ) -> Result<(), String> {
        self.main_emitted_local_count = self
            .code
            .wrap_main_and_relocate(wrapper, active_realm_global_index)?;
        Ok(())
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

#[cfg(test)]
mod arg_vector_signature_tests {
    use super::*;

    #[test]
    fn native_argument_vector_has_distinct_js_dispatch_and_raw_signatures() {
        let reference = crate::gc_types::arg_vector::ref_type();
        assert_eq!(CallAbi::Js.parameter_types()[6], reference);
        assert_eq!(CallAbi::Dispatch.parameter_types()[5], reference);
        assert_eq!(CallAbi::Raw.parameter_types(), vec![ValType::I64; 7]);
        assert_eq!(CallAbi::PreparedScript.parameter_types()[6], reference);
        assert_eq!(
            crate::runtime_helpers::RuntimeHelperId::FunctionCall.type_index(),
            CallAbi::Dispatch.type_index(),
        );
        assert_eq!(
            crate::runtime_helpers::RuntimeHelperId::ObjectRead.type_index(),
            CallAbi::Raw.type_index(),
        );

        let registry = ModuleTypeRegistry::new();
        let mut module = Module::new();
        module.section(&registry.section);
        wasmparser::Validator::new()
            .validate_all(&module.finish())
            .expect("the central GC array and call signatures must validate");
    }
}
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
/// Function signatures and the internal argument-vector array are appended
/// here. The opaque rooted runtime-GC registration follows those signatures
/// without exposing its assigned indices back to module assembly.
struct ModuleTypeSectionBuilder {
    section: TypeSection,
    next_index: u32,
}

impl ModuleTypeSectionBuilder {
    fn new() -> Self {
        Self {
            section: TypeSection::new(),
            next_index: 0,
        }
    }

    fn function<P, R>(&mut self, params: P, results: R) -> u32
    where
        P: IntoIterator<Item = ValType>,
        P::IntoIter: ExactSizeIterator,
        R: IntoIterator<Item = ValType>,
        R::IntoIter: ExactSizeIterator,
    {
        let index = self.next_index;
        self.next_index = self
            .next_index
            .checked_add(1)
            .expect("Wasm type index overflow");
        self.section.ty().function(params, results);
        index
    }

    fn arg_vector(&mut self) {
        assert_eq!(self.next_index, 1, "argument vector precedes call ABIs");
        crate::gc_types::arg_vector::register(&mut self.section);
        self.next_index += 1;
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

    pub(crate) fn legacy_arguments_root(&mut self) {
        crate::gc_types::legacy_arguments::append_global(&mut self.section);
    }

    fn finish(self, runtime: RuntimeModuleTypes) -> FinalizedModuleGlobals {
        runtime.finalize_globals(self.section)
    }
}
