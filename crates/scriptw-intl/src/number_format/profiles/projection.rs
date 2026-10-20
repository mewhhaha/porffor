//! Physical Number/Plural data selection from the sole admitted pinned owner.
use super::*;
use crate::number_image::PinnedNumberSource;

mod closure;
mod encode;
#[cfg(test)]
mod tests;

pub(crate) fn encode_full(source: &PinnedNumberSource) -> Result<Vec<u8>, InvalidNumberProfile> {
    let profiles = source.profiles();
    let maps = closure::TableMaps::full(profiles)?;
    let mut locales = Vec::new();
    locales
        .try_reserve_exact(profiles.locales.len())
        .map_err(|_| {
            error(
                NumberProfileTable::Locales,
                0,
                NumberProfileError::Allocation,
            )
        })?;
    locales.extend(0..profiles.locales.len());
    encode::encode(profiles, &maps, &locales, None, None)
}

pub(crate) fn encode_selected(
    source: &PinnedNumberSource,
    sorted_physical_locales: &[Box<str>],
) -> Result<Vec<u8>, InvalidNumberProfile> {
    encode_selection(source, sorted_physical_locales, None, None)
}

pub(crate) fn encode_selected_currencies(
    source: &PinnedNumberSource,
    sorted_physical_locales: &[Box<str>],
    currency_codes: &[CurrencyCode],
) -> Result<Vec<u8>, InvalidNumberProfile> {
    if currency_codes.is_empty()
        || currency_codes
            .windows(2)
            .any(|pair| pair[0].clone().ascii() >= pair[1].clone().ascii())
        || currency_codes
            .iter()
            .any(|code| !source.profiles().has_currency_data("en-US", code))
    {
        return Err(error(
            NumberProfileTable::CurrencySets,
            0,
            NumberProfileError::Order,
        ));
    }
    encode_selection(source, sorted_physical_locales, Some(currency_codes), None)
}

pub(crate) fn encode_selected_numbering(
    source: &PinnedNumberSource,
    sorted_physical_locales: &[Box<str>],
    currency_codes: Option<&[CurrencyCode]>,
    numbering_systems: &[crate::number_format::NumberingSystemOption],
) -> Result<Vec<u8>, InvalidNumberProfile> {
    if numbering_systems.is_empty()
        || numbering_systems
            .windows(2)
            .any(|pair| pair[0].name() >= pair[1].name())
        || numbering_systems
            .iter()
            .any(|value| source.profiles().system(value.name()).is_none())
    {
        return Err(error(
            NumberProfileTable::NumberingSystems,
            0,
            NumberProfileError::Order,
        ));
    }
    encode_selection(
        source,
        sorted_physical_locales,
        currency_codes,
        Some(numbering_systems),
    )
}

fn encode_selection(
    source: &PinnedNumberSource,
    sorted_physical_locales: &[Box<str>],
    currency_codes: Option<&[CurrencyCode]>,
    numbering_systems: Option<&[crate::number_format::NumberingSystemOption]>,
) -> Result<Vec<u8>, InvalidNumberProfile> {
    let profiles = source.profiles();
    if sorted_physical_locales.is_empty()
        || sorted_physical_locales
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(error(
            NumberProfileTable::Locales,
            0,
            NumberProfileError::Order,
        ));
    }
    if !sorted_physical_locales
        .iter()
        .any(|name| name.as_ref() == "en-US")
    {
        return Err(error(
            NumberProfileTable::Locales,
            0,
            NumberProfileError::MissingDefaultLocale,
        ));
    }
    let mut locales = Vec::new();
    locales
        .try_reserve_exact(sorted_physical_locales.len())
        .map_err(|_| {
            error(
                NumberProfileTable::Locales,
                0,
                NumberProfileError::Allocation,
            )
        })?;
    for (index, name) in sorted_physical_locales.iter().enumerate() {
        let row = profiles
            .locales
            .binary_search_by(|candidate| candidate.as_ref().cmp(name.as_ref()))
            .map_err(|_| {
                error(
                    NumberProfileTable::Locales,
                    index,
                    NumberProfileError::InvalidLocale,
                )
            })?;
        locales.push(row);
    }
    let maps = closure::TableMaps::selected(profiles, &locales, currency_codes, numbering_systems)?;
    encode::encode(profiles, &maps, &locales, currency_codes, numbering_systems)
}

fn numbering_selected(
    source: &NumberProfiles,
    profile: &LocaleProfile,
    index: usize,
    systems: Option<&[crate::number_format::NumberingSystemOption]>,
) -> bool {
    systems.is_none_or(|systems| {
        index == usize::from(profile.default_numbering)
            || systems
                .binary_search_by(|value| value.name().cmp(source.system_names[index].as_ref()))
                .is_ok()
    })
}

fn currency_selected(codes: Option<&[CurrencyCode]>, code: [u8; 3]) -> bool {
    codes.is_none_or(|codes| {
        codes
            .binary_search_by_key(&code, |candidate| candidate.clone().ascii())
            .is_ok()
    })
}

fn error(
    table: NumberProfileTable,
    index: usize,
    reason: NumberProfileError,
) -> InvalidNumberProfile {
    InvalidNumberProfile {
        table,
        index: u32::try_from(index).unwrap_or(u32::MAX),
        reason,
    }
}
