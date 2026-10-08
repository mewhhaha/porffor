//! `temporal` intrinsic installation.
//!
//! All implemented Temporal families share members across entry and created Realms.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

mod members;
mod realm;
pub(crate) use members::TemporalIntrinsicFamily;
pub(crate) use realm::TemporalPrototypeSource;

/// The property key a Temporal member is installed under, derived
/// from the one place its name is already stated.
///
/// `native_function_name()` is what `Function.prototype.name` reports, so for a
/// plain method it *is* the key. An accessor's name carries the spec's
/// `get `/`set ` prefix (`get epochMilliseconds`), which the key does not, so
/// the prefix is stripped rather than the key being written out a second time —
/// a literal beside the id is a second spelling of a closed fact, and a typo in
/// it installs a correctly-named function under the wrong key with nothing in
/// the compiler noticing.
fn temporal_intrinsic_property_key(builtin: StandardBuiltinId) -> Result<&'static str, EmitError> {
    let name = builtin.native_function_name().ok_or_else(|| {
        EmitError::unsupported(format!(
            "unsupported in lila wasm-aot first slice: builtin `{}` has no native function name",
            builtin.debug_name()
        ))
    })?;
    Ok(name
        .strip_prefix("get ")
        .or_else(|| name.strip_prefix("set "))
        .unwrap_or(name))
}

impl FunctionBuilder<'_> {
    pub(crate) fn install_temporal_instant_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::Instant,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_plain_date_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::PlainDate,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_plain_date_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::PlainDateTime,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_duration_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::Duration,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_plain_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::PlainTime,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_zoned_date_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::ZonedDateTime,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_plain_year_month_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::PlainYearMonth,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }

    pub(crate) fn install_temporal_plain_month_day_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_temporal_intrinsic_members(
            TemporalIntrinsicFamily::PlainMonthDay,
            context.constructor,
            context.prototype,
            context.realm,
            function,
        )
    }
}
