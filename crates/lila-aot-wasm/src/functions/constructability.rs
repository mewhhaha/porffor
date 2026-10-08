//! Constructability is retained by each concrete callable representation.

use super::*;
use crate::gc_types::{
    BoundFunction, BoundFunctionSchema, ExecutableCode, ExecutableCodeSchema, FunctionObject,
    FunctionObjectSchema, FunctionProtocolCode, GcI32Constant, GcNullability, ProxyCallCapability,
    ProxyObject, ProxyObjectSchema, ValueLocals,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_is_constructor_i32(&mut self, value: &ValueLocals, function: &mut Function) {
        let schema = self.runtime_schema();
        let result = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        result.store(function);
        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<FunctionObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::If(BlockType::Empty));
        let callable = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<FunctionObject>(schema, function),
            function,
        );
        let code = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CODE)
                .read(&callable, schema, function)
                .reference(),
            function,
        );
        let protocol = schema.reserve_i32_local(function);
        schema
            .struct_type::<ExecutableCode>()
            .field(ExecutableCodeSchema::PROTOCOL)
            .read(&code, schema, function)
            .store(protocol, function);
        FunctionProtocolCode::emit_is_constructable(protocol, function);
        result.store(function);
        // HTMLDDA is the host's call-only exotic function regardless of a
        // source protocol; ordinary source functions never gain this slot.
        schema
            .struct_type::<FunctionObject>()
            .field(FunctionObjectSchema::IS_HTMLDDA)
            .read(&callable, schema, function)
            .store(protocol, function);
        protocol.load(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I32Const(0));
        result.store(function);
        function.instruction(&Instruction::End);
        schema.release_i32_local(protocol, function);
        code.clear(function);
        callable.clear(function);
        function.instruction(&Instruction::Else);

        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<BoundFunction>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::If(BlockType::Empty));
        let bound = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<BoundFunction>(schema, function),
            function,
        );
        schema
            .struct_type::<BoundFunction>()
            .field(BoundFunctionSchema::CONSTRUCTABLE)
            .read(&bound, schema, function)
            .store(result, function);
        bound.clear(function);
        function.instruction(&Instruction::Else);

        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ProxyObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::If(BlockType::Empty));
        let proxy = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<ProxyObject>(schema, function),
            function,
        );
        let capability = schema.reserve_i32_local(function);
        schema
            .struct_type::<ProxyObject>()
            .field(ProxyObjectSchema::CALL_CAPABILITY)
            .read(&proxy, schema, function)
            .store(capability, function);
        capability.load(function);
        function.instruction(&Instruction::I32Const(
            ProxyCallCapability::CallAndConstruct.encode(),
        ));
        function.instruction(&Instruction::I32Eq);
        result.store(function);
        schema.release_i32_local(capability, function);
        proxy.clear(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        result.load(function);
        schema.release_i32_local(result, function);
    }
}
