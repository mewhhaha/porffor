//! Export and import retain the same twelve admitted component owners.
use super::*;
use crate::{IntlDataImageError, MAX_INTL_COMPONENT_IMAGE_BYTES};

const MAGIC: &[u8; 8] = b"LILAB001";
const SPARSE_MAGIC: &[u8; 8] = b"LILAB002";
const COMPONENTS: usize = 12;
const MAX_IDENTITY_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub enum IntlBundleExportError {
    Framing(&'static str),
    TooLarge,
    Allocation,
    Identity,
    Image(IntlDataImageError),
    Provider(String),
    Catalogue(String),
}
impl core::fmt::Display for IntlBundleExportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Framing(reason) => write!(f, "invalid Intl bundle export: {reason}"),
            Self::TooLarge => {
                f.write_str("Intl bundle export exceeds its bounded image or identity size")
            }
            Self::Allocation => f.write_str("cannot allocate the bounded Intl bundle export"),
            Self::Identity => {
                f.write_str("Intl bundle identity differs from its admitted components")
            }
            Self::Image(error) => write!(f, "{error}"),
            Self::Provider(error) => write!(f, "Intl bundle provider admission failed: {error}"),
            Self::Catalogue(error) => {
                write!(f, "Intl bundle supported-values admission failed: {error}")
            }
        }
    }
}
impl std::error::Error for IntlBundleExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Image(error) => Some(error),
            Self::Framing(_)
            | Self::TooLarge
            | Self::Allocation
            | Self::Identity
            | Self::Provider(_)
            | Self::Catalogue(_) => None,
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], IntlBundleExportError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(IntlBundleExportError::TooLarge)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(IntlBundleExportError::Framing("truncated payload"))?;
        self.offset = end;
        Ok(bytes)
    }
    fn u16(&mut self) -> Result<u16, IntlBundleExportError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
    fn u32(&mut self) -> Result<u32, IntlBundleExportError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    fn component(&mut self, expected: u16) -> Result<Arc<[u8]>, IntlBundleExportError> {
        if self.u16()? != expected || self.u16()? != 0 {
            return Err(IntlBundleExportError::Framing(
                "noncanonical component order or reserved word",
            ));
        }
        let length = self.u32()? as usize;
        if length == 0 || length > MAX_INTL_COMPONENT_IMAGE_BYTES {
            return Err(IntlBundleExportError::TooLarge);
        }
        Ok(Arc::from(self.take(length)?))
    }
}

impl SelectedIntlDataBundle {
    pub const MAX_EXPORT_BYTES: usize =
        16 + MAX_IDENTITY_BYTES + COMPONENTS * (8 + MAX_INTL_COMPONENT_IMAGE_BYTES);

    pub fn export_bytes(&self) -> Result<Arc<[u8]>, IntlBundleExportError> {
        self.admit_export_catalogues()?;
        let identity = self.identity().artifact_identity();
        let identity = identity.as_bytes();
        if identity.len() > MAX_IDENTITY_BYTES {
            return Err(IntlBundleExportError::TooLarge);
        }
        let sections = self.component_sections();
        let mut total = 16 + identity.len();
        for (_, bytes) in &sections {
            if bytes.is_empty() || bytes.len() > MAX_INTL_COMPONENT_IMAGE_BYTES {
                return Err(IntlBundleExportError::TooLarge);
            }
            total = total
                .checked_add(8 + bytes.len())
                .ok_or(IntlBundleExportError::TooLarge)?;
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(total)
            .map_err(|_| IntlBundleExportError::Allocation)?;
        output.extend_from_slice(if self.service_selection.is_some() {
            SPARSE_MAGIC
        } else {
            MAGIC
        });
        output.extend_from_slice(&(identity.len() as u32).to_le_bytes());
        output.extend_from_slice(&(sections.len() as u16).to_le_bytes());
        output.extend_from_slice(
            &self
                .service_selection
                .as_ref()
                .map_or(0, CheckedIntlServiceSelection::wire)
                .to_le_bytes(),
        );
        output.extend_from_slice(identity);
        for (name, bytes) in sections {
            let component = IntlDataComponent::ALL
                .into_iter()
                .find(|component| component.section_name() == name)
                .ok_or(IntlBundleExportError::Framing("unknown component section"))?;
            output.extend_from_slice(&component.code().to_le_bytes());
            output.extend_from_slice(&0u16.to_le_bytes());
            output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            output.extend_from_slice(&bytes);
        }
        Ok(output.into())
    }

    pub fn from_export_bytes(bytes: &[u8]) -> Result<Self, IntlBundleExportError> {
        if bytes.len() > Self::MAX_EXPORT_BYTES {
            return Err(IntlBundleExportError::TooLarge);
        }
        let mut reader = Reader { bytes, offset: 0 };
        let magic = reader.take(8)?;
        if magic != MAGIC && magic != SPARSE_MAGIC {
            return Err(IntlBundleExportError::Framing(
                "unsupported magic or version",
            ));
        }
        let identity_length = reader.u32()? as usize;
        if identity_length == 0 || identity_length > MAX_IDENTITY_BYTES {
            return Err(IntlBundleExportError::TooLarge);
        }
        let count = reader.u16()? as usize;
        let wire = reader.u16()?;
        let selection = if magic == MAGIC {
            if count != COMPONENTS || wire != 0 {
                return Err(IntlBundleExportError::Framing(
                    "incomplete component inventory or reserved word",
                ));
            }
            None
        } else {
            Some(CheckedIntlServiceSelection::from_wire(wire).map_err(|_| {
                IntlBundleExportError::Framing("invalid requested service selection")
            })?)
        };
        let checked = selection.clone().unwrap_or_else(|| {
            CheckedIntlServiceSelection::new(IntlServiceSet::ALL).expect("ALL is nonempty")
        });
        if count != checked.components().len() {
            return Err(IntlBundleExportError::Framing(
                "component closure differs from requested services",
            ));
        }
        let identity = reader.take(identity_length)?;
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(count)
            .map_err(|_| IntlBundleExportError::Allocation)?;
        for component in checked.components().iter() {
            frames.push((component, reader.component(component.code())?));
        }
        if reader.offset != bytes.len() {
            return Err(IntlBundleExportError::Framing("trailing bytes"));
        }
        let bundle = Self::from_component_sections(selection, frames)?;
        if bundle.identity().artifact_identity().as_bytes() != identity {
            return Err(IntlBundleExportError::Identity);
        }
        bundle.admit_export_catalogues()?;
        Ok(bundle)
    }

    fn admit_export_catalogues(&self) -> Result<(), IntlBundleExportError> {
        for key in SupportedValuesKey::ALL {
            match self.supported_values(key) {
                Ok(_) | Err(SupportedValuesSetupError::UnavailableData(_)) => {}
                Err(error) => return Err(IntlBundleExportError::Catalogue(error.to_string())),
            }
        }
        Ok(())
    }

    /// Admit only the exact checked frame closure. Missing foundations never
    /// acquire embedded fallback images, and extra frames cannot grant services.
    pub fn from_component_sections(
        service_selection: Option<CheckedIntlServiceSelection>,
        frames: Vec<(IntlDataComponent, Arc<[u8]>)>,
    ) -> Result<Self, IntlBundleExportError> {
        let checked = service_selection.clone().unwrap_or_else(|| {
            CheckedIntlServiceSelection::new(IntlServiceSet::ALL).expect("ALL is nonempty")
        });
        if frames.len() != checked.components().len()
            || !frames
                .iter()
                .map(|(component, _)| *component)
                .eq(checked.components().iter())
        {
            return Err(IntlBundleExportError::Framing(
                "noncanonical or incomplete required component closure",
            ));
        }
        let mut frames = frames.into_iter().peekable();
        fn take(
            frames: &mut std::iter::Peekable<std::vec::IntoIter<(IntlDataComponent, Arc<[u8]>)>>,
            component: IntlDataComponent,
        ) -> Option<Arc<[u8]>> {
            if frames.peek().map(|(kind, _)| *kind) == Some(component) {
                frames.next().map(|(_, bytes)| bytes)
            } else {
                None
            }
        }
        fn required<T>(value: &Option<T>) -> Result<&T, IntlBundleExportError> {
            value.as_ref().ok_or(IntlBundleExportError::Framing(
                "missing required foundation",
            ))
        }
        let image = IntlBundleExportError::Image;
        let locale = LocaleDataImage::from_bytes(
            take(&mut frames, IntlDataComponent::Locale)
                .ok_or(IntlBundleExportError::Framing("missing Locale foundation"))?,
        )
        .map_err(image)?;
        let lists = take(&mut frames, IntlDataComponent::List)
            .map(|bytes| ListDataImage::from_bytes(bytes, &locale))
            .transpose()
            .map_err(image)?;
        let collators = take(&mut frames, IntlDataComponent::Collator)
            .map(|bytes| CollatorDataImage::from_bytes(bytes, &locale))
            .transpose()
            .map_err(image)?;
        let numbers = if let Some(bytes) = take(&mut frames, IntlDataComponent::Number) {
            Some(
                NumberProfilesDataImage::from_bytes(bytes, &locale, required(&lists)?)
                    .map_err(image)?,
            )
        } else {
            None
        };
        let segmenters = take(&mut frames, IntlDataComponent::Segmenter)
            .map(|bytes| SegmenterDataImage::from_bytes(bytes, &locale))
            .transpose()
            .map_err(image)?;
        let display_names = take(&mut frames, IntlDataComponent::DisplayNames)
            .map(|bytes| DisplayNamesDataImage::from_bytes(bytes, &locale))
            .transpose()
            .map_err(image)?;
        let relative_times = if let Some(bytes) = take(&mut frames, IntlDataComponent::RelativeTime)
        {
            Some(
                RelativeTimeDataImage::from_bytes(bytes, &locale, required(&numbers)?)
                    .map_err(image)?,
            )
        } else {
            None
        };
        let durations = if let Some(bytes) = take(&mut frames, IntlDataComponent::Duration) {
            Some(
                DurationDataImage::from_bytes(
                    bytes,
                    &locale,
                    required(&numbers)?,
                    required(&lists)?,
                )
                .map_err(image)?,
            )
        } else {
            None
        };
        let named_zones = take(&mut frames, IntlDataComponent::NamedTimeZones)
            .map(NamedTimeZoneDataImage::from_bytes)
            .transpose()
            .map_err(image)?;
        let date_time = if let Some(bytes) = take(&mut frames, IntlDataComponent::DateTime) {
            Some(
                DateTimeDataImage::from_bytes(bytes, &locale, required(&named_zones)?)
                    .map_err(image)?,
            )
        } else {
            None
        };
        let time_zone_names =
            if let Some(bytes) = take(&mut frames, IntlDataComponent::TimeZoneNames) {
                Some(
                    TimeZoneNamesDataImage::from_bytes(bytes, required(&named_zones)?)
                        .map_err(image)?,
                )
            } else {
                None
            };
        let locale_information = if let Some(bytes) =
            take(&mut frames, IntlDataComponent::LocaleInformation)
        {
            Some(
                NativeLocaleInformationDataImage::from_bytes(bytes, &locale, required(&date_time)?)
                    .map_err(image)?,
            )
        } else {
            None
        };
        let provider = EmbeddedIntlProvider::with_selected_data_images(
            checked,
            locale.clone(),
            lists.clone(),
            collators.clone(),
            numbers.clone(),
            segmenters.clone(),
            display_names.clone(),
            relative_times.clone(),
            durations.clone(),
            named_zones.clone(),
            date_time.clone(),
            time_zone_names.clone(),
            locale_information.clone(),
        )
        .map_err(|error| IntlBundleExportError::Provider(error.to_string()))?;
        Ok(Self {
            locale,
            lists,
            collators,
            numbers,
            segmenters,
            display_names,
            relative_times,
            durations,
            named_zones,
            date_time,
            time_zone_names,
            locale_information,
            service_selection,
            provider,
            catalogues: std::array::from_fn(|_| OnceLock::new()),
        })
    }

    #[cfg(test)]
    fn from_admitted_export_components(
        frames: [Arc<[u8]>; COMPONENTS],
    ) -> Result<Self, IntlBundleExportError> {
        Self::from_component_sections(
            None,
            IntlDataComponent::ALL.into_iter().zip(frames).collect(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_export_roundtrip_preserves_actual_frames_identity_and_owned_catalogues() {
        let profile = CustomIntlProfile::from_manifest_json(r#"{"schema_version":1,"custom_id":"export-source","locale_filters":{"list":["fr"],"relative_time":["fr"],"number_plural":["es"]}}"#).unwrap();
        let selected = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(profile));
        let original = selected.selected().unwrap();
        let first = original.export_bytes().unwrap();
        let admitted = SelectedIntlDataBundle::from_export_bytes(&first).unwrap();
        assert_eq!(admitted.identity(), original.identity());
        assert_eq!(admitted.export_bytes().unwrap(), first);
        for ((left_name, left), (right_name, right)) in original
            .component_sections()
            .into_iter()
            .zip(admitted.component_sections())
        {
            assert_eq!(left_name, right_name);
            assert_eq!(left, right);
        }
        for key in SupportedValuesKey::ALL {
            assert_eq!(
                admitted.supported_values(key).unwrap().provider_identity(),
                admitted.identity()
            );
        }
    }
    #[test]
    fn complete_exports_reject_truncation_damage_reorder_identity_and_trailing_data() {
        let selected = IntlDataSelection::new(IntlCompilationProfile::Conformance);
        let original = selected.selected().unwrap();
        let bytes = original.export_bytes().unwrap();
        for length in [0, 8, 15, bytes.len() - 1] {
            assert!(SelectedIntlDataBundle::from_export_bytes(&bytes[..length]).is_err());
        }
        let mut damaged = bytes.to_vec();
        damaged[16] ^= 1;
        assert!(matches!(
            SelectedIntlDataBundle::from_export_bytes(&damaged),
            Err(IntlBundleExportError::Identity)
        ));
        let identity_length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let mut damaged = bytes.to_vec();
        damaged[16 + identity_length] = 1;
        assert!(SelectedIntlDataBundle::from_export_bytes(&damaged).is_err());
        let mut damaged = bytes.to_vec();
        damaged[12] = 11;
        assert!(SelectedIntlDataBundle::from_export_bytes(&damaged).is_err());
        let mut damaged = bytes.to_vec();
        let last = damaged.len() - 1;
        damaged[last] ^= 1;
        assert!(SelectedIntlDataBundle::from_export_bytes(&damaged).is_err());
        let mut damaged = bytes.to_vec();
        damaged.push(0);
        assert!(SelectedIntlDataBundle::from_export_bytes(&damaged).is_err());
    }
    #[test]
    fn export_admission_rebinds_actual_list_number_dependency_without_trusting_same_custom_id() {
        let first = CustomIntlProfile::from_manifest_json(r#"{"schema_version":1,"custom_id":"same-export-id","locale_filters":{"list":["es"],"number_plural":["es"]}}"#).unwrap();
        let second = CustomIntlProfile::from_manifest_json(r#"{"schema_version":1,"custom_id":"same-export-id","locale_filters":{"list":["fr"],"number_plural":["es"]}}"#).unwrap();
        let first = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(first));
        let second = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(second));
        let mut frames = first
            .selected()
            .unwrap()
            .component_sections()
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        frames[1] = second.selected().unwrap().component_sections()[1].1.clone();
        assert!(SelectedIntlDataBundle::from_admitted_export_components(
            frames.try_into().unwrap()
        )
        .is_err());
    }
    #[test]
    fn export_admission_rejects_same_id_unpaired_currency_images() {
        let id = CustomProfileId::parse("paired-currency-export").unwrap();
        let first = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_currency_codes(id.clone(), &["EUR"]).unwrap(),
        ));
        let second = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_currency_codes(id, &["EUR", "JPY"]).unwrap(),
        ));
        let mut frames = first
            .selected()
            .unwrap()
            .component_sections()
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        frames[5] = second.selected().unwrap().component_sections()[5].1.clone();
        assert!(SelectedIntlDataBundle::from_admitted_export_components(
            frames.try_into().unwrap()
        )
        .is_err());
    }
    #[test]
    fn export_admission_rejects_same_id_unpaired_numbering_images_with_valid_locale_info() {
        let id = CustomProfileId::parse("paired-numbering-export").unwrap();
        let first = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_numbering_systems(id.clone(), &["deva"]).unwrap(),
        ));
        let second = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_numbering_systems(id, &["beng"]).unwrap(),
        ));
        let mut frames = first
            .selected()
            .unwrap()
            .component_sections()
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        let foreign = second.selected().unwrap().component_sections();
        frames[9] = foreign[9].1.clone();
        frames[11] = foreign[11].1.clone();
        assert!(SelectedIntlDataBundle::from_admitted_export_components(
            frames.try_into().unwrap()
        )
        .is_err());
    }
    #[test]
    fn export_admission_rejects_same_id_named_zone_dependents_from_another_physical_closure() {
        let id = CustomProfileId::parse("named-zone-export").unwrap();
        let first = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_named_time_zones(id.clone(), &["America/New_York"]).unwrap(),
        ));
        let second = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::for_named_time_zones(id, &["Europe/Paris"]).unwrap(),
        ));
        let mut frames = first
            .selected()
            .unwrap()
            .component_sections()
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        let foreign = second.selected().unwrap().component_sections();
        frames[9] = foreign[9].1.clone();
        frames[11] = foreign[11].1.clone();
        assert!(SelectedIntlDataBundle::from_admitted_export_components(
            frames.try_into().unwrap()
        )
        .is_err());
        let mut frames = first
            .selected()
            .unwrap()
            .component_sections()
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect::<Vec<_>>();
        frames[10] = foreign[10].1.clone();
        assert!(SelectedIntlDataBundle::from_admitted_export_components(
            frames.try_into().unwrap()
        )
        .is_err());
    }
}
