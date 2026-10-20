use super::{FieldToken, LiteralToken, PlaceholderToken, Token};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

// The profile contains over 140,000 tokens. Dispatch objects directly instead
// of buffering each one and trying all three untagged variants. Keep the old
// derived representation for sequence inputs, whose variant order matters.
#[derive(Deserialize)]
#[serde(untagged)]
enum SequenceToken {
    Literal(LiteralToken),
    Field(FieldToken),
    Placeholder(PlaceholderToken),
}

impl From<SequenceToken> for Token {
    fn from(value: SequenceToken) -> Self {
        match value {
            SequenceToken::Literal(value) => Self::Literal(value),
            SequenceToken::Field(value) => Self::Field(value),
            SequenceToken::Placeholder(value) => Self::Placeholder(value),
        }
    }
}

impl<'de> Deserialize<'de> for Token {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Key {
            Literal,
            Field,
            Width,
            Placeholder,
        }

        struct TokenVisitor;

        impl<'de> Visitor<'de> for TokenVisitor {
            type Value = Token;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a literal, field, or placeholder DateTime token")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Token, M::Error> {
                let (mut literal, mut field, mut width, mut placeholder) = (None, None, None, None);
                while let Some(key) = map.next_key::<Key>()? {
                    match key {
                        Key::Literal => {
                            if literal.is_some() {
                                return Err(de::Error::duplicate_field("literal"));
                            }
                            literal = Some(map.next_value::<String>()?);
                        }
                        Key::Field => {
                            if field.is_some() {
                                return Err(de::Error::duplicate_field("field"));
                            }
                            field = Some(map.next_value::<char>()?);
                        }
                        Key::Width => {
                            if width.is_some() {
                                return Err(de::Error::duplicate_field("width"));
                            }
                            width = Some(map.next_value::<u8>()?);
                        }
                        Key::Placeholder => {
                            if placeholder.is_some() {
                                return Err(de::Error::duplicate_field("placeholder"));
                            }
                            placeholder = Some(map.next_value::<u8>()?);
                        }
                    }
                }
                match (literal, field, width, placeholder) {
                    (Some(literal), None, None, None) => {
                        Ok(Token::Literal(LiteralToken { literal }))
                    }
                    (None, Some(field), Some(width), None) => {
                        Ok(Token::Field(FieldToken { field, width }))
                    }
                    (None, None, None, Some(placeholder)) => {
                        Ok(Token::Placeholder(PlaceholderToken { placeholder }))
                    }
                    _ => Err(de::Error::custom(
                        "token fields do not form exactly one variant",
                    )),
                }
            }

            fn visit_seq<S: SeqAccess<'de>>(self, sequence: S) -> Result<Token, S::Error> {
                SequenceToken::deserialize(de::value::SeqAccessDeserializer::new(sequence))
                    .map(Token::from)
            }
        }

        deserializer.deserialize_any(TokenVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Value {
        Literal(String),
        Field(char, u8),
        Placeholder(u8),
    }

    fn value(token: Token) -> Value {
        match token {
            Token::Literal(token) => Value::Literal(token.literal),
            Token::Field(token) => Value::Field(token.field, token.width),
            Token::Placeholder(token) => Value::Placeholder(token.placeholder),
        }
    }

    fn parity(source: &str) {
        let expected = serde_json::from_str::<SequenceToken>(source)
            .ok()
            .map(Token::from)
            .map(value);
        let actual = serde_json::from_str::<Token>(source).ok().map(value);
        assert_eq!(actual, expected, "token input: {source}");
    }

    #[test]
    fn token_decoder_preserves_derived_scalar_sequence_and_key_grammar() {
        for source in [
            "{}",
            "[]",
            "null",
            "true",
            "0",
            "\"x\"",
            r#"{"literal":""}"#,
            r#"{"literal":"\u0000\n\"\\😀"}"#,
            r#"{"field":"y","width":0}"#,
            r#"{"width":255,"field":"😀"}"#,
            r#"{"field":"\uD83D\uDE00","width":2}"#,
            r#"{"field":"\uD800","width":2}"#,
            r#"{"field":"\u0000","width":2}"#,
            r#"{"field":"é","width":2}"#,
            r#"{"field":"y","width":-0}"#,
            r#"{"field":"y","width":1.0}"#,
            r#"{"field":"y","width":1e0}"#,
            r#"{"placeholder":18446744073709551616}"#,
            r#"{"literal":"x","\u006citeral":"y"}"#,
            r#"{"field":"y","width":1,"\u0077idth":2}"#,
            r#"{"placeholder":0,"place\u0068older":1}"#,
            r#"{"literal":null,"placeholder":0}"#,
            r#"{"literal":"x","field":"y","width":1}"#,
            r#"["x"]"#,
            r#"["y",1]"#,
            "[0]",
            "[255]",
            "[-0]",
            "[1.0]",
            "[1e0]",
        ] {
            parity(source);
        }

        // Repeated keys, mixed variants, wrong scalar types and extra elements
        // must retain the old rejection behavior in every ordering.
        let entries: Vec<_> = ["literal", "field", "width", "placeholder", "extra"]
            .into_iter()
            .flat_map(|key| [r#""x""#, "1", "null", r#""xy""#].map(|v| format!("\"{key}\":{v}")))
            .collect();
        for a in &entries {
            parity(&format!("{{{a}}}"));
            for b in &entries {
                parity(&format!("{{{a},{b}}}"));
                for c in &entries {
                    parity(&format!("{{{a},{b},{c}}}"));
                }
            }
        }
        let elements = [
            r#""x""#, r#""xy""#, r#""""#, "0", "255", "256", "null", "true", "[]", "{}",
        ];
        for a in elements {
            parity(&format!("[{a}]"));
            for b in elements {
                parity(&format!("[{a},{b}]"));
                for c in elements {
                    parity(&format!("[{a},{b},{c}]"));
                }
            }
        }
    }

    #[test]
    fn every_embedded_profile_token_matches_the_derived_decoder() {
        fn visit(node: &serde_json::Value, count: &mut usize) {
            match node {
                serde_json::Value::Object(fields) => {
                    for (key, child) in fields {
                        if key == "tokens" {
                            for token in child.as_array().expect("pattern token array") {
                                parity(&token.to_string());
                                *count += 1;
                            }
                        } else {
                            visit(child, count);
                        }
                    }
                }
                serde_json::Value::Array(items) => {
                    for item in items {
                        visit(item, count);
                    }
                }
                _ => {}
            }
        }
        let profile = serde_json::from_str(include_str!("../generated/profile.json")).unwrap();
        let mut count = 0;
        visit(&profile, &mut count);
        assert!(count > 140_000, "embedded profile tokens were not visited");
    }
}
