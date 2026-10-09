//! Consolidated integration tests: Intl and locale-sensitive Date behavior.
//!
//! One binary per area bounds the number of linked Wasmtime test executables
//! while keeping per-process memory bounded.

mod aot_date_locale;
mod aot_date_system_time_zone;
mod aot_intl_bound_format_realm;
mod aot_intl_collator;
mod aot_intl_compilation_profile;
mod aot_intl_created_realm;
mod aot_intl_datetime_buddhist;
mod aot_intl_datetime_calendar16;
mod aot_intl_datetime_numbering;
mod aot_intl_datetime_provider;
mod aot_intl_datetime_range_endpoints;
mod aot_intl_display_names;
mod aot_intl_duration_format;
mod aot_intl_keyword_aliases;
mod aot_intl_list_format;
mod aot_intl_locale_constructor;
mod aot_intl_locale_host_import;
mod aot_intl_locale_hour_cycles;
mod aot_intl_locale_information_lists;
mod aot_intl_locale_likely_subtags;
mod aot_intl_locale_numbering_systems;
mod aot_intl_locale_options;
mod aot_intl_locale_text_info;
mod aot_intl_locale_week_info;
mod aot_intl_named_time_zones;
mod aot_intl_numberformat;
mod aot_intl_numbering_tols;
mod aot_intl_plural_rules;
mod aot_intl_relative_time_format;
mod aot_intl_segmenter;
mod aot_intl_supported_values;
