use super::*;
use crate::provider::LocaleCanonicalizationData;
use crate::{LocaleId, LocaleTransformError, LocaleTransformRequest};
use code::LanguageCode;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[path = "profile_identity.rs"]
mod identity;
pub use identity::DISPLAY_NAMES_DATA_SHA256;
pub const DISPLAY_NAMES_PROFILE: &[u8] = include_bytes!("profile.json");
const CLDR_COMMIT: &str = "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c";
const LOCALES: &[&str] = &[
    "ar",
    "ar-EG",
    "de",
    "en",
    "en-US",
    "fr",
    "hi",
    "it",
    "ja",
    "ko",
    "zh",
    "zh-Hans",
    "zh-Hans-CN",
];

#[derive(Debug)]
struct BinaryPattern {
    first: u8,
    prefix: Box<str>,
    between: Box<str>,
    suffix: Box<str>,
}
impl BinaryPattern {
    fn new(text: String) -> Result<Self, DisplayNamesError> {
        let zero = text
            .find("{0}")
            .ok_or_else(|| data_error("composition lacks zero"))?;
        let one = text
            .find("{1}")
            .ok_or_else(|| data_error("composition lacks one"))?;
        let (first, left, right) = if zero < one {
            (0, zero, one)
        } else {
            (1, one, zero)
        };
        let prefix = &text[..left];
        let between = &text[left + 3..right];
        let suffix = &text[right + 3..];
        if [prefix, between, suffix]
            .iter()
            .any(|piece| piece.contains(['{', '}']))
        {
            return Err(data_error("unknown or repeated composition placeholder"));
        }
        Ok(Self {
            first,
            prefix: prefix.into(),
            between: between.into(),
            suffix: suffix.into(),
        })
    }
    fn compose(&self, zero: &str, one: &str) -> Result<String, DisplayNamesError> {
        let (first, second) = if self.first == 0 {
            (zero, one)
        } else {
            (one, zero)
        };
        let size = [
            self.prefix.as_ref(),
            first,
            self.between.as_ref(),
            second,
            self.suffix.as_ref(),
        ]
        .iter()
        .try_fold(0usize, |sum, part| sum.checked_add(part.len()))
        .ok_or(DisplayNamesError::Resource("composition extent"))?;
        u32::try_from(size)
            .map_err(|_| DisplayNamesError::Resource("composition exceeds Wasm32"))?;
        let mut output = String::new();
        output
            .try_reserve_exact(size)
            .map_err(|_| DisplayNamesError::Resource("composition allocation"))?;
        for text in [
            self.prefix.as_ref(),
            first,
            self.between.as_ref(),
            second,
            self.suffix.as_ref(),
        ] {
            output.push_str(text);
        }
        Ok(output)
    }
}
#[derive(Debug)]
struct LanguageName {
    code: LanguageCode,
    name: Box<str>,
}
type NameMap = BTreeMap<Box<str>, Box<str>>;
#[derive(Debug)]
struct Names {
    language: Arc<[LanguageName]>,
    region: Arc<NameMap>,
    script: Arc<NameMap>,
    language_script: Arc<NameMap>,
    currency: Arc<NameMap>,
    calendar: Arc<NameMap>,
    date_time_field: Arc<NameMap>,
    variant: Arc<NameMap>,
}
#[derive(Debug)]
enum CheckedPool {
    Language(Arc<[LanguageName]>),
    Region(Arc<NameMap>),
    Script(Arc<NameMap>),
    LanguageScript(Arc<NameMap>),
    Currency(Arc<NameMap>),
    Calendar(Arc<NameMap>),
    DateTimeField(Arc<NameMap>),
    Variant(Arc<NameMap>),
}
fn check_rows(rows: &[(String, String)]) -> Result<(), DisplayNamesError> {
    if rows.is_empty()
        || rows.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        || rows
            .iter()
            .any(|(_, name)| name.is_empty() || name == "↑↑↑" || name == "∅∅∅")
    {
        return Err(data_error(
            "empty, unordered, duplicate, or unresolved names",
        ));
    }
    Ok(())
}
fn names_map(
    rows: Vec<(String, String)>,
    kind: DisplayNamesType,
    canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
) -> Result<Arc<NameMap>, DisplayNamesError> {
    rows.into_iter()
        .map(|(source, name)| {
            let code =
                DisplayNameCode::from_text_with(kind, &source, canonicalize).map_err(data_error)?;
            if code.as_str() != source {
                return Err(data_error("noncanonical nonlanguage name key"));
            }
            Ok((source.into_boxed_str(), name.into_boxed_str()))
        })
        .collect::<Result<NameMap, _>>()
        .map(Arc::new)
}
impl CheckedPool {
    fn from_raw(
        raw: raw::NamePool,
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
        catalogue: Option<&crate::display_names_image::DisplayNamesCatalogue<'_>>,
    ) -> Result<Self, DisplayNamesError> {
        // Only an exactly rederived currency projection can admit an empty
        // Currency pool. The complete-source decoder retains its old rule.
        let selected_currencies = catalogue.and_then(|owner| owner.currency_codes());
        if !matches!(raw.kind, raw::Domain::Currency)
            || selected_currencies.is_none()
            || !raw.entries.is_empty()
        {
            check_rows(&raw.entries)?;
        }
        if matches!(raw.kind, raw::Domain::Currency) {
            if let Some(codes) = selected_currencies {
                if raw.entries.iter().any(|(code, _)| {
                    !codes
                        .iter()
                        .any(|selected| selected.clone().ascii().as_slice() == code.as_bytes())
                }) {
                    return Err(data_error("unselected currency name"));
                }
            }
        }
        Ok(match raw.kind {
            raw::Domain::Language => {
                // CanonicalCodeForDisplayNames aliases the requested code
                // before lookup. Deprecated CLDR debugging translations do not
                // become names for a different canonical language identifier.
                let mut languages: BTreeMap<Box<str>, LanguageName> = BTreeMap::new();
                for (source, name) in raw.entries {
                    let checked = DisplayNameCode::from_text_with(
                        DisplayNamesType::Language,
                        &source,
                        canonicalize,
                    )
                    .map_err(data_error)?;
                    let language = checked.language().expect("checked language code").clone();
                    let exact = source == checked.as_str();
                    if !exact {
                        continue;
                    }
                    let candidate = LanguageName {
                        code: language,
                        name: name.into(),
                    };
                    if languages
                        .insert(checked.as_str().into(), candidate)
                        .is_some()
                    {
                        return Err(data_error("duplicate canonical language field"));
                    }
                }
                if languages.is_empty() {
                    return Err(data_error("empty canonical language fields"));
                }
                Self::Language(languages.into_values().collect())
            }
            raw::Domain::Region => Self::Region(names_map(
                raw.entries,
                DisplayNamesType::Region,
                canonicalize,
            )?),
            raw::Domain::Script => Self::Script(names_map(
                raw.entries,
                DisplayNamesType::Script,
                canonicalize,
            )?),
            raw::Domain::LanguageScript => Self::LanguageScript(names_map(
                raw.entries,
                DisplayNamesType::Script,
                canonicalize,
            )?),
            raw::Domain::Currency => Self::Currency(names_map(
                raw.entries,
                DisplayNamesType::Currency,
                canonicalize,
            )?),
            raw::Domain::Calendar => Self::Calendar(names_map(
                raw.entries,
                DisplayNamesType::Calendar,
                canonicalize,
            )?),
            raw::Domain::DateTimeField => {
                let fields = names_map(raw.entries, DisplayNamesType::DateTimeField, canonicalize)?;
                if fields.len() != DisplayNamesDateTimeField::ALL.len()
                    || DisplayNamesDateTimeField::ALL
                        .iter()
                        .any(|field| !fields.contains_key(field.name()))
                {
                    return Err(data_error("incomplete dateTimeField domain"));
                }
                Self::DateTimeField(fields)
            }
            raw::Domain::Variant => {
                let variants = raw
                    .entries
                    .into_iter()
                    .map(|(key, name)| {
                        if !code::valid_variant(&key) || key != key.to_ascii_lowercase() {
                            return Err(data_error("invalid variant key"));
                        }
                        Ok((key.into_boxed_str(), name.into_boxed_str()))
                    })
                    .collect::<Result<NameMap, _>>()?;
                Self::Variant(Arc::new(variants))
            }
        })
    }
}
impl Names {
    fn from_refs(
        raw: raw::Names,
        pools: &[CheckedPool],
        used: &mut [bool],
    ) -> Result<Self, DisplayNamesError> {
        macro_rules! reference {
            ($index:expr, $kind:ident) => {{
                let index = $index as usize;
                let pool = pools
                    .get(index)
                    .ok_or_else(|| data_error("name reference out of bounds"))?;
                let CheckedPool::$kind(value) = pool else {
                    return Err(data_error("name reference domain mismatch"));
                };
                used[index] = true;
                Arc::clone(value)
            }};
        }
        Ok(Self {
            language: reference!(raw.language, Language),
            region: reference!(raw.region, Region),
            script: reference!(raw.script, Script),
            language_script: reference!(raw.language_script, LanguageScript),
            currency: reference!(raw.currency, Currency),
            calendar: reference!(raw.calendar, Calendar),
            date_time_field: reference!(raw.date_time_field, DateTimeField),
            variant: reference!(raw.variant, Variant),
        })
    }
    fn language_name(
        &self,
        input: &LanguageCode,
        display: DisplayNamesLanguageDisplay,
        pattern: &BinaryPattern,
        separator: &BinaryPattern,
    ) -> Result<Option<Box<str>>, DisplayNamesError> {
        let mut best: Option<&LanguageName> = None;
        for candidate in self
            .language
            .iter()
            .filter(|row| row.code.is_subset_of(input))
        {
            if display == DisplayNamesLanguageDisplay::Standard
                && candidate.code.subtag_count() != 1
            {
                continue;
            }
            let rank = |row: &LanguageName| {
                (
                    row.code.subtag_count(),
                    row.code.script.is_some(),
                    row.code.region.is_some(),
                    input
                        .variants
                        .iter()
                        .map(|v| row.code.variants.contains(v))
                        .collect::<Vec<_>>(),
                )
            };
            if best.is_none_or(|current| rank(candidate) > rank(current)) {
                best = Some(candidate);
            }
        }
        let Some(best) = best else {
            return Ok(None);
        };
        let mut remaining: Vec<&str> = Vec::new();
        if let Some(script) = input
            .script
            .as_ref()
            .filter(|s| best.code.script.as_ref() != Some(*s))
        {
            let Some(name) = self.language_script.get(script.as_ref()) else {
                return Ok(None);
            };
            remaining.push(name);
        }
        if let Some(region) = input
            .region
            .as_ref()
            .filter(|r| best.code.region.as_ref() != Some(*r))
        {
            let Some(name) = self.region.get(region.as_ref()) else {
                return Ok(None);
            };
            remaining.push(name);
        }
        for variant in input
            .variants
            .iter()
            .filter(|v| !best.code.variants.contains(v))
        {
            let Some(name) = self.variant.get(variant.as_ref()) else {
                return Ok(None);
            };
            remaining.push(name);
        }
        let mut qualifiers: Option<String> = None;
        for name in remaining {
            qualifiers = Some(match qualifiers {
                None => name.to_owned(),
                Some(previous) => separator.compose(&previous, name)?,
            });
        }
        Ok(Some(match qualifiers {
            None => best.name.clone(),
            Some(qualifiers) => pattern.compose(&best.name, &qualifiers)?.into_boxed_str(),
        }))
    }
}

#[derive(Debug)]
pub(super) struct LocaleProfile {
    locale: CanonicalLocaleId,
    pattern: BinaryPattern,
    separator: BinaryPattern,
    names: [Names; 3],
}
impl LocaleProfile {
    pub(super) fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
}
#[derive(Debug)]
pub struct DisplayNamesProfiles {
    profiles: Box<[Arc<LocaleProfile>]>,
}
impl DisplayNamesProfiles {
    pub fn from_pinned_data<P: IntlOperationProvider<CanonicalizeLocale>>(
        canonicalizer: &P,
    ) -> Result<Self, DisplayNamesError> {
        Self::from_json(DISPLAY_NAMES_PROFILE, canonicalizer)
    }
    pub fn from_json<P: IntlOperationProvider<CanonicalizeLocale>>(
        bytes: &[u8],
        canonicalizer: &P,
    ) -> Result<Self, DisplayNamesError> {
        Self::from_json_with(bytes, &|locale| {
            canonicalizer
                .execute(LocaleTransformRequest::new(locale))
                .map(|result| result.into_locale())
        })
    }
    pub(crate) fn from_image_data(
        bytes: &[u8],
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<Self, DisplayNamesError> {
        Self::from_json_with(bytes, &|locale| canonicalizer.canonicalize(&locale))
    }
    pub(crate) fn from_projected_image_data(
        catalogue: &crate::display_names_image::DisplayNamesCatalogue<'_>,
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<Self, DisplayNamesError> {
        // Only the complete pinned-source projection can mint this catalogue.
        // It already proved exact selected bytes; the native decoder still
        // checks every original field, pool domain and reachable association.
        let locales = catalogue
            .locales()
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>();
        Self::from_raw_with(
            serde_json::from_slice(catalogue.bytes()).map_err(data_error)?,
            &locales,
            &|locale| canonicalizer.canonicalize(&locale),
            Some(catalogue),
        )
    }
    fn from_json_with(
        bytes: &[u8],
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
    ) -> Result<Self, DisplayNamesError> {
        if <[u8; 32]>::from(Sha256::digest(bytes)) != DISPLAY_NAMES_DATA_SHA256 {
            return Err(data_error("payload checksum mismatch"));
        }
        Self::from_raw_with(
            serde_json::from_slice(bytes).map_err(data_error)?,
            LOCALES,
            canonicalize,
            None,
        )
    }
    #[cfg(test)]
    pub(super) fn from_raw<P: IntlOperationProvider<CanonicalizeLocale>>(
        raw: raw::Profile,
        canonicalizer: &P,
    ) -> Result<Self, DisplayNamesError> {
        Self::from_raw_with(
            raw,
            LOCALES,
            &|locale| {
                canonicalizer
                    .execute(LocaleTransformRequest::new(locale))
                    .map(|result| result.into_locale())
            },
            None,
        )
    }
    fn from_raw_with(
        raw: raw::Profile,
        expected_locales: &[&str],
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
        catalogue: Option<&crate::display_names_image::DisplayNamesCatalogue<'_>>,
    ) -> Result<Self, DisplayNamesError> {
        if raw.schema_version != 1
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != CLDR_COMMIT
            || raw.source_manifest_sha256 != identity::SOURCE_MANIFEST_SHA256
            || raw.bcp47_manifest_sha256 != identity::BCP47_MANIFEST_SHA256
            || raw.default_locale != "en-US"
            || raw.locales.len() != expected_locales.len()
            || raw
                .locales
                .iter()
                .map(|row| row.locale.as_str())
                .ne(expected_locales.iter().copied())
        {
            return Err(data_error("source identity or exact catalogue changed"));
        }
        let mut previous: Option<Vec<u8>> = None;
        let mut pools = Vec::new();
        for row in raw.name_pool {
            let canonical = serde_json::to_vec(&serde_json::to_value(&row).map_err(data_error)?)
                .map_err(data_error)?;
            if previous.as_ref().is_some_and(|key| key >= &canonical) {
                return Err(data_error("noncanonical or duplicate name pool"));
            }
            previous = Some(canonical);
            pools.push(CheckedPool::from_raw(row, canonicalize, catalogue)?);
        }
        let mut used = vec![false; pools.len()];
        let mut profiles = Vec::new();
        for row in raw.locales {
            let locale = CanonicalLocaleId::from_data(row.locale).map_err(data_error)?;
            profiles.push(Arc::new(LocaleProfile {
                locale,
                pattern: BinaryPattern::new(row.locale_pattern)?,
                separator: BinaryPattern::new(row.locale_separator)?,
                names: [
                    Names::from_refs(row.styles.long, &pools, &mut used)?,
                    Names::from_refs(row.styles.short, &pools, &mut used)?,
                    Names::from_refs(row.styles.narrow, &pools, &mut used)?,
                ],
            }));
        }
        if used.iter().any(|&value| !value) {
            return Err(data_error("unused name pool"));
        }
        let result = Self {
            profiles: profiles.into_boxed_slice(),
        };
        result.admit(CanonicalLocaleId::from_data("en-US").map_err(data_error)?)?;
        Ok(result)
    }
    pub fn available_locales(&self) -> impl ExactSizeIterator<Item = &CanonicalLocaleId> {
        self.profiles.iter().map(|p| p.locale())
    }
    /// A selected provider consumes only configurations minted by its own
    /// admitted templates, even when another image contains identical bytes.
    pub(crate) fn display_name_with_data(
        &self,
        request: &DisplayNameRequest,
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<DisplayNameResult, DisplayNamesError> {
        self.display_name_with(request, &|locale| canonicalizer.canonicalize(&locale))
    }
    fn display_name_with(
        &self,
        request: &DisplayNameRequest,
        canonicalize: &impl Fn(LocaleId) -> Result<CanonicalLocaleId, LocaleTransformError>,
    ) -> Result<DisplayNameResult, DisplayNamesError> {
        let locale = request.configuration().locale();
        let index = self
            .profiles
            .binary_search_by(|profile| profile.locale().as_str().cmp(locale.resolved().as_str()))
            .map_err(|_| DisplayNamesError::InvalidResolvedLocale)?;
        if !Arc::ptr_eq(&self.profiles[index], &locale.profile) {
            return Err(DisplayNamesError::InvalidResolvedLocale);
        }
        let code = DisplayNameCode::parse_with(
            request.configuration().selection().kind(),
            request.code(),
            canonicalize,
        )?;
        display_name(request.configuration(), &code)
    }
    pub fn admit(
        &self,
        locale: CanonicalLocaleId,
    ) -> Result<ResolvedDisplayNamesLocale, DisplayNamesError> {
        let index = self
            .profiles
            .binary_search_by(|p| p.locale.as_str().cmp(locale.as_str()))
            .map_err(|_| DisplayNamesError::InvalidResolvedLocale)?;
        Ok(ResolvedDisplayNamesLocale {
            profile: Arc::clone(&self.profiles[index]),
        })
    }
    fn matching(&self, requested: &str, matcher: LocaleMatcher) -> Option<usize> {
        // Best-fit intentionally uses the allowed lookup policy over this finite
        // complete catalogue, without advertising additional source locales.
        match matcher {
            LocaleMatcher::Lookup | LocaleMatcher::BestFit => {}
        }
        let mut candidate = requested;
        loop {
            if let Ok(index) = self
                .profiles
                .binary_search_by(|p| p.locale.as_str().cmp(candidate))
            {
                return Some(index);
            }
            let position = candidate.rfind('-')?;
            candidate = &candidate[..position];
            if candidate
                .rsplit('-')
                .next()
                .is_some_and(|part| part.len() == 1)
            {
                candidate = &candidate[..candidate.rfind('-')?];
            }
        }
    }
    pub fn resolve(
        &self,
        request: &DisplayNamesLocaleRequest,
    ) -> Result<ResolvedDisplayNamesLocale, DisplayNamesError> {
        for locale in &request.requested {
            if let Some(index) = self.matching(locale.as_str(), request.matcher) {
                return Ok(ResolvedDisplayNamesLocale {
                    profile: Arc::clone(&self.profiles[index]),
                });
            }
        }
        self.admit(CanonicalLocaleId::from_data("en-US").map_err(data_error)?)
    }
    pub fn supported_locales(
        &self,
        request: &DisplayNamesLocaleRequest,
    ) -> DisplayNamesSupportedLocalesResult {
        let mut output = Vec::new();
        for locale in &request.requested {
            if self.matching(locale.as_str(), request.matcher).is_some() && !output.contains(locale)
            {
                output.push(locale.clone());
            }
        }
        DisplayNamesSupportedLocalesResult {
            locales: output.into_boxed_slice(),
        }
    }
}

pub(super) fn display_name(
    configuration: &CheckedDisplayNamesConfiguration,
    code: &DisplayNameCode,
) -> Result<DisplayNameResult, DisplayNamesError> {
    if code.kind() != configuration.selection.kind() {
        return Err(DisplayNamesError::InvalidConfiguration);
    }
    let profile = &configuration.locale.profile;
    let names = &profile.names[match configuration.style {
        DisplayNamesStyle::Long => 0,
        DisplayNamesStyle::Short => 1,
        DisplayNamesStyle::Narrow => 2,
    }];
    let name = match configuration.selection {
        DisplayNamesSelection::Language(display) => names.language_name(
            code.language().expect("checked language kind"),
            display,
            &profile.pattern,
            &profile.separator,
        )?,
        DisplayNamesSelection::Region => names.region.get(code.as_str()).cloned(),
        DisplayNamesSelection::Script => names.script.get(code.as_str()).cloned(),
        DisplayNamesSelection::Currency => names.currency.get(code.as_str()).cloned(),
        DisplayNamesSelection::Calendar => names.calendar.get(code.as_str()).cloned(),
        DisplayNamesSelection::DateTimeField => names.date_time_field.get(code.as_str()).cloned(),
    };
    DisplayNameResult::new(name.or_else(|| {
        (configuration.fallback == DisplayNamesFallback::Code).then(|| code.as_str().into())
    }))
}
