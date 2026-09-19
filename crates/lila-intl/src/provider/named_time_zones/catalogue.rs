use sha2::{Digest, Sha256};

use crate::{InvalidTimeZoneData, NamedTimeZoneIdentity};

const CATALOGUE: &str = include_str!("../../../data/iana-tzdb-2026a/catalogue.tsv");

pub(super) struct CatalogueRow {
    pub(super) identity: NamedTimeZoneIdentity,
    pub(super) tzif_digest: [u8; 32],
}

pub(super) fn read() -> Result<Vec<CatalogueRow>, InvalidTimeZoneData> {
    let actual: [u8; 32] = Sha256::digest(CATALOGUE.as_bytes()).into();
    if actual != super::identity::CATALOGUE_SHA256 {
        return Err(InvalidTimeZoneData(
            "pinned named catalogue digest mismatch",
        ));
    }
    parse(CATALOGUE)
}

fn parse(source: &str) -> Result<Vec<CatalogueRow>, InvalidTimeZoneData> {
    let mut rows = Vec::new();
    let mut previous = None;
    for line in source.lines() {
        let mut fields = line.split('\t');
        let identifier = fields
            .next()
            .ok_or(InvalidTimeZoneData("missing catalogue identifier"))?;
        let primary = fields
            .next()
            .ok_or(InvalidTimeZoneData("missing catalogue primary"))?;
        let hash = fields
            .next()
            .ok_or(InvalidTimeZoneData("missing catalogue digest"))?;
        if fields.next().is_some() || hash.len() != 64 {
            return Err(InvalidTimeZoneData("invalid catalogue row extent"));
        }
        if previous.is_some_and(|previous| previous >= identifier) {
            return Err(InvalidTimeZoneData(
                "catalogue identifiers are not strictly ordered",
            ));
        }
        previous = Some(identifier);
        let identity = NamedTimeZoneIdentity::from_data(identifier, primary)?;
        let mut tzif_digest = [0_u8; 32];
        for (output, pair) in tzif_digest.iter_mut().zip(hash.as_bytes().chunks_exact(2)) {
            *output = hex_digit(pair[0])? * 16 + hex_digit(pair[1])?;
        }
        rows.push(CatalogueRow {
            identity,
            tzif_digest,
        });
    }
    if rows.is_empty() {
        return Err(InvalidTimeZoneData("empty named time-zone catalogue"));
    }
    Ok(rows)
}

fn hex_digit(byte: u8) -> Result<u8, InvalidTimeZoneData> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(InvalidTimeZoneData("invalid catalogue digest byte")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_catalogue_never_becomes_unknown_user_input() {
        let hash = "00".repeat(32);
        for source in [
            String::new(),
            "UTC".into(),
            format!("UTC\tUTC\t{hash}\textra"),
            format!("UTC\tUTC\t{}g", &hash[..63]),
            format!("+01:00\tUTC\t{hash}"),
            format!("UTC\tUTC\t{hash}\nUTC\tUTC\t{hash}"),
        ] {
            assert!(parse(&source).is_err(), "{source:?}");
        }
    }
}
