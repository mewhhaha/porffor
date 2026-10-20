//! Immutable source-order lookup recipes shared by every lowering context.
//!
//! Only structural lookup is indexed. Initializer/class facts still depend on
//! the current signatures and flow, and are evaluated by the current lowerer.
use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingInferenceRecipe<'a> {
    Initializer(Option<&'a Expression>),
    Class(&'a ClassDeclaration),
    ForInKey,
}

#[derive(Debug, Default)]
struct BindingRecipes<'a>(BTreeMap<String, Vec<BindingInferenceRecipe<'a>>>);

#[derive(Debug, Clone)]
pub(crate) struct OwnerBindingInference<'a>(Arc<BTreeMap<String, BindingRecipes<'a>>>);

impl<'a> OwnerBindingInference<'a> {
    pub(crate) fn collect(
        script: &'a [StatementListItem],
        functions: &BTreeMap<FunctionId, FunctionPlan<'a>>,
        interner: &Interner,
    ) -> Self {
        let mut owners = BTreeMap::new();
        let mut recipes = BindingRecipes::default();
        recipes.items(script, interner);
        owners.insert(SCRIPT_OWNER_ID.to_owned(), recipes);
        for (id, plan) in functions {
            let mut recipes = BindingRecipes::default();
            recipes.items(plan.body.statements(), interner);
            owners.insert(id.clone(), recipes);
        }
        Self(Arc::new(owners))
    }

    pub(crate) fn recipes(
        &self,
        owner: &str,
        name: &str,
    ) -> impl Iterator<Item = BindingInferenceRecipe<'a>> + '_ {
        self.0
            .get(owner)
            .and_then(|owner| owner.0.get(name))
            .into_iter()
            .flatten()
            .copied()
    }
}

impl<'a> BindingRecipes<'a> {
    fn push(&mut self, name: String, recipe: BindingInferenceRecipe<'a>) {
        self.0.entry(name).or_default().push(recipe);
    }

    fn variables(&mut self, variables: &'a [Variable], interner: &Interner) {
        // The original declaration lookup stops at its first matching binding,
        // including when that initializer has no static fact. Later statements
        // remain candidates, but a duplicate within this declaration does not.
        let mut seen = BTreeSet::new();
        for variable in variables {
            if let Binding::Identifier(identifier) = variable.binding() {
                let name = interner.resolve_expect(identifier.sym()).to_string();
                if seen.insert(name.clone()) {
                    self.push(name, BindingInferenceRecipe::Initializer(variable.init()));
                }
            }
        }
    }

    fn items(&mut self, items: &'a [StatementListItem], interner: &Interner) {
        for item in items {
            match item {
                StatementListItem::Statement(statement) => self.statement(statement, interner),
                StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                    Declaration::Lexical(lexical) => match lexical {
                        LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => {
                            self.variables(list.as_ref(), interner)
                        }
                        LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {}
                    },
                    Declaration::ClassDeclaration(class) => self.push(
                        interner.resolve_expect(class.name().sym()).to_string(),
                        BindingInferenceRecipe::Class(class),
                    ),
                    Declaration::FunctionDeclaration(_)
                    | Declaration::GeneratorDeclaration(_)
                    | Declaration::AsyncFunctionDeclaration(_)
                    | Declaration::AsyncGeneratorDeclaration(_) => {}
                },
            }
        }
    }

    fn statement(&mut self, statement: &'a Statement, interner: &Interner) {
        match statement {
            Statement::Var(var) => self.variables(var.0.as_ref(), interner),
            Statement::Block(block) => self.items(block.statement_list().statements(), interner),
            Statement::If(branch) => {
                self.statement(branch.body(), interner);
                if let Some(otherwise) = branch.else_node() {
                    self.statement(otherwise, interner);
                }
            }
            Statement::WhileLoop(loop_) => self.statement(loop_.body(), interner),
            Statement::DoWhileLoop(loop_) => self.statement(loop_.body(), interner),
            Statement::ForLoop(loop_) => {
                if let Some(ForLoopInitializer::Var(var)) = loop_.init() {
                    self.variables(var.0.as_ref(), interner);
                }
                self.statement(loop_.body(), interner);
            }
            Statement::ForInLoop(loop_) => {
                let key = match loop_.initializer() {
                    IterableLoopInitializer::Identifier(identifier) => Some(identifier),
                    IterableLoopInitializer::Var(variable) => match variable.binding() {
                        Binding::Identifier(identifier) => Some(identifier),
                        Binding::Pattern(_) => None,
                    },
                    IterableLoopInitializer::Access(_)
                    | IterableLoopInitializer::WebCompatCall(_)
                    | IterableLoopInitializer::Let(_)
                    | IterableLoopInitializer::Const(_)
                    | IterableLoopInitializer::Using(_)
                    | IterableLoopInitializer::AwaitUsing(_)
                    | IterableLoopInitializer::Pattern(_) => None,
                };
                if let Some(key) = key {
                    self.push(
                        interner.resolve_expect(key.sym()).to_string(),
                        BindingInferenceRecipe::ForInKey,
                    );
                }
                self.statement(loop_.body(), interner);
            }
            Statement::Switch(switch) => {
                for case in switch.cases() {
                    self.items(case.body().statements(), interner);
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = labelled_base_statement(labelled) {
                    self.statement(statement, interner);
                }
            }
            Statement::Expression(_)
            | Statement::Empty
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::ForOfLoop(_)
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {}
        }
    }
}
