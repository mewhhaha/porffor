//! The consumed subset of pinned Test262 metadata, admitted once at discovery.
//! This is deliberately not a general YAML parser. Unrelated metadata stays
//! opaque; malformed execution flags, includes and negative expectations fail.
use super::{NegativeExpectation, NegativePhase};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub(super) struct Metadata {
    pub(super) flags: BTreeSet<String>,
    pub(super) features: BTreeSet<String>,
    pub(super) includes: Vec<String>,
    pub(super) negative: Option<NegativeExpectation>,
}

pub(super) fn parse(source: &str, path: &str) -> Result<Metadata, String> {
    parse_inner(source).map_err(|error| format!("invalid Test262 metadata for {path}: {error}"))
}

fn parse_inner(source: &str) -> Result<Metadata, String> {
    let Some(start) = source.find("/*---") else {
        return if source.contains("---*/") {
            Err("frontmatter end marker has no start".into())
        } else {
            // Explicit caller fixtures may be ordinary source without metadata.
            Ok(Metadata::default())
        };
    };
    if source.matches("/*---").count() != 1 || source.matches("---*/").count() != 1 {
        return Err("expected one complete frontmatter block".into());
    }
    let end = source.find("---*/").expect("one end marker counted");
    if end < start + 5 {
        return Err("frontmatter markers are reversed".into());
    }
    let body = source[start + 5..end]
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut fields: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut current = None;
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with([' ', '\t']) {
            if current.is_some() && !line.contains(':') {
                return Err(
                    "consumed metadata requires an indented continuation or a new mapping".into(),
                );
            }
            current = None;
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                if matches!(key, "flags" | "features" | "includes" | "negative") {
                    if fields.insert(key, vec![value.trim()]).is_some() {
                        return Err(format!("duplicate {key} field"));
                    }
                    current = Some(key);
                }
            }
        } else if let Some(key) = current {
            fields
                .get_mut(key)
                .expect("current field owns a row")
                .push(line.trim_end());
        }
    }
    let list = |key| parse_list(key, fields.get(key).map(Vec::as_slice));
    Ok(Metadata {
        flags: list("flags")?.into_iter().collect(),
        features: list("features")?.into_iter().collect(),
        includes: list("includes")?,
        negative: parse_negative(fields.get("negative").map(Vec::as_slice))?,
    })
}

fn bare(value: &str) -> bool {
    !value.is_empty()
        && !value.chars().any(|ch| {
            ch.is_whitespace() || matches!(ch, '[' | ']' | '{' | '}' | ',' | ':' | '#' | '\'' | '"')
        })
}

fn indented_rows<'a>(key: &str, lines: &[&'a str]) -> Result<Vec<&'a str>, String> {
    let indent = lines
        .first()
        .map_or(0, |line| line.len() - line.trim_start_matches(' ').len());
    if indent == 0
        || lines
            .iter()
            .any(|line| line.len() - line.trim_start_matches(' ').len() != indent)
    {
        return Err(format!("{key} requires consistently space-indented rows"));
    }
    Ok(lines.iter().map(|line| &line[indent..]).collect())
}

fn parse_list(key: &str, lines: Option<&[&str]>) -> Result<Vec<String>, String> {
    let Some(lines) = lines else {
        return Ok(Vec::new());
    };
    let values = if lines.len() == 1 && lines[0].starts_with('[') && lines[0].ends_with(']') {
        let body = &lines[0][1..lines[0].len() - 1];
        if body.trim().is_empty() {
            return Ok(Vec::new());
        }
        body.split(',').map(str::trim).collect::<Vec<_>>()
    } else if lines[0].is_empty() && lines.len() > 1 {
        indented_rows(key, &lines[1..])?
            .into_iter()
            .map(|line| {
                line.strip_prefix("- ")
                    .map(str::trim)
                    .ok_or_else(|| format!("{key} requires an indented dash list"))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        return Err(format!("{key} requires a bracket sequence or dash list"));
    };
    if values.iter().any(|value| !bare(value)) {
        return Err(format!("{key} contains an empty or unsupported scalar"));
    }
    Ok(values.into_iter().map(str::to_owned).collect())
}

fn parse_negative(lines: Option<&[&str]>) -> Result<Option<NegativeExpectation>, String> {
    let Some(lines) = lines else {
        return Ok(None);
    };
    if !lines[0].is_empty() {
        return Err("negative requires an indented phase/type mapping".into());
    }
    let mut fields = BTreeMap::new();
    for line in indented_rows("negative", &lines[1..])? {
        let (key, value) = line
            .split_once(':')
            .ok_or("negative requires indented phase/type fields")?;
        let value = value.trim();
        if !matches!(key, "phase" | "type") || !bare(value) || fields.insert(key, value).is_some() {
            return Err(format!("negative has an invalid or duplicate {key} field"));
        }
    }
    let phase = fields.get("phase").ok_or("negative is missing phase")?;
    let error_type = fields.get("type").ok_or("negative is missing type")?;
    Ok(Some(NegativeExpectation {
        phase: NegativePhase::parse(phase)?,
        error_type: (*error_type).to_owned(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_list_forms_preserve_include_order_and_empty_flags() {
        let metadata = parse("/*---\r\nflags: []\r\nfeatures:\r\n  - class\r\n  - class-fields-private\r\nincludes: [b.js, a.js, b.js]\r\n---*/", "fixture.js").unwrap();
        assert!(metadata.flags.is_empty());
        assert_eq!(
            metadata.features,
            BTreeSet::from(["class".into(), "class-fields-private".into()])
        );
        assert_eq!(metadata.includes, ["b.js", "a.js", "b.js"]);
        assert!(parse("ordinary_source();", "fixture.js")
            .unwrap()
            .negative
            .is_none());
    }

    #[test]
    fn malformed_consumed_metadata_cannot_become_a_positive_case() {
        for body in [
            "flags: [raw]\nflags: []",
            "flags: [raw,,async]",
            "flags: ['raw']",
            "flags: [raw] # comment",
            "flags:\n raw",
            "features:",
            "negative:",
            "negative:\n  type: SyntaxError",
            "negative:\n  phase: parse",
            "negative:\n  phase: parse\n  phase: early\n  type: SyntaxError",
            "negative:\n  phase: parse\n  type: SyntaxError\n  extra: ignored",
            "negative:\n  phase: run-time\n  type: SyntaxError",
            "negative:\n    phase: parse\n  type: SyntaxError",
            "flags:\n  - raw\n    - async",
            "negative:\n\tphase: parse\n\ttype: SyntaxError",
        ] {
            let error = parse(&format!("/*---\n{body}\n---*/"), "bad.js").unwrap_err();
            assert!(error.contains("bad.js"), "{error}");
        }
        for source in [
            "/*---\nflags: [raw]",
            "---*/",
            "---*//*---",
            "/*---\n---*/\n/*---\n---*/",
        ] {
            assert!(parse(source, "bad.js").is_err(), "{source}");
        }
    }

    #[test]
    fn negative_requires_both_fields_for_every_closed_phase() {
        for phase in ["parse", "early", "resolution", "runtime"] {
            for indent in ["  ", "    "] {
                let metadata = parse(
                &format!("/*---\nnegative:\n{indent}type: SyntaxError\n{indent}phase: {phase}\n---*/"),
                "negative.js",
            )
            .unwrap();
                let negative = metadata.negative.unwrap();
                assert_eq!(negative.phase.as_str(), phase);
                assert_eq!(negative.error_type, "SyntaxError");
            }
        }
    }
}
