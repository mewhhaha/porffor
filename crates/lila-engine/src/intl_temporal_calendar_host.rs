use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{QueryTemporalCalendar, TemporalCalendarRequest};

/// Temporal calendar queries use a fixed, validated primitive snapshot. The
/// AOT caller performs JavaScript property access and conversion before this
/// pure host operation is invoked.
pub(super) fn query_temporal_calendar(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = TemporalCalendarRequest::decode(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!("invalid Temporal calendar query: {error}"))
    })?;
    let handle = kernel
        .operation::<QueryTemporalCalendar>()
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "Temporal calendar kernel capability mismatch: {error}"
            ))
        })?;
    let answer = handle.execute(request).map_err(|error| {
        wasmtime::Error::msg(format!("Temporal calendar query failed: {error}"))
    })?;
    copied.write(&mut caller, &answer.encode())
}
