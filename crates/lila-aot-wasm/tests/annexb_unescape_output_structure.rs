const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");

fn output_coordinator() -> &'static str {
    STRING_SOURCE
        .split_once("mod annexb_unescape_output {")
        .expect("Annex B unescape output module")
        .1
        .split_once("#[derive(Clone, Copy, PartialEq, Eq)]\npub(crate) enum UriCodecKind")
        .expect("Annex B unescape output module end")
        .0
}

fn unescape_decoder() -> &'static str {
    STRING_SOURCE
        .split_once("pub(crate) fn emit_annexb_unescape_string_payload(")
        .expect("Annex B unescape decoder")
        .1
        .split_once("pub(crate) fn emit_load_string_byte(")
        .expect("Annex B unescape decoder end")
        .0
}

#[test]
fn pending_lead_has_one_private_consuming_lifecycle() {
    let coordinator = output_coordinator();
    assert_eq!(
        coordinator
            .matches("pub(super) struct PendingLeadLocal(u32);")
            .count(),
        1
    );
    assert!(
        coordinator.contains("#[must_use = \"a pending Annex B unescape lead must be flushed\"]")
    );
    assert!(!coordinator.contains("derive("));
    assert!(!coordinator.contains("impl Copy for PendingLeadLocal"));
    assert_eq!(coordinator.matches("pub(super) fn reserve(").count(), 1);
    assert_eq!(
        coordinator
            .matches("pub(super) fn finish_into_payload(")
            .count(),
        1
    );
    assert!(coordinator.contains("pub(super) fn finish_into_payload(\n            self,"));

    let decoder = unescape_decoder();
    assert_eq!(
        decoder
            .matches("annexb_unescape_output::PendingLeadLocal::reserve(")
            .count(),
        1
    );
    assert_eq!(
        decoder.matches("pending_lead.finish_into_payload(").count(),
        1
    );
    assert!(!decoder.contains("emit_pack_string_payload("));
}

#[test]
fn every_decoded_or_raw_value_uses_the_output_coordinator() {
    let decoder = unescape_decoder();
    assert_eq!(decoder.matches("pending_lead.consume_scalar(").count(), 3);
    assert_eq!(
        decoder
            .matches("self.emit_decode_utf8_scalar_at_index(")
            .count(),
        1
    );
    assert!(!decoder.contains("emit_store_utf8_codepoint("));
    assert!(!decoder.contains("emit_store_byte_local("));
}

#[test]
fn coordinator_pairs_surrogates_and_flushes_lone_leads() {
    let coordinator = output_coordinator();
    let consume_unit = coordinator
        .split_once("fn consume_unit(")
        .expect("unit consumer")
        .1
        .split_once("pub(super) fn finish_into_payload(")
        .expect("unit consumer end")
        .0;
    let pair = consume_unit
        .find("builder.emit_is_low_surrogate_i32")
        .expect("trail-surrogate check");
    let pending_flush = consume_unit[pair..]
        .find("builder.emit_store_utf8_codepoint(dst_pos_local, self.0,")
        .expect("pending lead flush")
        + pair;
    let retain = consume_unit
        .find("builder.emit_is_high_surrogate_i32")
        .expect("lead-surrogate check");
    assert!(pair < pending_flush);
    assert!(pending_flush < retain);
    assert!(consume_unit.contains("Instruction::I64Const(0x10000)"));

    let finalizer = coordinator
        .split_once("pub(super) fn finish_into_payload(")
        .expect("output finalizer")
        .1;
    let final_flush = finalizer
        .find("builder.emit_store_utf8_codepoint(")
        .expect("final pending-lead flush");
    let output_len = finalizer
        .find("Instruction::LocalSet(output_len_local)")
        .expect("completed byte length");
    let pack = finalizer
        .find("builder.emit_pack_string_payload(")
        .expect("completed output pack");
    let release = finalizer
        .find("builder.release_temp_local(self.0);")
        .expect("pending local release");
    assert!(final_flush < output_len);
    assert!(output_len < pack);
    assert!(pack < release);
}
