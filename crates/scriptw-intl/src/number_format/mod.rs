//! Exact mathematical NumberFormat kernel and pinned locale partitions.

mod configuration;
pub mod numeric;
pub mod options;
mod partition;
mod partition_resource;
mod parts;
mod plural_rules;
mod profiles;

pub(crate) use configuration::resolve_number_locale_in;
pub use configuration::{
    filter_number_locales, resolve_number_locale, DecimalNumberingSystem,
    InvalidNumberingSystemOption, NumberFormatConfiguration, NumberLocaleRequest,
    NumberSupportedLocalesRequest, NumberingSystemOption, ResolvedNumberLocale,
};
pub(crate) use configuration::{locale_default_numbering_system, matching_locale};
pub use partition::{partition_number, partition_number_range};
pub(crate) use partition_resource::owned_text;
pub use partition_resource::{
    NumberFormatKernelError, NumberPartitionResourceError, PartitionLimits,
};
pub use parts::{
    NumberPart, NumberPartKind, NumberRangePart, RangeNumberPartition, RangePartSource,
    ScalarNumberPartition,
};
pub(crate) use profiles::NUMBERING_SUPPLEMENT_PROVENANCE;
pub use profiles::{
    embedded_number_profiles, embedded_number_profiles_arc, CurrencyFractionRecord,
    CurrencyFractions, InvalidNumberProfile, NumberProfileError, NumberProfileTable,
    NumberProfiles, NUMBER_FORMAT_DATA_SHA256,
};
pub(crate) use profiles::{
    encode_full, encode_selected, encode_selected_currencies, encode_selected_numbering,
};
pub(crate) use profiles::{NumberLocaleDomains, NumberLocaleView};

#[cfg(test)]
mod tests;
