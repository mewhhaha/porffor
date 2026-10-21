//! Full small Locale preference authority, independent of public format locales.
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct LocaleCollationPreferences {
    locales: Box<[(CanonicalLocaleId, Box<[Box<str>]>)]>,
}
impl LocaleCollationPreferences {
    pub(super) fn from_projection(catalogue: &crate::collator_image::CollatorCatalogue) -> Self {
        Self {
            locales: catalogue.preference_rows().iter().cloned().collect(),
        }
    }
    pub(super) fn from_profiles(locales: &[LocaleProfile]) -> Self {
        Self {
            locales: locales
                .iter()
                .map(|profile| {
                    (
                        profile.locale.clone(),
                        profile
                            .collations
                            .keys()
                            .filter(|name| {
                                !matches!(name.as_ref(), "standard" | "search" | "searchjl")
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )
                })
                .collect(),
        }
    }
    pub(crate) fn rows(&self) -> impl Iterator<Item = (&CanonicalLocaleId, &[Box<str>])> {
        self.locales
            .iter()
            .map(|(locale, names)| (locale, names.as_ref()))
    }
    pub(super) fn resolve(
        &self,
        requested: &CanonicalLocaleId,
        roots: &LocaleIndependentSortPreferences,
    ) -> Result<Box<[Box<str>]>, CollatorOperationError> {
        // This checked canonical identifier has the full Unicode language domain,
        // including 5–8-letter base and transform languages that ICU Locale cannot
        // parse. Lookup needs only RemoveUnicodeLocaleExtension, not reparsing or
        // language substitution. A private-use tail is opaque, including `u`.
        let mut name = String::new();
        let mut unicode = false;
        let mut private = false;
        for subtag in requested.as_str().split('-') {
            if !private && subtag.len() == 1 {
                private = subtag == "x";
                unicode = subtag == "u";
            }
            if !unicode {
                if !name.is_empty() {
                    name.push('-');
                }
                name.push_str(subtag);
            }
        }
        let mut candidate = name.as_str();
        let selected = loop {
            if let Ok(index) = self
                .locales
                .binary_search_by(|(locale, _)| locale.as_str().cmp(candidate))
            {
                break Some(self.locales[index].1.as_ref());
            }
            let Some(shortened) = truncate_lookup(candidate) else {
                break None;
            };
            candidate = shortened;
        };
        match selected {
            Some(names) => Self::owned_names(names.iter().map(Box::as_ref)),
            None => Self::owned_names(roots.0.iter().map(|profile| {
                profile
                    .public_collation()
                    .expect("checked root sort profile name")
            })),
        }
    }
    fn owned_names<'a>(
        selected: impl ExactSizeIterator<Item = &'a str>,
    ) -> Result<Box<[Box<str>]>, CollatorOperationError> {
        let mut names = Vec::new();
        names
            .try_reserve_exact(selected.len())
            .map_err(|_| CollatorOperationError::Resource("Locale collation list allocation"))?;
        for name in selected {
            let mut owned = String::new();
            owned.try_reserve_exact(name.len()).map_err(|_| {
                CollatorOperationError::Resource("Locale collation name allocation")
            })?;
            owned.push_str(name);
            names.push(owned.into_boxed_str());
        }
        Ok(names.into_boxed_slice())
    }
}
