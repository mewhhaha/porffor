use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{CanonicalLocaleId, CheckedListConfiguration, ListPart, ListStyle, ListType};
fn locale() -> ListLocaleRequest {
    ListLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn gc_response<O: ListHostOperation>(probe: &mut IntlHostProbe, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(O::HOST_OP, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_list_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let profiles = embedded_list_profiles().unwrap();
    let response = gc_response::<ResolveListLocale>(&mut probe, &locale().encode().unwrap());
    let resolved = ResolvedListLocale::decode(&response, profiles).unwrap();
    assert_eq!(resolved.resolved().as_str(), "en-US");
    let request = ListSupportedLocalesRequest {
        requested: locale().requested,
        matcher: LocaleMatcher::Lookup,
    };
    let response = gc_response::<SupportedListLocales>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        ListSupportedLocalesResult::decode(&response)
            .unwrap()
            .locales[0]
            .as_str(),
        "en-US"
    );
    let request = FormatListPartsRequest::new(
        CheckedListConfiguration::new(resolved, ListType::Conjunction, ListStyle::Long),
        vec![vec![0xd800].into_boxed_slice(), Box::new([])].into_boxed_slice(),
    )
    .unwrap();
    let response = gc_response::<FormatListParts>(&mut probe, &request.encode().unwrap());
    assert_eq!(
        ListParts::decode(&response, 2).unwrap().parts(),
        [
            ListPart::Element(0),
            ListPart::Literal(" and ".encode_utf16().collect()),
            ListPart::Element(1)
        ]
    );
}
#[test]
fn malformed_list_requests_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let mut bytes = locale().encode().unwrap();
    bytes[0] = 2;
    for op in [
        IntlHostOp::ResolveListLocale,
        IntlHostOp::SupportedListLocales,
        IntlHostOp::FormatListParts,
    ] {
        assert!(probe.invoke(op, &bytes).is_err());
    }
}
