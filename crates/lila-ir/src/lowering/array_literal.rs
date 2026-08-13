//! Array-initializer lowering.
//!
//! Spread-bearing literals use the typed 13.2.4.1 ArrayAccumulation IR instead
//! of observable `concat`/`Array.from` calls. Generator literals use the same
//! IR with suspension-owned array and logical-index slots, so every prefix is
//! committed before the suspension that follows it.

use super::*;

impl ScriptLowerer<'_> {
    fn array_accumulation_result_info() -> ValueInfo {
        ValueInfo {
            kind: ValueKind::Array,
            possible_kinds: KindSet::from_kind(ValueKind::Array),
            // A spread can execute arbitrary iterator code and contribute an
            // arbitrary number and kind of elements. An empty ArrayShape would
            // falsely prove length zero, so the shape is deliberately absent.
            heap_shape: None,
            function_targets: BTreeSet::new(),
        }
    }

    fn array_accumulation_expr(
        target: ArrayAccumulationTargetIr,
        elements: Vec<ArrayAccumulationElementIr>,
    ) -> TypedExpr {
        let accumulation = match target {
            ArrayAccumulationTargetIr::Fresh => ArrayAccumulationIr::fresh(elements),
            ArrayAccumulationTargetIr::SuspensionOwned(slots) => {
                ArrayAccumulationIr::suspension_owned(slots, elements)
            }
        };
        TypedExpr::from_info(
            Self::array_accumulation_result_info(),
            ExprIr::ArrayAccumulation(accumulation),
        )
    }

    fn lower_array_accumulation_element(
        &mut self,
        element: &Expression,
    ) -> ArrayAccumulationElementIr {
        match element {
            Expression::Spread(spread) => ArrayAccumulationElementIr::Spread(ArraySpreadIr {
                value: Box::new(self.lower_expression(spread.target())),
                protocol: ArraySpreadProtocol::ARRAY_ACCUMULATION,
            }),
            expression => ArrayAccumulationElementIr::Value(self.lower_expression(expression)),
        }
    }

    /// Ordinary no-spread literals keep their shaped `ExprIr::ArrayLiteral`
    /// form. The first spread changes the whole initializer to source-ordered
    /// ArrayAccumulation; there is no type-based dense-array shortcut.
    pub(super) fn lower_array_literal(&mut self, array: &ArrayLiteral) -> TypedExpr {
        let has_spread = array
            .as_ref()
            .iter()
            .any(|element| matches!(element, Some(Expression::Spread(_))));

        if has_spread {
            let elements = array
                .as_ref()
                .iter()
                .map(|element| match element {
                    None => ArrayAccumulationElementIr::Elision,
                    Some(element) => self.lower_array_accumulation_element(element),
                })
                .collect();
            return Self::array_accumulation_expr(ArrayAccumulationTargetIr::Fresh, elements);
        }

        let mut elements = Vec::with_capacity(array.as_ref().len());
        let mut shape = ArrayShape::default();
        for element in array.as_ref() {
            let lowered = match element {
                Some(element) => self.lower_expression(element),
                None => TypedExpr::from_info(ValueInfo::undefined(), ExprIr::ArrayHole),
            };
            shape.elements.push(lowered.value_info());
            elements.push(lowered);
        }
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Array,
                possible_kinds: KindSet::from_kind(ValueKind::Array),
                heap_shape: Some(Box::new(HeapShape::Array(shape))),
                function_targets: BTreeSet::new(),
            },
            ExprIr::ArrayLiteral(elements),
        )
    }

    fn alloc_array_accumulator_array_slot(&mut self) -> ArrayAccumulatorArraySlot {
        ArrayAccumulatorArraySlot::new(self.alloc_suspension_owned_binding(
            "generator.array.accumulator.",
            Self::array_accumulation_result_info(),
        ))
    }

    fn alloc_array_accumulator_next_index_slot(&mut self) -> ArrayAccumulatorU64NextIndexSlot {
        ArrayAccumulatorU64NextIndexSlot::new(self.alloc_suspension_owned_binding(
            "generator.array.next_index.",
            ValueInfo::new(ValueKind::Number),
        ))
    }

    fn alloc_array_accumulator_slots(&mut self) -> ArrayAccumulatorSlots {
        let array = self.alloc_array_accumulator_array_slot();
        let next_index = self.alloc_array_accumulator_next_index_slot();
        ArrayAccumulatorSlots::new(array, next_index)
    }

    fn flush_array_accumulation_prefix(
        statements: &mut Vec<StatementIr>,
        slots: &ArrayAccumulatorSlots,
        elements: &mut Vec<ArrayAccumulationElementIr>,
    ) {
        if elements.is_empty() {
            return;
        }
        statements.push(StatementIr::Expression(Self::array_accumulation_expr(
            ArrayAccumulationTargetIr::SuspensionOwned(slots.clone()),
            std::mem::take(elements),
        )));
    }

    /// A generator array literal allocates both pieces of ArrayAccumulation
    /// state before evaluating its first element. Every already-lowered prefix
    /// is flushed before the statements containing the next suspension, and
    /// the final expression returns the same suspension-owned array.
    pub(super) fn lower_staged_generator_array_literal(
        &mut self,
        array: &ArrayLiteral,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let slots = self.alloc_array_accumulator_slots();
        let array_info = Self::array_accumulation_result_info();
        let mut statements = vec![
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: slots.array().as_str().to_string(),
                init: TypedExpr::from_info(array_info, ExprIr::ArrayLiteral(Vec::new())),
            },
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: slots.next_index().as_str().to_string(),
                init: TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Number),
                    ExprIr::Number(0.0f64.to_bits()),
                ),
            },
        ];
        let mut elements = Vec::new();

        for element in array.as_ref() {
            let Some(element) = element else {
                elements.push(ArrayAccumulationElementIr::Elision);
                continue;
            };

            let (expression, spread) = match element {
                Expression::Spread(spread) => (spread.target(), true),
                expression => (expression, false),
            };
            let (nested_statements, value) =
                if contains(expression, ContainsSymbol::YieldExpression) {
                    self.lower_staged_generator_expression(expression)?
                } else {
                    (Vec::new(), self.lower_expression(expression))
                };

            if !nested_statements.is_empty() {
                Self::flush_array_accumulation_prefix(&mut statements, &slots, &mut elements);
                statements.extend(nested_statements);
            }

            elements.push(if spread {
                ArrayAccumulationElementIr::Spread(ArraySpreadIr {
                    value: Box::new(value),
                    protocol: ArraySpreadProtocol::ARRAY_ACCUMULATION,
                })
            } else {
                ArrayAccumulationElementIr::Value(value)
            });
        }

        let result = Self::array_accumulation_expr(
            ArrayAccumulationTargetIr::SuspensionOwned(slots),
            elements,
        );
        Some((statements, result))
    }
}
