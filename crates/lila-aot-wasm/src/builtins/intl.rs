//! ECMA-402 `Intl` service layer — the smallest genuinely working slice.
//!
//! What is here: the `Intl` namespace object, `Intl.getCanonicalLocales`
//! (ECMA-402 8.2.1) and `Intl.Locale` (ECMA-402 14) with the `language`,
//! `script`, `region` and `baseName` getters plus `toString`.
//!
//! The observable shell rests on
//! [`FunctionBuilder::emit_intl_canonicalize_locale_tag`], which implements
//! *structural* `CanonicalizeUnicodeLocaleId`: it validates a string against
//! the `unicode_locale_id` grammar of UTS 35 and re-cases and re-orders it into
//! canonical form. `Intl.getCanonicalLocales` then sends that validated byte
//! span through the typed `lila_host.intl_call` boundary for pinned ICU4X alias
//! resolution before it performs list deduplication.
//!
//! The provider-backed alias pass is deliberately limited to
//! `Intl.getCanonicalLocales`; `Intl.Locale` still carries only the structural
//! result so its tag and component slots cannot disagree. General locale
//! matching, complete extension handling, and the other data-backed Intl
//! services remain open.
//!
//! One known deviation: the `Intl.Locale` constructor accepts but **ignores**
//! its `options` argument, so `new Intl.Locale("en", { region: "US" })` yields
//! `"en"` rather than `"en-US"`. Ignoring is the narrower wrong answer than
//! rejecting, because `undefined` and `{}` — the only options values the rest
//! of this slice can honour — then behave exactly as the spec requires.
//!
//! Strings are UTF-8 byte spans in linear memory and a well-formed language tag
//! is ASCII, so every structural pass below is a plain byte loop. That pass
//! preserves the input length; the later provider result has its own bounded
//! output buffer because alias replacement may change the length.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;
use crate::objects::TaggedLocals;
use lila_intl::{IntlHostCallOutcome, IntlHostOp, MAX_INTL_IDENTIFIER_BYTES};

/// Sort key used to force the `x-` private-use sequence after every other
/// extension sequence. Real singleton bytes are ASCII, so 0x100 sorts last.
const INTL_PRIVATE_USE_SORT_KEY: i64 = 0x100;

/// The original array-like value and its one observed length. Keeping these
/// together prevents the per-index walk from accidentally consuming a copied
/// element buffer or losing the source tag needed by object internal methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalLocaleListArrayLikeLocals {
    source: TaggedLocals,
    length: u32,
}

impl CanonicalLocaleListArrayLikeLocals {
    const fn new(source: TaggedLocals, length: u32) -> Self {
        Self { source, length }
    }

    const fn source(self) -> TaggedLocals {
        self.source
    }

    const fn length(self) -> u32 {
        self.length
    }
}

/// An allocated `Intl.Locale` result that is not yet branded or initialized.
///
/// The raw local is private and this state is deliberately non-`Copy`:
/// `Intl.Locale` must perform `OrdinaryCreateFromConstructor` before observing
/// its tag, but an abrupt tag/options completion must not publish that partial
/// object. Only `emit_initialize_intl_locale_object` can consume this state.
#[must_use]
struct ReservedIntlLocaleObjectLocal(u32);

/// An `Intl.Locale` result whose complete represented record and brand exist.
///
/// Only this state can cross the constructor's result boundary. Keeping the
/// raw local private makes publishing a reserved object a Rust type error.
#[must_use]
struct InitializedIntlLocaleObjectLocal(u32);

impl<'a> FunctionBuilder<'a> {
    fn intl_call_import_function_index(&self) -> Result<u32, EmitError> {
        self.functions
            .intl_call_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported("unsupported in lila wasm-aot: missing Intl host import")
            })
    }

    /// `dest = memory[slot_ptr + index * 8]`
    fn emit_intl_load_slot(
        &mut self,
        slot_ptr_local: u32,
        index_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        let address_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(slot_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.load_i64_to_local_from_offset(address_local, 0, dest_local, function);
        self.release_temp_local(address_local);
    }

    /// `memory[slot_ptr + index * 8] = value`
    fn emit_intl_store_slot(
        &mut self,
        slot_ptr_local: u32,
        index_local: u32,
        value_local: u32,
        function: &mut Function,
    ) {
        let address_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(slot_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.store_i64_local_at_offset(address_local, 0, value_local, function);
        self.release_temp_local(address_local);
    }

    fn emit_intl_store_byte(
        &self,
        base_local: u32,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    fn emit_intl_set_const(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(local));
    }

    /// Sets `all_alpha_local` and `all_digit_local` for the subtag spanning
    /// `[start, start + len)` of `buf_local`. Every byte in the buffer is
    /// already known to be lowercase ASCII alphanumeric, so "all alphanumeric"
    /// needs no test.
    fn emit_intl_subtag_kind(
        &mut self,
        buf_local: u32,
        start_local: u32,
        len_local: u32,
        all_alpha_local: u32,
        all_digit_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();

        self.emit_intl_set_const(all_alpha_local, 1, function);
        self.emit_intl_set_const(all_digit_local, 1, function);
        self.emit_intl_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.emit_load_string_byte(buf_local, address_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(all_alpha_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(all_alpha_local));

        function.instruction(&Instruction::LocalGet(all_digit_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(all_digit_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(address_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
    }

    /// `cmp_local` becomes 0 when the two subtags are equal, 1 when `a` sorts
    /// before `b` and 2 when it sorts after. Both spans are lowercase ASCII, so
    /// byte order is the required order.
    fn emit_intl_compare_subtags(
        &mut self,
        buf_local: u32,
        desc_a_local: u32,
        desc_b_local: u32,
        cmp_local: u32,
        function: &mut Function,
    ) {
        let start_a_local = self.reserve_temp_local();
        let len_a_local = self.reserve_temp_local();
        let start_b_local = self.reserve_temp_local();
        let len_b_local = self.reserve_temp_local();
        let min_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_a_local = self.reserve_temp_local();
        let byte_b_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(desc_a_local, start_a_local, len_a_local, function);
        self.emit_unpack_string_payload(desc_b_local, start_b_local, len_b_local, function);
        function.instruction(&Instruction::LocalGet(len_a_local));
        function.instruction(&Instruction::LocalGet(len_b_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(len_a_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_b_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(min_len_local));

        self.emit_intl_set_const(cmp_local, 0, function);
        self.emit_intl_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(min_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_a_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.emit_load_string_byte(buf_local, address_local, byte_a_local, function);
        function.instruction(&Instruction::LocalGet(start_b_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.emit_load_string_byte(buf_local, address_local, byte_b_local, function);
        function.instruction(&Instruction::LocalGet(byte_a_local));
        function.instruction(&Instruction::LocalGet(byte_b_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_a_local));
        function.instruction(&Instruction::LocalGet(byte_b_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cmp_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cmp_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_a_local));
        function.instruction(&Instruction::LocalGet(len_b_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(cmp_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_a_local));
        function.instruction(&Instruction::LocalGet(len_b_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(cmp_local, 2, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(address_local);
        self.release_temp_local(byte_b_local);
        self.release_temp_local(byte_a_local);
        self.release_temp_local(index_local);
        self.release_temp_local(min_len_local);
        self.release_temp_local(len_b_local);
        self.release_temp_local(start_b_local);
        self.release_temp_local(len_a_local);
        self.release_temp_local(start_a_local);
    }

    /// Appends the subtag described by `desc_local` to `out_local` at
    /// `pos_local`, advancing `pos_local`. `case_mode_local` is 0 to keep the
    /// stored lowercase bytes, 1 to uppercase every byte (region) and 2 to
    /// uppercase only the first byte (script).
    fn emit_intl_write_subtag(
        &mut self,
        buf_local: u32,
        out_local: u32,
        pos_local: u32,
        desc_local: u32,
        case_mode_local: u32,
        function: &mut Function,
    ) {
        let start_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let upper_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(desc_local, start_local, len_local, function);
        self.emit_intl_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        self.emit_load_string_byte(buf_local, address_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(case_mode_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(case_mode_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(upper_local));
        function.instruction(&Instruction::LocalGet(upper_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::End);

        self.emit_intl_store_byte(out_local, pos_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(upper_local);
        self.release_temp_local(address_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(start_local);
    }

    fn emit_intl_write_separator(
        &self,
        out_local: u32,
        pos_local: u32,
        scratch_byte_local: u32,
        function: &mut Function,
    ) {
        self.emit_intl_set_const(scratch_byte_local, b'-' as i64, function);
        self.emit_intl_store_byte(out_local, pos_local, scratch_byte_local, function);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
    }

    /// Structural `CanonicalizeUnicodeLocaleId` (ECMA-402 6.2.3 by way of
    /// UTS 35 `unicode_locale_id`).
    ///
    /// On success `ok_local` is 1 and the four payload outputs hold packed
    /// string payloads into a freshly allocated buffer; `script` and `region`
    /// are 0 when the tag carries no such subtag. On failure `ok_local` is 0
    /// and every output is 0 — the caller decides which error to raise.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_intl_canonicalize_locale_tag(
        &mut self,
        input_payload_local: u32,
        tag_payload_local: u32,
        language_payload_local: u32,
        script_payload_local: u32,
        region_payload_local: u32,
        base_name_payload_local: u32,
        ok_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let buf_local = self.reserve_temp_local();
        let desc_local = self.reserve_temp_local();
        let ext_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let alloc_size_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let scratch_local = self.reserve_temp_local();
        let segment_len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let other_entry_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let subtag_start_local = self.reserve_temp_local();
        let subtag_len_local = self.reserve_temp_local();
        let all_alpha_local = self.reserve_temp_local();
        let all_digit_local = self.reserve_temp_local();
        let language_desc_local = self.reserve_temp_local();
        let script_desc_local = self.reserve_temp_local();
        let region_desc_local = self.reserve_temp_local();
        let variant_start_local = self.reserve_temp_local();
        let variant_end_local = self.reserve_temp_local();
        let ext_count_local = self.reserve_temp_local();
        let singleton_mask_local = self.reserve_temp_local();
        let group_start_local = self.reserve_temp_local();
        let group_count_local = self.reserve_temp_local();
        let private_use_local = self.reserve_temp_local();
        let min_len_local = self.reserve_temp_local();
        let cmp_local = self.reserve_temp_local();
        let inner_index_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let case_mode_local = self.reserve_temp_local();
        let base_name_len_local = self.reserve_temp_local();
        let field_start_local = self.reserve_temp_local();

        for local in [
            tag_payload_local,
            language_payload_local,
            script_payload_local,
            region_payload_local,
            base_name_payload_local,
        ] {
            self.emit_intl_set_const(local, 0, function);
        }
        self.emit_intl_set_const(ok_local, 1, function);
        self.emit_unpack_string_payload(
            input_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::End);

        // Pass 1 — copy the tag while lowercasing it, reject any byte outside
        // `[0-9a-zA-Z-]`, and record one `(start << 32) | len` descriptor per
        // `-`-separated subtag.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(alloc_size_local));
        self.emit_heap_alloc_from_local(alloc_size_local, function)?;
        function.instruction(&Instruction::LocalSet(buf_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(alloc_size_local));
        self.emit_heap_alloc_from_local(alloc_size_local, function)?;
        function.instruction(&Instruction::LocalSet(desc_local));
        function.instruction(&Instruction::LocalGet(alloc_size_local));
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_heap_alloc_from_local(scratch_local, function)?;
        function.instruction(&Instruction::LocalSet(ext_local));

        self.emit_intl_set_const(count_local, 0, function);
        self.emit_intl_set_const(index_local, 0, function);
        self.emit_intl_set_const(start_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(segment_len_local));
        function.instruction(&Instruction::LocalGet(segment_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(segment_len_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_intl_store_slot(desc_local, count_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        self.emit_intl_store_byte(buf_local, index_local, byte_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(segment_len_local));
        function.instruction(&Instruction::LocalGet(segment_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(segment_len_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_intl_store_slot(desc_local, count_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Pass 2 — walk the descriptors against the `unicode_locale_id`
        // grammar, recording where each grammatical field sits.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(script_desc_local, 0, function);
        self.emit_intl_set_const(region_desc_local, 0, function);
        self.emit_intl_set_const(ext_count_local, 0, function);
        self.emit_intl_set_const(singleton_mask_local, 0, function);
        self.emit_intl_set_const(cursor_local, 0, function);

        self.emit_intl_load_slot(desc_local, cursor_local, language_desc_local, function);
        self.emit_unpack_string_payload(
            language_desc_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        self.emit_intl_subtag_kind(
            buf_local,
            subtag_start_local,
            subtag_len_local,
            all_alpha_local,
            all_digit_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(all_alpha_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::End);
        self.emit_intl_set_const(cursor_local, 1, function);

        // Optional script subtag: exactly four letters.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_load_slot(desc_local, cursor_local, entry_local, function);
        self.emit_unpack_string_payload(
            entry_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        self.emit_intl_subtag_kind(
            buf_local,
            subtag_start_local,
            subtag_len_local,
            all_alpha_local,
            all_digit_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(all_alpha_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::LocalSet(script_desc_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Optional region subtag: two letters or three digits.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_load_slot(desc_local, cursor_local, entry_local, function);
        self.emit_unpack_string_payload(
            entry_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        self.emit_intl_subtag_kind(
            buf_local,
            subtag_start_local,
            subtag_len_local,
            all_alpha_local,
            all_digit_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(all_alpha_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(all_digit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::LocalSet(region_desc_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Variants: `alphanum{5,8}` or `digit alphanum{3}`.
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(variant_start_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(desc_local, cursor_local, entry_local, function);
        self.emit_unpack_string_payload(
            entry_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        self.emit_load_string_byte(buf_local, subtag_start_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(variant_end_local));

        // Extension sequences: a singleton followed by at least one subtag.
        // `x` opens the private-use sequence, whose subtags may be one byte.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(desc_local, cursor_local, entry_local, function);
        self.emit_unpack_string_payload(
            entry_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(buf_local, subtag_start_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::LocalGet(singleton_mask_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(singleton_mask_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(singleton_mask_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'x' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(private_use_local));
        function.instruction(&Instruction::LocalGet(private_use_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(min_len_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(group_start_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_intl_set_const(group_count_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(desc_local, cursor_local, other_entry_local, function);
        self.emit_unpack_string_payload(
            other_entry_local,
            subtag_start_local,
            subtag_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::LocalGet(min_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(subtag_len_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::LocalGet(group_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(group_count_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(group_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(private_use_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(INTL_PRIVATE_USE_SORT_KEY));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(group_start_local));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(group_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_intl_store_slot(ext_local, ext_count_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(ext_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ext_count_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Pass 3 — order variants, reject duplicates, order extensions.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(variant_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(variant_end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(desc_local, index_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::LocalGet(variant_start_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_intl_load_slot(desc_local, scratch_local, other_entry_local, function);
        self.emit_intl_compare_subtags(
            buf_local,
            other_entry_local,
            entry_local,
            cmp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cmp_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_store_slot(desc_local, inner_index_local, other_entry_local, function);
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_intl_store_slot(desc_local, inner_index_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(variant_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(variant_end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_intl_load_slot(desc_local, index_local, entry_local, function);
        self.emit_intl_load_slot(desc_local, scratch_local, other_entry_local, function);
        self.emit_intl_compare_subtags(
            buf_local,
            other_entry_local,
            entry_local,
            cmp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cmp_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_intl_set_const(index_local, 1, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(ext_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(ext_local, index_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_intl_load_slot(ext_local, scratch_local, other_entry_local, function);
        function.instruction(&Instruction::LocalGet(other_entry_local));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_store_slot(ext_local, inner_index_local, other_entry_local, function);
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_intl_store_slot(ext_local, inner_index_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Pass 4 — render the canonical form. Canonicalisation is
        // length-preserving, so the output buffer is exactly the input length.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(alloc_size_local));
        self.emit_heap_alloc_from_local(alloc_size_local, function)?;
        function.instruction(&Instruction::LocalSet(out_local));
        self.emit_intl_set_const(pos_local, 0, function);

        self.emit_intl_set_const(case_mode_local, 0, function);
        self.emit_intl_write_subtag(
            buf_local,
            out_local,
            pos_local,
            language_desc_local,
            case_mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(language_payload_local));

        function.instruction(&Instruction::LocalGet(script_desc_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_write_separator(out_local, pos_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::LocalSet(field_start_local));
        self.emit_intl_set_const(case_mode_local, 2, function);
        self.emit_intl_write_subtag(
            buf_local,
            out_local,
            pos_local,
            script_desc_local,
            case_mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(field_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::LocalGet(field_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(script_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(region_desc_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_write_separator(out_local, pos_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::LocalSet(field_start_local));
        self.emit_intl_set_const(case_mode_local, 1, function);
        self.emit_intl_write_subtag(
            buf_local,
            out_local,
            pos_local,
            region_desc_local,
            case_mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(field_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::LocalGet(field_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(region_payload_local));
        function.instruction(&Instruction::End);

        self.emit_intl_set_const(case_mode_local, 0, function);
        function.instruction(&Instruction::LocalGet(variant_start_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(variant_end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_write_separator(out_local, pos_local, byte_local, function);
        self.emit_intl_load_slot(desc_local, index_local, entry_local, function);
        self.emit_intl_write_subtag(
            buf_local,
            out_local,
            pos_local,
            entry_local,
            case_mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::LocalSet(base_name_len_local));

        self.emit_intl_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(ext_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_load_slot(ext_local, index_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xFF_FFFF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(group_start_local));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(0xFF_FFFF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(group_count_local));
        function.instruction(&Instruction::LocalGet(group_start_local));
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::LocalGet(group_start_local));
        function.instruction(&Instruction::LocalGet(group_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_write_separator(out_local, pos_local, byte_local, function);
        self.emit_intl_load_slot(desc_local, inner_index_local, other_entry_local, function);
        self.emit_intl_write_subtag(
            buf_local,
            out_local,
            pos_local,
            other_entry_local,
            case_mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(base_name_len_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(base_name_payload_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(field_start_local);
        self.release_temp_local(base_name_len_local);
        self.release_temp_local(case_mode_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(out_local);
        self.release_temp_local(inner_index_local);
        self.release_temp_local(cmp_local);
        self.release_temp_local(min_len_local);
        self.release_temp_local(private_use_local);
        self.release_temp_local(group_count_local);
        self.release_temp_local(group_start_local);
        self.release_temp_local(singleton_mask_local);
        self.release_temp_local(ext_count_local);
        self.release_temp_local(variant_end_local);
        self.release_temp_local(variant_start_local);
        self.release_temp_local(region_desc_local);
        self.release_temp_local(script_desc_local);
        self.release_temp_local(language_desc_local);
        self.release_temp_local(all_digit_local);
        self.release_temp_local(all_alpha_local);
        self.release_temp_local(subtag_len_local);
        self.release_temp_local(subtag_start_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(other_entry_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(segment_len_local);
        self.release_temp_local(scratch_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(start_local);
        self.release_temp_local(index_local);
        self.release_temp_local(alloc_size_local);
        self.release_temp_local(count_local);
        self.release_temp_local(ext_local);
        self.release_temp_local(desc_local);
        self.release_temp_local(buf_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    /// Loads the `Intl.Locale` internal record of `receiver`, throwing a
    /// TypeError and returning when the receiver does not carry one.
    fn emit_intl_locale_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_intl_set_const(brand_local, 0, function);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_INTL_LOCALE as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Intl.Locale.prototype method called on incompatible receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    /// Coerces one `Intl` locale argument to the string that must be
    /// canonicalized: an `Intl.Locale` contributes its `[[Locale]]` slot, a
    /// String contributes itself, any other object is `ToString`-ed, and every
    /// other value is a TypeError.
    pub(crate) fn emit_intl_locale_argument_to_string_payload(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        out_payload_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();

        self.emit_intl_set_const(handled_local, 0, function);
        self.emit_intl_set_const(out_payload_local, 0, function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_INTL_LOCALE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            brand_local,
            HEAP_INTL_LOCALE_TAG_OFFSET,
            out_payload_local,
            function,
        );
        self.emit_intl_set_const(handled_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
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
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(out_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(handled_local);
        self.release_temp_local(brand_local);
        Ok(())
    }

    /// ECMA-402 `Intl.Locale` step 6: resolve `NewTarget.prototype` and reserve
    /// the result object before the first observable tag or options operation.
    fn emit_reserve_intl_locale_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlLocaleObjectLocal, EmitError> {
        // The retained object local is reserved first. Prototype locals can
        // then be released in strict LIFO order while the lifecycle keeps the
        // object live across tag/options work.
        let object_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype = TaggedLocals::new(prototype_payload_local, prototype_tag_local);
        let result = (|| {
            self.emit_new_target_prototype_to_locals(
                INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX,
                NewTargetPrototypeFallback::CurrentGlobal,
                prototype.payload,
                prototype.tag,
                function,
            )?;
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(prototype.payload),
                Some(prototype.tag),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(object_payload_local));
            Ok(())
        })();
        self.release_temp_local(prototype.tag);
        self.release_temp_local(prototype.payload);
        if let Err(error) = result {
            self.release_temp_local(object_payload_local);
            return Err(error);
        }
        Ok(ReservedIntlLocaleObjectLocal(object_payload_local))
    }

    /// Consume the unreachable reserved result and install every Locale slot
    /// represented by the current backend before making it publishable.
    #[allow(clippy::too_many_arguments)]
    fn emit_initialize_intl_locale_object(
        &mut self,
        reserved: ReservedIntlLocaleObjectLocal,
        tag_payload_local: u32,
        language_payload_local: u32,
        script_payload_local: u32,
        region_payload_local: u32,
        base_name_payload_local: u32,
        function: &mut Function,
    ) -> Result<InitializedIntlLocaleObjectLocal, EmitError> {
        let object_payload_local = reserved.0;
        let record_local = self.reserve_temp_local();
        if let Err(error) = self.emit_heap_alloc_const(HEAP_INTL_LOCALE_RECORD_SIZE, function) {
            self.release_temp_local(record_local);
            self.release_temp_local(object_payload_local);
            return Err(error);
        }
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, value_local) in [
            (HEAP_INTL_LOCALE_TAG_OFFSET, tag_payload_local),
            (HEAP_INTL_LOCALE_LANGUAGE_OFFSET, language_payload_local),
            (HEAP_INTL_LOCALE_SCRIPT_OFFSET, script_payload_local),
            (HEAP_INTL_LOCALE_REGION_OFFSET, region_payload_local),
            (HEAP_INTL_LOCALE_BASE_NAME_OFFSET, base_name_payload_local),
        ] {
            self.store_i64_local_at_offset(record_local, offset, value_local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_LOCALE,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(InitializedIntlLocaleObjectLocal(object_payload_local))
    }

    /// Publish the only `Intl.Locale` lifecycle state allowed to escape.
    fn emit_publish_intl_locale_object(
        &mut self,
        initialized: InitializedIntlLocaleObjectLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(initialized.0);
    }

    pub(crate) fn emit_intl_locale_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let tag_payload_local = self.reserve_temp_local();
        let language_payload_local = self.reserve_temp_local();
        let script_payload_local = self.reserve_temp_local();
        let region_payload_local = self.reserve_temp_local();
        let base_name_payload_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Intl.Locale constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `OrdinaryCreateFromConstructor` precedes the tag type check and
        // `ToString`. The reserved object cannot escape if either tag work or
        // the future ordered options pass completes abruptly.
        let reserved_object = self.emit_reserve_intl_locale_object(function)?;

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_intl_locale_argument_to_string_payload(
            argument_payload_local,
            argument_tag_local,
            input_payload_local,
            "Intl.Locale tag must be a string or an object",
            function,
        )?;
        self.emit_intl_canonicalize_locale_tag(
            input_payload_local,
            tag_payload_local,
            language_payload_local,
            script_payload_local,
            region_payload_local,
            base_name_payload_local,
            ok_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let initialized_object = self.emit_initialize_intl_locale_object(
            reserved_object,
            tag_payload_local,
            language_payload_local,
            script_payload_local,
            region_payload_local,
            base_name_payload_local,
            function,
        )?;
        self.emit_publish_intl_locale_object(initialized_object, function);

        self.release_temp_local(ok_local);
        self.release_temp_local(base_name_payload_local);
        self.release_temp_local(region_payload_local);
        self.release_temp_local(script_payload_local);
        self.release_temp_local(language_payload_local);
        self.release_temp_local(tag_payload_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    /// Emits one of the `Intl.Locale.prototype` string slots. `optional` marks
    /// the slots that are `undefined` when the tag omits the subtag.
    pub(crate) fn emit_intl_locale_string_slot(
        &mut self,
        slot_offset: u64,
        optional: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();

        self.emit_intl_locale_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(record_local, slot_offset, value_local, function);
        if optional {
            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_set_const(self.result_local, 0, function);
            self.emit_intl_set_const(
                self.result_tag_local,
                ValueKind::Undefined.tag() as i64,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            self.emit_intl_set_const(
                self.result_tag_local,
                ValueKind::String.tag() as i64,
                function,
            );
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            self.emit_intl_set_const(
                self.result_tag_local,
                ValueKind::String.tag() as i64,
                function,
            );
        }

        self.release_temp_local(value_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_intl_get_canonical_locales(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let single_payload_local = self.reserve_temp_local();
        let has_single_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let source_key_local = self.reserve_temp_local();
        let source_length_payload_local = self.reserve_temp_local();
        let source_length_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_buffer_local = self.reserve_temp_local();
        let result_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let property_present_local = self.reserve_temp_local();
        let inner_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let tag_payload_local = self.reserve_temp_local();
        let language_payload_local = self.reserve_temp_local();
        let script_payload_local = self.reserve_temp_local();
        let region_payload_local = self.reserve_temp_local();
        let base_name_payload_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();
        let provider_output_local = self.reserve_temp_local();
        let provider_output_len_local = self.reserve_temp_local();
        let provider_output_capacity_local = self.reserve_temp_local();
        let duplicate_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let existing_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let result_prototype_payload_local = self.reserve_temp_local();
        let result_prototype_tag_local = self.reserve_temp_local();
        let array_like = CanonicalLocaleListArrayLikeLocals::new(
            TaggedLocals::new(source_payload_local, source_tag_local),
            source_len_local,
        );
        let result_prototype =
            TaggedLocals::new(result_prototype_payload_local, result_prototype_tag_local);

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_intl_set_const(has_single_local, 0, function);
        self.emit_intl_set_const(single_payload_local, 0, function);
        self.emit_intl_set_const(array_like.length(), 0, function);
        self.emit_intl_set_const(array_like.source().payload, 0, function);
        self.emit_intl_set_const(
            array_like.source().tag,
            ValueKind::Undefined.tag() as i64,
            function,
        );

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(single_payload_local));
        self.emit_intl_set_const(has_single_local, 1, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_INTL_LOCALE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            brand_local,
            HEAP_INTL_LOCALE_TAG_OFFSET,
            single_payload_local,
            function,
        );
        self.emit_intl_set_const(has_single_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // `undefined` is the empty list and a String or an `Intl.Locale` is a
        // one-element list. Every other value goes through `ToObject` in this
        // builtin's defining Realm, then observes `length` exactly once while
        // retaining that original object for the indexed HasProperty/Get walk
        // below.
        function.instruction(&Instruction::LocalGet(has_single_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            argument_payload_local,
            argument_tag_local,
            array_like.source().payload,
            array_like.source().tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(source_key_local));
        self.emit_object_read(
            array_like.source().payload,
            array_like.source().tag,
            array_like.source().payload,
            array_like.source().tag,
            source_key_local,
            source_length_payload_local,
            source_length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            source_length_tag_local,
            source_length_payload_local,
            array_like.length(),
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_single_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(array_like.length(), 1, function);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(
            array_like.length(),
            result_payload_local,
            function,
        )?;
        self.emit_intl_set_const(function_realm_local, 0, function);
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
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            result_prototype.payload,
            function,
        );
        self.emit_intl_set_const(
            result_prototype.tag,
            ValueKind::Array.tag() as i64,
            function,
        );
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            result_prototype.payload,
            function,
        );
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            result_prototype.tag,
            function,
        );
        self.load_i64_to_local_from_offset(
            result_payload_local,
            HEAP_PTR_OFFSET,
            result_buffer_local,
            function,
        );
        self.emit_intl_set_const(result_len_local, 0, function);
        self.emit_intl_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(array_like.length()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_set_const(property_present_local, 1, function);
        function.instruction(&Instruction::LocalGet(has_single_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            source_key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            array_like.source().payload,
            array_like.source().tag,
            source_key_local,
            property_present_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(property_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            array_like.source().payload,
            array_like.source().tag,
            array_like.source().payload,
            array_like.source().tag,
            source_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_intl_locale_argument_to_string_payload(
            element_payload_local,
            element_tag_local,
            input_payload_local,
            "Intl.getCanonicalLocales locale must be a string or an object",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(single_payload_local));
        function.instruction(&Instruction::LocalSet(input_payload_local));
        function.instruction(&Instruction::End);

        // A missing property skips Get, coercion, provider work and
        // deduplication. The index advances only after all work for a present
        // property completes, so mutations remain observable in spec order.
        function.instruction(&Instruction::LocalGet(property_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_canonicalize_locale_tag(
            input_payload_local,
            tag_payload_local,
            language_payload_local,
            script_payload_local,
            region_payload_local,
            base_name_payload_local,
            ok_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Structural validation and every observable element read/coercion are
        // complete before the pure provider call. Reserve a fresh maximum-size
        // buffer for each candidate because a retained result string may
        // outlive this iteration and CLDR aliases can change the tag length.
        self.emit_intl_set_const(
            provider_output_capacity_local,
            MAX_INTL_IDENTIFIER_BYTES as i64,
            function,
        );
        self.emit_heap_alloc_from_local(provider_output_capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(provider_output_local));
        function.instruction(&Instruction::I64Const(
            IntlHostOp::CanonicalizeLocale.wire(),
        ));
        function.instruction(&Instruction::LocalGet(tag_payload_local));
        self.emit_pack_string_payload(
            provider_output_local,
            provider_output_capacity_local,
            function,
        );
        function.instruction(&Instruction::Call(self.intl_call_import_function_index()?));
        function.instruction(&Instruction::LocalSet(provider_output_len_local));

        function.instruction(&Instruction::LocalGet(provider_output_len_local));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Any negative value other than `Rejected`, or a length beyond the
        // supplied capacity, is a host ABI fault rather than a JavaScript
        // RangeError. The closed Rust outcome type prevents Lila's engine from
        // producing one; `unreachable` rejects a non-conforming embedder.
        function.instruction(&Instruction::LocalGet(provider_output_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(provider_output_len_local));
        function.instruction(&Instruction::LocalGet(provider_output_capacity_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(provider_output_local, provider_output_len_local, function);
        function.instruction(&Instruction::LocalSet(tag_payload_local));

        self.emit_intl_set_const(duplicate_local, 0, function);
        self.emit_intl_set_const(inner_index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(result_buffer_local));
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            existing_local,
            function,
        );
        self.emit_string_payload_equality_i32(existing_local, tag_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_set_const(duplicate_local, 1, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(inner_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(inner_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(duplicate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_buffer_local));
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            ValueKind::String.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            tag_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Duplicates are dropped in place, so the array is published with the
        // number of entries actually written; the surplus capacity is unused.
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_LEN_OFFSET,
            result_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(result_prototype_tag_local);
        self.release_temp_local(result_prototype_payload_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(existing_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(duplicate_local);
        self.release_temp_local(provider_output_capacity_local);
        self.release_temp_local(provider_output_len_local);
        self.release_temp_local(provider_output_local);
        self.release_temp_local(ok_local);
        self.release_temp_local(base_name_payload_local);
        self.release_temp_local(region_payload_local);
        self.release_temp_local(script_payload_local);
        self.release_temp_local(language_payload_local);
        self.release_temp_local(tag_payload_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(inner_index_local);
        self.release_temp_local(property_present_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_len_local);
        self.release_temp_local(result_buffer_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(source_length_tag_local);
        self.release_temp_local(source_length_payload_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(source_len_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(has_single_local);
        self.release_temp_local(single_payload_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }
}

#[cfg(test)]
mod intl_locale_construction_order_tests {
    #[test]
    fn reserved_locale_lifecycle_preserves_order_and_prototype_tag() {
        let source = include_str!("intl.rs");
        let reserve = source
            .split_once("fn emit_reserve_intl_locale_object(")
            .expect("Locale reserve transition should exist")
            .1
            .split_once("/// Consume the unreachable reserved result")
            .expect("Locale reserve transition should be bounded")
            .0;
        let constructor = source
            .split_once("pub(crate) fn emit_intl_locale_constructor(")
            .expect("Locale constructor should exist")
            .1
            .split_once("/// Emits one of the `Intl.Locale.prototype` string slots")
            .expect("Locale constructor should be bounded")
            .0;

        let object_reserve = reserve
            .find("let object_payload_local = self.reserve_temp_local();")
            .expect("the retained object local should be reserved");
        let prototype_reserve = reserve
            .find("let prototype_payload_local = self.reserve_temp_local();")
            .expect("the prototype payload local should be reserved");
        let prototype_tag_reserve = reserve
            .find("let prototype_tag_local = self.reserve_temp_local();")
            .expect("the prototype tag local should be reserved");
        let prototype_pair = reserve
            .find(
                "let prototype = TaggedLocals::new(prototype_payload_local, prototype_tag_local);",
            )
            .expect("the complete tagged prototype should be formed");
        assert!(object_reserve < prototype_reserve);
        assert!(prototype_reserve < prototype_tag_reserve);
        assert!(prototype_tag_reserve < prototype_pair);
        assert_eq!(
            reserve
                .matches("emit_new_target_prototype_to_locals(")
                .count(),
            1
        );
        assert_eq!(
            reserve
                .matches("NewTargetPrototypeFallback::CurrentGlobal")
                .count(),
            1
        );
        assert_eq!(
            reserve
                .matches("emit_alloc_plain_object_with_prototype_and_tag(")
                .count(),
            1
        );
        assert!(reserve.contains("Some(prototype.tag),"));
        let prototype_tag_release = reserve
            .find("self.release_temp_local(prototype.tag);")
            .expect("the prototype tag local should be released");
        let prototype_payload_release = reserve
            .find("self.release_temp_local(prototype.payload);")
            .expect("the prototype payload local should be released");
        let retained_object_error_release = reserve
            .find("self.release_temp_local(object_payload_local);")
            .expect("an emitter error should release the retained object local");
        assert!(prototype_tag_release < prototype_payload_release);
        assert!(prototype_payload_release < retained_object_error_release);

        let reserve_call = constructor
            .find("let reserved_object = self.emit_reserve_intl_locale_object(function)?;")
            .expect("the constructor should reserve the result");
        let tag_observation = constructor
            .find("self.emit_intl_locale_argument_to_string_payload(")
            .expect("the constructor should observe the tag");
        let initialize_call = constructor
            .find("let initialized_object = self.emit_initialize_intl_locale_object(")
            .expect("the constructor should initialize the reserved result");
        let publish_call = constructor
            .find("self.emit_publish_intl_locale_object(initialized_object, function);")
            .expect("the constructor should publish the initialized result");
        assert!(reserve_call < tag_observation);
        assert!(tag_observation < initialize_call);
        assert!(initialize_call < publish_call);
    }
}
