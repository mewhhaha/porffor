use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_labelled(&mut self, labelled: &AstLabelled) -> (StatementIr, ValueKind) {
        if let Some(function) = labelled_function_declaration(labelled) {
            return (
                self.lower_function_declaration(function),
                ValueKind::Undefined,
            );
        }

        let Some((labels, label_kind, base_statement)) = self.collect_labels(labelled) else {
            self.unsupported("label on function declaration");
            return (StatementIr::Empty, ValueKind::Undefined);
        };

        for label in &labels {
            self.labels.push(ActiveLabel {
                name: label.clone(),
                kind: label_kind,
            });
        }

        let lowered = self.lower_statement(base_statement);

        for _ in 0..labels.len() {
            self.labels.pop();
        }

        (
            StatementIr::Labelled {
                labels,
                statement: Box::new(lowered.0),
            },
            lowered.1,
        )
    }

    fn collect_labels<'b>(
        &self,
        labelled: &'b AstLabelled,
    ) -> Option<(Vec<String>, LabelTargetKind, &'b Statement)> {
        let mut labels = vec![self.interner.resolve_expect(labelled.label()).to_string()];
        let mut item = labelled.item();

        loop {
            match item {
                LabelledItem::Statement(Statement::Labelled(next)) => {
                    labels.push(self.interner.resolve_expect(next.label()).to_string());
                    item = next.item();
                }
                LabelledItem::Statement(statement) => {
                    return Some((labels, Self::label_target_kind(statement), statement));
                }
                LabelledItem::FunctionDeclaration(_) => {
                    return None;
                }
            }
        }
    }

    fn label_target_kind(statement: &Statement) -> LabelTargetKind {
        match statement {
            Statement::WhileLoop(_)
            | Statement::DoWhileLoop(_)
            | Statement::ForLoop(_)
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_) => LabelTargetKind::Loop,
            _ => LabelTargetKind::Breakable,
        }
    }
}
