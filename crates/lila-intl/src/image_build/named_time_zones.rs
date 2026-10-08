//! Export the exact locked IANA2026a records into one native payload.

use std::error::Error;

const MAGIC: &[u8; 8] = b"LILATZ01";
const CATALOGUE: &[u8] = include_bytes!("../../data/iana-tzdb-2026a/catalogue.tsv");
const ZONE_TAB: &[u8] = include_bytes!("../../data/locale-time-zones-iana2026a/zone.tab");
const REGIONS: &[u8] = include_bytes!("../../data/locale-time-zones-iana2026a/regions.tsv");

fn put_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    output.extend_from_slice(&u32::try_from(bytes.len())?.to_le_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

pub(super) fn export() -> Result<Vec<u8>, Box<dyn Error>> {
    if jiff_tzdb::VERSION != Some("2026a") {
        return Err("named image exporter requires exact IANA2026a".into());
    }
    let source = core::str::from_utf8(CATALOGUE)?;
    let mut records = Vec::new();
    let mut previous = None;
    for line in source.lines() {
        let mut fields = line.split('\t');
        let name = fields.next().ok_or("missing named catalogue identifier")?;
        let _primary = fields.next().ok_or("missing named catalogue primary")?;
        let _digest = fields.next().ok_or("missing named catalogue digest")?;
        if fields.next().is_some() || previous.is_some_and(|before| before >= name) {
            return Err("invalid or unordered named catalogue row".into());
        }
        previous = Some(name);
        let (normalized, bytes) = jiff_tzdb::get(name).ok_or("missing named transition record")?;
        if normalized != name || !bytes.starts_with(b"TZif") {
            return Err("named transition record spelling or format differs".into());
        }
        records.push((name, bytes));
    }
    if records.len() != 598 || jiff_tzdb::available().count() != records.len() {
        return Err("named catalogue and actual transition archive differ".into());
    }
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    put_bytes(&mut output, CATALOGUE)?;
    put_bytes(&mut output, ZONE_TAB)?;
    put_bytes(&mut output, REGIONS)?;
    output.extend_from_slice(&u32::try_from(records.len())?.to_le_bytes());
    for (name, bytes) in records {
        put_bytes(&mut output, name.as_bytes())?;
        put_bytes(&mut output, bytes)?;
    }
    Ok(output)
}
