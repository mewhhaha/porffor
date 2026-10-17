use super::*;
use std::collections::BTreeSet;
use wasmparser::{
    ConstExpr, Element, ElementItems, ElementKind, ElementSectionReader, ExportSectionReader,
    ExternalKind, GlobalSectionReader, Operator, TableInit, TableSectionReader,
};
#[derive(Default)]
pub(super) struct References(BTreeSet<u32>);
impl References {
    pub fn insert(&mut self, index: u32) {
        self.0.insert(index);
    }
    pub fn operator(&mut self, operator: &Operator<'_>) {
        match operator {
            Operator::Call { function_index }
            | Operator::ReturnCall { function_index }
            | Operator::RefFunc { function_index } => self.insert(*function_index),
            _ => {}
        }
    }
    pub fn expression(
        &mut self,
        expression: &ConstExpr<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        let mut reader = expression.get_operators_reader();
        while !reader.eof() {
            self.operator(&reader.read().map_err(invalid)?);
        }
        Ok(())
    }
    pub fn element(&mut self, element: Element<'_>) -> Result<(), NativeProgramTemplateError> {
        if let ElementKind::Active { offset_expr, .. } = &element.kind {
            self.expression(offset_expr)?;
        }
        match element.items {
            // A plain declarative list only declares ref.func admissibility.
            ElementItems::Functions(_) if matches!(element.kind, ElementKind::Declared) => {}
            ElementItems::Functions(functions) => {
                for function in functions {
                    self.insert(function.map_err(invalid)?);
                }
            }
            ElementItems::Expressions(_, expressions) => {
                for expression in expressions {
                    self.expression(&expression.map_err(invalid)?)?;
                }
            }
        }
        Ok(())
    }
    pub fn tables(
        &mut self,
        reader: TableSectionReader<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        for table in reader {
            if let TableInit::Expr(expression) = table.map_err(invalid)?.init {
                self.expression(&expression)?;
            }
        }
        Ok(())
    }
    pub fn globals(
        &mut self,
        reader: GlobalSectionReader<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        for global in reader {
            self.expression(&global.map_err(invalid)?.init_expr)?;
        }
        Ok(())
    }
    pub fn exports(
        &mut self,
        reader: ExportSectionReader<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        for export in reader {
            let export = export.map_err(invalid)?;
            if export.kind == ExternalKind::Func {
                self.insert(export.index);
            }
        }
        Ok(())
    }
    pub fn elements(
        &mut self,
        reader: ElementSectionReader<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        for element in reader {
            self.element(element.map_err(invalid)?)?;
        }
        Ok(())
    }
    pub fn into_inner(self) -> BTreeSet<u32> {
        self.0
    }
}
