const HEADING: &str = "## Current Status";
const START: &str = "<!-- scriptw-status:start -->";
const END: &str = "<!-- scriptw-status:end -->";

/// Replace only the uniquely owned section. A damaged marker must never widen
/// the replacement into adjacent hand-written documentation.
pub(super) fn replace_status_section(raw: &str, replacement: &str) -> Result<String, String> {
    let single = |needle: &str| {
        let mut matches = raw.match_indices(needle);
        let first = matches.next().map(|(offset, _)| offset);
        match (first, matches.next()) {
            (Some(offset), None) => Ok(offset),
            _ => Err(format!("expected exactly one {needle}")),
        }
    };
    let heading = single(HEADING)?;
    let start = single(START)?;
    let end = single(END)?;
    if (heading != 0 && !raw[..heading].ends_with('\n'))
        || heading + HEADING.len() > start
        || !raw[heading + HEADING.len()..start].trim().is_empty()
        || start + START.len() > end
        || raw[start..end].contains("\n## ")
    {
        return Err(
            "status markers must form one ordered section directly below its heading".into(),
        );
    }
    Ok(format!(
        "{}{}{}",
        &raw[..heading],
        replacement,
        &raw[end + END.len()..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_preserves_every_byte_outside_the_owned_section() {
        let old = format!(
            "# Example 🐸\r\n\r\n{HEADING}\r\n{START}\r\nold\r\n{END}\r\n\r\n## Design\r\nkeep\r\n"
        );
        let replacement = format!("{HEADING}\n{START}\nnew\n{END}");
        assert_eq!(
            replace_status_section(&old, &replacement).unwrap(),
            format!("# Example 🐸\r\n\r\n{replacement}\r\n\r\n## Design\r\nkeep\r\n")
        );
    }

    #[test]
    fn malformed_markers_never_select_a_larger_section() {
        for raw in [
            format!("{HEADING}\nold"),
            format!("{HEADING}\n{START}\nold"),
            format!("{HEADING}\nold\n{END}"),
            format!("{HEADING}\n{END}\n{START}"),
            format!("{HEADING}\n{START}\n{START}\n{END}"),
            format!("{HEADING}\n{START}\n{END}\n{END}"),
            format!("{HEADING}\n{START}\n## Design\n{END}"),
            format!("{HEADING}\nother text\n{START}\n{END}"),
            format!("#{HEADING}\n{START}\n{END}"),
            format!("{HEADING}\n{START}\n{END}\n{HEADING}"),
        ] {
            assert!(
                replace_status_section(&raw, "replacement").is_err(),
                "{raw}"
            );
        }
    }
}
