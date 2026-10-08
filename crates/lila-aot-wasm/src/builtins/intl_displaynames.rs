//! DisplayNames retains complete GC options and lossless observed code units.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::gc_types::*;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{
    DisplayNamesConfigurationWord as DnWord, DisplayNamesFallback, DisplayNamesLanguageDisplay,
    DisplayNamesStyle, DisplayNamesType, DisplayNamesWireOperation, DISPLAY_NAMES_WIRE_VERSION,
};
mod construction;
mod pool;
mod render;
mod resolved;
pub(crate) use pool::intl_display_names_pool_strings;
const DN_CONSTRUCT_ERROR: RuntimeErrorMessage = RuntimeErrorMessage::INTL_DISPLAYNAMES_REQUIRES_NEW;
const DN_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_DISPLAYNAMES_METHOD_REQUIRES_A_DISPLAYNAMES_RECEIVER;
const DN_REQUIRED_TYPE_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_DISPLAYNAMES_REQUIRES_A_TYPE_OPTION;
/// Minted only after the original whole code completes observable ToString.
struct CompletedDisplayNameCodeLocals(GcLocal<StringValue>);
enum DisplayNamesProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Name {
        record: &'a GcLocal<IntlDisplayNamesObject>,
        code: &'a CompletedDisplayNameCodeLocals,
    },
}
struct DisplayNamesProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: DisplayNamesWireOperation,
}
impl DisplayNamesProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            DISPLAY_NAMES_WIRE_VERSION,
            u64::from(self.operation.code()) * 2 + 1,
        ] {
            reader.read_u64(word, schema, f);
            word.load(f);
            f.instruction(&Instruction::I64Const(expected as i64));
            f.instruction(&Instruction::I64Ne);
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable);
            f.instruction(&Instruction::End);
        }
        schema.release_i64_local(word, f);
        reader
    }
    fn clear(self, f: &mut Function) {
        self.bytes.clear(f);
    }
}
impl FunctionBuilder<'_> {
    fn emit_display_names_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDisplayNamesObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DisplayNames receiver lacks callable entry")
                })?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlDisplayNamesObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DN_RECEIVER_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<IntlDisplayNamesObject>(schema, f), f);
        value.clear(f);
        Ok(record)
    }
    fn emit_display_names_provider_call(
        &mut self,
        request: DisplayNamesProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<DisplayNamesProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            DisplayNamesProviderRequest::Resolve { .. } => DisplayNamesWireOperation::ResolveLocale,
            DisplayNamesProviderRequest::Supported { .. } => {
                DisplayNamesWireOperation::SupportedLocales
            }
            DisplayNamesProviderRequest::Name { .. } => DisplayNamesWireOperation::DisplayName,
        };
        let message = IntlByteArrayBuilder::with_operation(operation.global_operation(), schema, f);
        message.append_u64_constant(DISPLAY_NAMES_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, f);
        let word = schema.reserve_i64_local(f);
        match request {
            DisplayNamesProviderRequest::Resolve { locales, matcher }
            | DisplayNamesProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
            }
            DisplayNamesProviderRequest::Name { record, code } => {
                let dn = schema.struct_type::<IntlDisplayNamesObject>();
                let locale = schema.reserve_gc_local(f).initialize(
                    dn.field(IntlDisplayNamesObjectSchema::LOCALE)
                        .read(record, schema, f)
                        .reference(),
                    f,
                );
                message.append_utf8(&locale, schema, f);
                locale.clear(f);
                let selected = schema.reserve_i32_local(f);
                for field in DnWord::ALL {
                    match field {
                        DnWord::Type => dn
                            .field(IntlDisplayNamesObjectSchema::TYPE)
                            .read(record, schema, f)
                            .store(selected, f),
                        DnWord::Style => dn
                            .field(IntlDisplayNamesObjectSchema::STYLE)
                            .read(record, schema, f)
                            .store(selected, f),
                        DnWord::Fallback => dn
                            .field(IntlDisplayNamesObjectSchema::FALLBACK)
                            .read(record, schema, f)
                            .store(selected, f),
                        DnWord::LanguageDisplay => {
                            dn.field(IntlDisplayNamesObjectSchema::LANGUAGE_DISPLAY)
                                .read(record, schema, f)
                                .store(selected, f);
                            selected.load(f);
                            f.instruction(&Instruction::I32Const(-1));
                            f.instruction(&Instruction::I32Eq);
                            self.open_frame(ControlFrameKind::If, f);
                            set_i32(selected, 0, f);
                            self.pop_control(ControlFrameKind::If);
                            f.instruction(&Instruction::End);
                        }
                    }
                    selected.load(f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    word.store(f);
                    message.append_u64(word, schema, f);
                }
                schema.release_i32_local(selected, f);
                message.append_utf16(&code.0, schema, f);
            }
        }
        schema.release_i64_local(word, f);
        let bytes = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&bytes, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        match operation {
            DisplayNamesWireOperation::DisplayName => self.emit_intl_number_range_error(
                RuntimeErrorMessage::INVALID_INTL_DISPLAYNAMES_CODE,
                f,
            )?,
            DisplayNamesWireOperation::ResolveLocale
            | DisplayNamesWireOperation::SupportedLocales => {
                f.instruction(&Instruction::Unreachable);
            }
        };
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        bytes.clear(f);
        Ok(DisplayNamesProviderResponse {
            bytes: result,
            operation,
        })
    }
}
