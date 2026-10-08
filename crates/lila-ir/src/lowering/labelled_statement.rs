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

        let async_entry_state = self.plain_async_entry_state();
        let lowered = self.lower_statement(base_statement);

        for _ in 0..labels.len() {
            self.labels.pop();
        }

        // A matching break commits this region's exit even if it bypasses all awaits.
        let async_plan = match (
            label_kind,
            async_entry_state,
            self.plain_async_entry_state(),
        ) {
            (LabelTargetKind::Loop, _, _) | (LabelTargetKind::Breakable, None, None) => None,
            (LabelTargetKind::Breakable, Some(entry), Some(exit)) if entry == exit => None,
            (LabelTargetKind::Breakable, Some(entry), Some(body_exit)) => {
                let plan = match AsyncFunctionLabelledPlanIr::new(entry, body_exit) {
                    Ok(plan) => plan,
                    Err(error) => {
                        self.unsupported_with_message(format!(
                            "unsupported in lila wasm-aot: invalid labelled continuation region: {error:?}",
                        ));
                        return (StatementIr::Empty, ValueKind::Undefined);
                    }
                };
                self.current_async_resume_state = Some(plan.exit_state());
                Some(plan)
            }
            (LabelTargetKind::Breakable, Some(_), None)
            | (LabelTargetKind::Breakable, None, Some(_)) => {
                self.unsupported("labelled region lost its plain async continuation owner");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        };

        (
            StatementIr::Labelled {
                labels,
                statement: Box::new(lowered.0),
                async_plan,
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
                LabelledItem::FunctionDeclaration(_) => return None,
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
