use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn try_lower_static_json_parse_reviver(
        &mut self,
        function_id: &FunctionId,
        args: &[TypedExpr],
    ) -> Option<ExprIr> {
        if StandardBuiltinId::from_function_id(function_id)? != StandardBuiltinId::JsonParse {
            return None;
        }
        if args.len() != 2 {
            return None;
        }
        if !args[1]
            .possible_kinds
            .is_subset_of(KindSet::from_kind(ValueKind::Function))
        {
            return None;
        }
        let input = self.static_json_parse_input(&args[0])?;
        let parsed_value = JsonStaticParser::new(&input).parse()?;
        if self.known_json_parse_reviver_targets(args).is_empty() {
            return None;
        }
        Some(ExprIr::JsonParseStaticReviver {
            value: parsed_value,
            reviver: Box::new(args[1].clone()),
        })
    }

    fn static_json_parse_input(&self, arg: &TypedExpr) -> Option<String> {
        match &arg.expr {
            ExprIr::String(input) => Some(input.clone()),
            ExprIr::Identifier(name) | ExprIr::GlobalPropertyRead { name } => {
                self.static_string_bindings.get(name).cloned()
            }
            _ => None,
        }
    }
}

struct JsonStaticParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> JsonStaticParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse(mut self) -> Option<JsonStaticValueIr> {
        let value = self.parse_value()?;
        self.skip_ws();
        (self.index == self.input.len()).then_some(value)
    }

    fn parse_value(&mut self) -> Option<JsonStaticValueIr> {
        self.skip_ws();
        let start = self.index;
        match self.peek_byte()? {
            b'n' => {
                self.consume_keyword("null")?;
                Some(JsonStaticValueIr::Null {
                    source: self.input[start..self.index].to_string(),
                })
            }
            b't' => {
                self.consume_keyword("true")?;
                Some(JsonStaticValueIr::Boolean {
                    value: true,
                    source: self.input[start..self.index].to_string(),
                })
            }
            b'f' => {
                self.consume_keyword("false")?;
                Some(JsonStaticValueIr::Boolean {
                    value: false,
                    source: self.input[start..self.index].to_string(),
                })
            }
            b'"' => {
                let (value, source) = self.parse_string_literal()?;
                Some(JsonStaticValueIr::String { value, source })
            }
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => {
                self.parse_number()?;
                let source = self.input[start..self.index].to_string();
                let number = source.parse::<f64>().ok()?;
                Some(JsonStaticValueIr::Number {
                    bits: number.to_bits(),
                    source,
                })
            }
            _ => None,
        }
    }

    fn parse_array(&mut self) -> Option<JsonStaticValueIr> {
        self.consume_byte(b'[')?;
        self.skip_ws();
        let mut values = Vec::new();
        if self.consume_byte(b']').is_some() {
            return Some(JsonStaticValueIr::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_byte(b']').is_some() {
                break;
            }
            self.consume_byte(b',')?;
        }
        Some(JsonStaticValueIr::Array(values))
    }

    fn parse_object(&mut self) -> Option<JsonStaticValueIr> {
        self.consume_byte(b'{')?;
        self.skip_ws();
        let mut properties: Vec<(String, JsonStaticValueIr)> = Vec::new();
        if self.consume_byte(b'}').is_some() {
            return Some(JsonStaticValueIr::Object(properties));
        }
        loop {
            self.skip_ws();
            let (key, _) = self.parse_string_literal()?;
            self.skip_ws();
            self.consume_byte(b':')?;
            let value = self.parse_value()?;
            if let Some((_, existing_value)) = properties
                .iter_mut()
                .find(|(existing_key, _)| existing_key == &key)
            {
                *existing_value = value;
            } else {
                properties.push((key, value));
            }
            self.skip_ws();
            if self.consume_byte(b'}').is_some() {
                break;
            }
            self.consume_byte(b',')?;
        }
        Some(JsonStaticValueIr::Object(properties))
    }

    fn parse_string_literal(&mut self) -> Option<(String, String)> {
        let start = self.index;
        self.consume_byte(b'"')?;
        let mut escaped = false;
        while self.index < self.input.len() {
            let byte = self.input.as_bytes()[self.index];
            self.index += 1;
            if escaped {
                escaped = false;
                continue;
            }
            if byte == b'\\' {
                escaped = true;
                continue;
            }
            if byte == b'"' {
                let source = self.input[start..self.index].to_string();
                let value = serde_json::from_str::<String>(&source).ok()?;
                return Some((value, source));
            }
            if byte < 0x20 {
                return None;
            }
        }
        None
    }

    fn parse_number(&mut self) -> Option<()> {
        if self.consume_byte(b'-').is_some() && self.index >= self.input.len() {
            return None;
        }
        match self.peek_byte()? {
            b'0' => {
                self.index += 1;
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return None;
                }
            }
            b'1'..=b'9' => {
                self.index += 1;
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.index += 1;
                }
            }
            _ => return None,
        }
        if self.consume_byte(b'.').is_some() {
            let first = self.peek_byte()?;
            if !first.is_ascii_digit() {
                return None;
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            let first = self.peek_byte()?;
            if !first.is_ascii_digit() {
                return None;
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        Some(())
    }

    fn consume_keyword(&mut self, keyword: &str) -> Option<()> {
        self.input[self.index..].starts_with(keyword).then(|| {
            self.index += keyword.len();
        })
    }

    fn consume_byte(&mut self, byte: u8) -> Option<()> {
        (self.peek_byte()? == byte).then(|| {
            self.index += 1;
        })
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.index).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }
}
