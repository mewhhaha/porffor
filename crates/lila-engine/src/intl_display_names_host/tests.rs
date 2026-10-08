use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::*;

fn locale() -> DisplayNamesLocaleRequest {
    DisplayNamesLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn configuration() -> CheckedDisplayNamesConfiguration {
    let provider = EmbeddedIntlProvider::new().unwrap();
    let resolved =
        IntlOperationProvider::<ResolveDisplayNamesLocale>::execute(&provider, locale()).unwrap();
    CheckedDisplayNamesConfiguration::new(
        resolved,
        DisplayNamesSelection::Region,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::None,
    )
}
fn gc_response(probe: &mut IntlHostProbe, operation: IntlHostOp, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(operation, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_display_names_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let provider = EmbeddedIntlProvider::new().unwrap();
    let profiles = DisplayNamesProfiles::from_pinned_data(&provider).unwrap();
    let response = gc_response(
        &mut probe,
        IntlHostOp::ResolveDisplayNamesLocale,
        &encode_display_names_locale_request(&locale()).unwrap(),
    );
    assert_eq!(
        decode_display_names_locale_response(&response, &profiles)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    let response = gc_response(
        &mut probe,
        IntlHostOp::SupportedDisplayNamesLocales,
        &encode_display_names_supported_request(&locale()).unwrap(),
    );
    assert_eq!(
        decode_display_names_supported_response(&response)
            .unwrap()
            .locales[0]
            .as_str(),
        "en-US"
    );
    let request = DisplayNameRequest::new(configuration(), "US".encode_utf16().collect()).unwrap();
    let response = gc_response(
        &mut probe,
        IntlHostOp::DisplayName,
        &encode_display_name_request(&request).unwrap(),
    );
    assert_eq!(
        decode_display_name_response(&response).unwrap().name(),
        Some("United States")
    );
}

#[test]
fn invalid_display_name_code_is_rejected_as_a_null_gc_response() {
    let mut probe = IntlHostProbe::new();
    // The wire preserves this isolated surrogate; native code validation owns rejection.
    let request =
        DisplayNameRequest::new(configuration(), vec![0xd800].into_boxed_slice()).unwrap();
    let bytes = encode_display_name_request(&request).unwrap();

    assert_eq!(probe.invoke(IntlHostOp::DisplayName, &bytes).unwrap(), None);
}

#[test]
fn malformed_display_names_frames_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let mut bytes = encode_display_names_locale_request(&locale()).unwrap();
    bytes[0] = 2;
    for operation in [
        IntlHostOp::ResolveDisplayNamesLocale,
        IntlHostOp::SupportedDisplayNamesLocales,
        IntlHostOp::DisplayName,
    ] {
        assert!(probe.invoke(operation, &bytes).is_err());
    }
}
