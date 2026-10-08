//! Original callable text is recovered through compiler-owned source edits.

use crate::*;

impl super::synchronous_definition::ModuleExecutionAnalysis {
    pub(crate) fn class_method_to_string_representation(
        &self,
        method: &ClassMethodDefinition,
        source_text: &str,
    ) -> CallableToStringRepresentation {
        CallableToStringRepresentation::ExactSource(
            self.original_callable_source(method.linear_span())
                .map(str::to_owned)
                .unwrap_or_else(|| class_method_source_slice(method, source_text)),
        )
    }
}

/// A rewritten unit carries the original byte boundary for every surviving
/// boundary. Inserted interiors have no source authority. Callable boundaries
/// must resolve to original UTF-8 boundaries before a slice can be published.
#[derive(Debug)]
pub(super) struct OriginalUnitSource {
    original: String,
    text: String,
    boundaries: Vec<Option<usize>>,
}

impl OriginalUnitSource {
    pub(super) fn new(source: &str) -> Self {
        Self {
            original: source.into(),
            text: source.into(),
            boundaries: (0..=source.len()).map(Some).collect(),
        }
    }

    pub(super) fn text(&self) -> &str {
        &self.text
    }

    pub(super) fn into_text(self) -> String {
        self.text
    }

    pub(super) fn stable_rewrite(mut self, text: String) -> Self {
        assert_eq!(
            text.len(),
            self.text.len(),
            "this rewrite preserves byte offsets"
        );
        self.text = text;
        self
    }

    pub(super) fn replace(&mut self, start: usize, end: usize, replacement: &str) {
        assert!(
            !replacement.is_empty(),
            "source edits retain both endpoint authorities"
        );
        assert!(
            self.text.get(start..end).is_some(),
            "source edit is a UTF-8 range"
        );
        let original_end = self.boundaries[end];
        let interiors = std::iter::repeat_n(None, replacement.len() - 1);
        self.boundaries.splice(
            start + 1..end + 1,
            interiors.chain(std::iter::once(original_end)),
        );
        self.text.replace_range(start..end, replacement);
        assert_eq!(self.boundaries.len(), self.text.len() + 1);
    }

    pub(super) fn wrap(mut self, prefix: &str, suffix: &str) -> Self {
        self.boundaries
            .splice(0..0, std::iter::repeat_n(None, prefix.len()));
        self.boundaries
            .extend(std::iter::repeat_n(None, suffix.len()));
        self.text = format!("{prefix}{}{suffix}", self.text);
        assert_eq!(self.boundaries.len(), self.text.len() + 1);
        self
    }

    fn original_slice(&self, span: boa_ast::LinearSpan) -> Option<String> {
        let bytes = source_byte_range_from_utf16_span(&self.text, span);
        let start = self.boundaries[bytes.start]?;
        let end = self.boundaries[bytes.end]?;
        Some(
            self.original
                .get(start..end)
                .expect("callable origins are original UTF-8 boundaries")
                .into(),
        )
    }
}

#[derive(Debug)]
struct OriginalBody {
    start: usize,
    end: usize,
    source: OriginalUnitSource,
}

#[derive(Debug, Default)]
pub(super) struct CallableSourceDefinitions(Vec<OriginalBody>);

impl CallableSourceDefinitions {
    pub(super) fn record_body(&mut self, source: OriginalUnitSource, preceding: &str) -> &str {
        let start = preceding.encode_utf16().count();
        let end = start + source.text().encode_utf16().count();
        self.0.push(OriginalBody { start, end, source });
        self.0.last().expect("body was recorded").source.text()
    }

    fn original_source(&self, span: boa_ast::LinearSpan) -> Option<String> {
        let start = span.start().pos();
        let end = span.end().pos();
        let body = self
            .0
            .iter()
            .find(|body| body.start <= start && end <= body.end)?;
        let relative = boa_ast::LinearSpan::new(
            boa_ast::LinearPosition::new(start - body.start),
            boa_ast::LinearPosition::new(end - body.start),
        );
        body.source.original_slice(relative)
    }

    pub(super) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        struct Callables<'a, 'b> {
            definitions: &'b CallableSourceDefinitions,
            analysis: &'b Analysis<'a>,
            functions: BTreeMap<FunctionId, String>,
            spans: BTreeMap<(usize, usize), String>,
        }
        impl Callables<'_, '_> {
            fn record(&mut self, span: boa_ast::LinearSpan, function: Option<FunctionId>) {
                let Some(source) = self.definitions.original_source(span) else {
                    return;
                };
                self.spans
                    .insert((span.start().pos(), span.end().pos()), source.clone());
                if let Some(function) = function {
                    self.functions.insert(function, source);
                }
            }
        }
        macro_rules! declaration {
            ($visit:ident, $kind:ty, $key:ident) => {
                fn $visit(&mut self, function: &'a $kind) -> ControlFlow<Self::BreakTy> {
                    let id = self
                        .analysis
                        .function_declaration_ids
                        .get(&$key(function))
                        .cloned();
                    self.record(function.linear_span(), id);
                    function.visit_with(self)
                }
            };
        }
        macro_rules! expression {
            ($visit:ident, $kind:ty, $key:ident) => {
                fn $visit(&mut self, function: &'a $kind) -> ControlFlow<Self::BreakTy> {
                    let id = self
                        .analysis
                        .function_expr_ids
                        .get(&$key(function))
                        .cloned();
                    self.record(function.linear_span(), id);
                    function.visit_with(self)
                }
            };
        }
        impl<'a> Visitor<'a> for Callables<'a, '_> {
            type BreakTy = core::convert::Infallible;

            declaration!(
                visit_function_declaration,
                FunctionDeclaration,
                function_declaration_key
            );
            declaration!(
                visit_generator_declaration,
                GeneratorDeclaration,
                generator_declaration_key
            );
            declaration!(
                visit_async_function_declaration,
                AsyncFunctionDeclaration,
                async_function_declaration_key
            );
            declaration!(
                visit_async_generator_declaration,
                AsyncGeneratorDeclaration,
                async_generator_declaration_key
            );
            expression!(
                visit_generator_expression,
                GeneratorExpression,
                generator_expression_key
            );
            expression!(
                visit_async_function_expression,
                AsyncFunctionExpression,
                async_function_expression_key
            );
            expression!(
                visit_async_generator_expression,
                AsyncGeneratorExpression,
                async_generator_expression_key
            );
            expression!(visit_arrow_function, ArrowFunction, arrow_function_key);
            expression!(
                visit_async_arrow_function,
                AsyncArrowFunction,
                async_arrow_function_key
            );

            fn visit_function_expression(
                &mut self,
                function: &'a FunctionExpression,
            ) -> ControlFlow<Self::BreakTy> {
                if let Some(span) = function.linear_span() {
                    let id = self
                        .analysis
                        .function_expr_ids
                        .get(&function_expression_key(function))
                        .cloned();
                    self.record(span, id);
                }
                function.visit_with(self)
            }

            fn visit_object_method_definition(
                &mut self,
                method: &'a ObjectMethodDefinition,
            ) -> ControlFlow<Self::BreakTy> {
                let id = self
                    .analysis
                    .function_expr_ids
                    .get(&object_method_key(method))
                    .cloned();
                self.record(method.linear_span(), id);
                method.visit_with(self)
            }

            fn visit_class_declaration(
                &mut self,
                class: &'a ClassDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                self.record(class.linear_span(), None);
                class.visit_with(self)
            }

            fn visit_class_expression(
                &mut self,
                class: &'a ClassExpression,
            ) -> ControlFlow<Self::BreakTy> {
                self.record(class.linear_span(), None);
                class.visit_with(self)
            }

            fn visit_class_element(
                &mut self,
                element: &'a ClassElement,
            ) -> ControlFlow<Self::BreakTy> {
                match element {
                    ClassElement::MethodDefinition(method) => {
                        self.record(method.linear_span(), None)
                    }
                    ClassElement::FieldDefinition(_)
                    | ClassElement::AccessorFieldDefinition(_)
                    | ClassElement::StaticFieldDefinition(_)
                    | ClassElement::StaticAccessorFieldDefinition(_)
                    | ClassElement::PrivateFieldDefinition(_)
                    | ClassElement::PrivateStaticFieldDefinition(_)
                    | ClassElement::StaticBlock(_) => {}
                }
                element.visit_with(self)
            }
        }
        let mut callables = Callables {
            definitions: self,
            analysis,
            functions: BTreeMap::new(),
            spans: BTreeMap::new(),
        };
        let _ = script.visit_with(&mut callables);
        let Callables {
            functions, spans, ..
        } = callables;
        analysis
            .module_execution
            .record_original_callable_sources(spans);
        let restore = |function: &mut PendingFunction<'_>| {
            if let Some(source) = functions.get(&function.id) {
                function.to_string_representation =
                    CallableToStringRepresentation::ExactSource(source.clone());
            }
        };
        for function in &mut analysis.script_root_functions {
            restore(function);
        }
        for plan in analysis.function_plans.values_mut() {
            if let Some(source) = functions.get(&plan.id) {
                plan.to_string_representation =
                    CallableToStringRepresentation::ExactSource(source.clone());
            }
            for function in &mut plan.root_functions {
                restore(function);
            }
        }
    }
}
