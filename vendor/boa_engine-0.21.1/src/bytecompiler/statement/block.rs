use crate::bytecompiler::ByteCompiler;
use boa_ast::{
    StatementList, StatementListItem,
    declaration::{Declaration, LexicalDeclaration},
    expression::Expression,
    scope::FunctionScopes,
    function::ClassElement,
    operations::{ContainsSymbol, contains},
    statement::{Block, Statement},
};

fn shift_function_scopes(scopes: &FunctionScopes, delta: u32) {
    let function_scope = scopes.function_scope();
    function_scope.set_index(function_scope.scope_index() + delta);

    if let Some(scope) = scopes.parameters_eval_scope() {
        scope.set_index(scope.scope_index() + delta);
    }

    if let Some(scope) = scopes.parameters_scope() {
        scope.set_index(scope.scope_index() + delta);
    }

    if let Some(scope) = scopes.lexical_scope() {
        scope.set_index(scope.scope_index() + delta);
    }
}

fn expr_contains_direct_eval(expr: &Expression) -> bool {
    match expr {
        Expression::ArrowFunction(function) => function.contains_direct_eval(),
        Expression::AsyncArrowFunction(function) => function.contains_direct_eval(),
        Expression::FunctionExpression(function) => function.contains_direct_eval(),
        Expression::GeneratorExpression(function) => function.contains_direct_eval(),
        Expression::AsyncFunctionExpression(function) => function.contains_direct_eval(),
        Expression::AsyncGeneratorExpression(function) => function.contains_direct_eval(),
        Expression::ClassExpression(class) => class
            .elements()
            .iter()
            .any(class_element_contains_direct_eval),
        Expression::Parenthesized(expr) => expr_contains_direct_eval(expr.expression()),
        _ => contains(expr, ContainsSymbol::DirectEval),
    }
}

fn class_element_contains_direct_eval(element: &ClassElement) -> bool {
    match element {
        ClassElement::FieldDefinition(field)
        | ClassElement::AccessorFieldDefinition(field)
        | ClassElement::StaticFieldDefinition(field)
        | ClassElement::StaticAccessorFieldDefinition(field) => field
            .initializer()
            .is_some_and(expr_contains_direct_eval),
        ClassElement::PrivateFieldDefinition(field)
        | ClassElement::PrivateStaticFieldDefinition(field) => field
            .initializer()
            .is_some_and(expr_contains_direct_eval),
        _ => false,
    }
}

fn shift_expression_scopes(expr: &Expression, delta: u32) {
    match expr {
        Expression::ClassExpression(class) => {
            if let Some(scope) = class.name_scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            for element in class.elements() {
                shift_class_element_scopes(element, delta);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(scope) = function.name_scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::ArrowFunction(function) => {
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::AsyncArrowFunction(function) => {
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::GeneratorExpression(function) => {
            if let Some(scope) = function.name_scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::AsyncFunctionExpression(function) => {
            if let Some(scope) = function.name_scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::AsyncGeneratorExpression(function) => {
            if let Some(scope) = function.name_scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            shift_function_scopes(function.scopes(), delta);
            shift_statement_list_scopes(function.body().statement_list(), delta);
        }
        Expression::Parenthesized(expr) => shift_expression_scopes(expr.expression(), delta),
        _ => {}
    }
}

fn shift_class_element_scopes(element: &ClassElement, delta: u32) {
    match element {
        ClassElement::FieldDefinition(field)
        | ClassElement::AccessorFieldDefinition(field)
        | ClassElement::StaticFieldDefinition(field)
        | ClassElement::StaticAccessorFieldDefinition(field) => {
            field.scope().set_index(field.scope().scope_index() + delta);
            if let Some(expr) = field.initializer() {
                shift_expression_scopes(expr, delta);
            }
        }
        ClassElement::PrivateFieldDefinition(field)
        | ClassElement::PrivateStaticFieldDefinition(field) => {
            field.scope().set_index(field.scope().scope_index() + delta);
            if let Some(expr) = field.initializer() {
                shift_expression_scopes(expr, delta);
            }
        }
        _ => {}
    }
}

fn class_expr_contains_direct_eval(expr: &Expression) -> bool {
    match expr {
        Expression::ClassExpression(class) => class
            .elements()
            .iter()
            .any(class_element_contains_direct_eval),
        _ => false,
    }
}

fn lexical_decl_contains_class_direct_eval(decl: &LexicalDeclaration) -> bool {
    decl.variable_list()
        .as_ref()
        .iter()
        .filter_map(|var| var.init())
        .any(class_expr_contains_direct_eval)
}

fn declaration_needs_block_environment(decl: &Declaration) -> bool {
    match decl {
        Declaration::ClassDeclaration(class) => class
            .elements()
            .iter()
            .any(class_element_contains_direct_eval),
        Declaration::Lexical(decl) => lexical_decl_contains_class_direct_eval(decl),
        _ => false,
    }
}

fn shift_declaration_scopes(decl: &Declaration, delta: u32) {
    match decl {
        Declaration::ClassDeclaration(class) => {
            class.name_scope().set_index(class.name_scope().scope_index() + delta);
            for element in class.elements() {
                shift_class_element_scopes(element, delta);
            }
        }
        Declaration::Lexical(decl) => {
            for var in decl.variable_list().as_ref() {
                if let Some(expr) = var.init() {
                    shift_expression_scopes(expr, delta);
                }
            }
        }
        _ => {}
    }
}

fn statement_needs_block_environment(stmt: &Statement) -> bool {
    match stmt {
        Statement::Block(block) => statement_list_needs_block_environment(block.statement_list()),
        _ => false,
    }
}

fn shift_statement_scopes(stmt: &Statement, delta: u32) {
    match stmt {
        Statement::Block(block) => {
            if let Some(scope) = block.scope() {
                scope.set_index(scope.scope_index() + delta);
            }
            shift_statement_list_scopes(block.statement_list(), delta);
        }
        Statement::Expression(expr) => shift_expression_scopes(expr, delta),
        _ => {}
    }
}

pub(crate) fn statement_list_needs_block_environment(list: &StatementList) -> bool {
    list.statements().iter().any(|item| match item {
        StatementListItem::Statement(stmt) => statement_needs_block_environment(stmt),
        StatementListItem::Declaration(decl) => declaration_needs_block_environment(decl),
    })
}

pub(crate) fn shift_statement_list_scopes(list: &StatementList, delta: u32) {
    for item in list.statements() {
        match item {
            StatementListItem::Statement(stmt) => shift_statement_scopes(stmt, delta),
            StatementListItem::Declaration(decl) => shift_declaration_scopes(decl, delta),
        }
    }
}

impl ByteCompiler<'_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        if statement_list_needs_block_environment(block.statement_list())
            && let Some(scope) = block.scope()
        {
            if scope.scope_index() <= self.lexical_scope.scope_index() {
                let delta = self.lexical_scope.scope_index() + 1 - scope.scope_index();
                scope.set_index(scope.scope_index() + delta);
                shift_statement_list_scopes(block.statement_list(), delta);
            }
            scope.set_needs_environment();
        }
        let scope = self.push_declarative_scope(block.scope());
        self.block_declaration_instantiation(block);
        self.compile_statement_list(block.statement_list(), use_expr, true);
        self.pop_declarative_scope(scope);
    }
}
