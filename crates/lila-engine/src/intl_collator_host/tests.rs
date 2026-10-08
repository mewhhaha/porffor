use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{
    CanonicalLocaleId, CheckedCollatorConfiguration, CollatorSensitivity, CollatorUsage,
};
fn locale() -> CollatorLocaleRequest {
    CollatorLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("de").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        usage: CollatorUsage::Search,
        collation: None,
        numeric: None,
        case_first: None,
    }
}
fn gc_response<O: CollatorHostOperation>(probe: &mut IntlHostProbe, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(O::HOST_OP, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_collator_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let profiles = embedded_collator_profiles().unwrap();
    let request = locale();
    let response = gc_response::<ResolveCollatorLocale>(&mut probe, &request.encode().unwrap());
    let selected = ResolvedCollatorLocale::decode(&response, profiles, &request).unwrap();
    assert_eq!(selected.usage(), CollatorUsage::Search);
    let response = gc_response::<SupportedCollatorLocales>(
        &mut probe,
        &CollatorSupportedLocalesRequest {
            requested: request.requested,
            matcher: LocaleMatcher::Lookup,
        }
        .encode()
        .unwrap(),
    );
    assert_eq!(
        CollatorSupportedLocalesResult::decode(&response)
            .unwrap()
            .locales[0]
            .as_str(),
        "de"
    );
    let compare = CompareCollatorRequest::new(
        CheckedCollatorConfiguration::new(selected, CollatorSensitivity::Base, false),
        "AE".encode_utf16().collect(),
        "Ä".encode_utf16().collect(),
    )
    .unwrap();
    let response = gc_response::<CompareCollator>(&mut probe, &compare.encode().unwrap());
    assert_eq!(
        CollatorOrdering::decode(&response).unwrap(),
        CollatorOrdering::Equal
    );
}
#[test]
fn malformed_collator_requests_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let mut bytes = locale().encode().unwrap();
    bytes[0] = 2;
    for op in [
        IntlHostOp::ResolveCollatorLocale,
        IntlHostOp::SupportedCollatorLocales,
        IntlHostOp::CompareCollator,
    ] {
        assert!(probe.invoke(op, &bytes).is_err());
    }
}
