//! Exact mathematical NumberFormat kernel and pinned locale partitions.

mod configuration;
pub mod numeric;
pub mod options;
mod partition;
mod partition_resource;
mod parts;
mod plural_rules;
mod profiles;

pub use configuration::{
    filter_number_locales, resolve_number_locale, DecimalNumberingSystem,
    InvalidNumberingSystemOption, NumberFormatConfiguration, NumberLocaleRequest,
    NumberSupportedLocalesRequest, NumberingSystemOption, ResolvedNumberLocale,
};
pub use partition::{partition_number, partition_number_range};
pub use partition_resource::{
    NumberFormatKernelError, NumberPartitionResourceError, PartitionLimits,
};
pub use parts::{
    NumberPart, NumberPartKind, NumberRangePart, RangeNumberPartition, RangePartSource,
    ScalarNumberPartition,
};
pub use profiles::{
    embedded_number_profiles, CurrencyFractionRecord, CurrencyFractions, InvalidNumberProfile,
    NumberProfileError, NumberProfileTable, NumberProfiles, NUMBER_FORMAT_DATA_SHA256,
};

#[cfg(test)]
mod tests;
