use super::*;

#[must_use = "a prepared static JSON.parse reviver must be emitted or rejected"]
pub(super) struct PreparedStaticJsonParseReviver {
    parsed_value: JsonStaticValueIr,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn prepare_static_json_parse_reviver(
        &self,
        function_id: &FunctionId,
        arguments: &[Expression],
    ) -> Option<PreparedStaticJsonParseReviver> {
        if StandardBuiltinId::from_function_id(function_id)? != StandardBuiltinId::JsonParse {
            return None;
        }
        if arguments.len() != 2 {
            return None;
        }
        if arguments
            .iter()
            .any(|argument| matches!(argument, Expression::Spread(_)))
        {
            return None;
        }
        let input = self.static_string_expression(&arguments[0]).or_else(|| {
            if !self.with_environment_chain.is_empty() {
                return None;
            }
            let Expression::Identifier(identifier) = Self::unwrap_parenthesized_expr(&arguments[0])
            else {
                return None;
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            let binding = self.lookup_binding(&name)?;
            if binding.initialization != Initialization::Initialized
                || binding.possible_kinds != KindSet::from_kind(ValueKind::String)
            {
                return None;
            }
            let mutable_binding_may_be_reobserved = binding.mode != BindingMode::Const
                && (self.loop_depth > 0
                    || self
                        .captured_binding_positions
                        .contains_key(&binding.storage_name));
            if mutable_binding_may_be_reobserved {
                return None;
            }
            self.static_string_bindings.get(&binding).cloned()
        })?;
        let parsed_value = JsonStaticParser::new(&input).parse()?;
        Some(PreparedStaticJsonParseReviver { parsed_value })
    }

    pub(super) fn finish_static_json_parse_reviver(
        &self,
        prepared: PreparedStaticJsonParseReviver,
        callee: &TypedExpr,
        arguments: &[TypedExpr],
    ) -> Option<TypedExpr> {
        let [input, reviver] = arguments else {
            panic!(
                "prepared static JSON.parse expected 2 lowered arguments, got {}",
                arguments.len()
            );
        };
        if input.possible_kinds != KindSet::from_kind(ValueKind::String) {
            return None;
        }
        if !reviver
            .possible_kinds
            .is_subset_of(KindSet::from_kind(ValueKind::Function))
        {
            return None;
        }
        if self.known_json_parse_reviver_targets(arguments).is_empty() {
            return None;
        }
        Some(TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            ExprIr::JsonParseStaticReviver {
                callee: Box::new(callee.clone()),
                input: Box::new(input.clone()),
                value: prepared.parsed_value,
                reviver: Box::new(reviver.clone()),
            },
        ))
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
