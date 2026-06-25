use super::*;

#[derive(Debug, Clone)]
pub(crate) struct PendingFunction<'a> {
    pub(crate) id: FunctionId,
    pub(crate) name: String,
    pub(crate) to_string_representation: CallableToStringRepresentation,
    pub(crate) flavor: FunctionFlavor,
    pub(crate) strict: bool,
    pub(crate) constructable: bool,
    pub(crate) self_binding_name: Option<String>,
    pub(crate) parameters: &'a FormalParameterList,
    pub(crate) body: &'a FunctionBody,
    pub(crate) is_expression: bool,
    pub(crate) capture_aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CaptureBindingPlan {
    pub(crate) owner_id: String,
    pub(crate) source_name: String,
    pub(crate) slot: u32,
    pub(crate) hops: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct OwnerPlan {
    pub(crate) flavor: FunctionFlavor,
    pub(crate) strict: bool,
    pub(crate) parent_owner_id: Option<String>,
    pub(crate) root_bindings: BTreeSet<String>,
    pub(crate) function_bindings: BTreeMap<String, FunctionId>,
    pub(crate) owned_env_slots: BTreeMap<String, u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionPlan<'a> {
    pub(crate) id: FunctionId,
    pub(crate) name: String,
    pub(crate) to_string_representation: CallableToStringRepresentation,
    pub(crate) flavor: FunctionFlavor,
    pub(crate) strict: bool,
    pub(crate) constructable: bool,
    pub(crate) self_binding_name: Option<String>,
    pub(crate) parent_owner_id: String,
    pub(crate) parameters: &'a FormalParameterList,
    pub(crate) body: &'a FunctionBody,
    pub(crate) is_expression: bool,
    pub(crate) root_functions: Vec<PendingFunction<'a>>,
    pub(crate) captures: BTreeMap<String, CaptureBindingPlan>,
}

#[derive(Debug, Clone)]
pub(crate) struct Analysis<'a> {
    pub(crate) owner_plans: BTreeMap<String, OwnerPlan>,
    pub(crate) function_plans: BTreeMap<FunctionId, FunctionPlan<'a>>,
    pub(crate) function_declaration_ids: BTreeMap<String, FunctionId>,
    pub(crate) function_expr_ids: BTreeMap<String, FunctionId>,
    pub(crate) class_method_ids: BTreeMap<String, FunctionId>,
    pub(crate) owner_free_refs: BTreeMap<String, BTreeMap<String, String>>,
    pub(crate) function_order: Vec<FunctionId>,
    pub(crate) script_root_functions: Vec<PendingFunction<'a>>,
    pub(crate) script_items: &'a [StatementListItem],
}

#[derive(Default)]
pub(crate) struct AnalysisBuilder<'a> {
    owner_plans: BTreeMap<String, OwnerPlan>,
    function_plans: BTreeMap<FunctionId, FunctionPlan<'a>>,
    function_declaration_ids: BTreeMap<String, FunctionId>,
    function_expr_ids: BTreeMap<String, FunctionId>,
    class_method_ids: BTreeMap<String, FunctionId>,
    function_free_refs: BTreeMap<FunctionId, BTreeMap<String, String>>,
    function_order: Vec<FunctionId>,
    next_function_id: usize,
}

impl<'a> AnalysisBuilder<'a> {
    pub(crate) fn finish(
        mut self,
        script: &'a Script,
        interner: &'a Interner,
        source_text: &'a str,
    ) -> Analysis<'a> {
        let script_root_functions = self.collect_root_functions(
            interner,
            source_text,
            script.statements().statements(),
            script.strict(),
        );
        self.owner_plans.insert(
            SCRIPT_OWNER_ID.to_string(),
            OwnerPlan {
                flavor: FunctionFlavor::Ordinary,
                strict: script.strict(),
                parent_owner_id: None,
                root_bindings: self.collect_owner_bindings(
                    interner,
                    &[],
                    None,
                    false,
                    false,
                    false,
                    script.statements().statements(),
                    &script_root_functions,
                ),
                function_bindings: script_root_functions
                    .iter()
                    .map(|function| (function.name.clone(), function.id.clone()))
                    .collect(),
                owned_env_slots: BTreeMap::new(),
            },
        );
        self.scan_owner_items(
            SCRIPT_OWNER_ID,
            script.statements().statements(),
            interner,
            source_text,
            None,
            &BTreeMap::new(),
        );
        for function in script_root_functions.iter().cloned() {
            self.collect_function_plan(
                function,
                SCRIPT_OWNER_ID.to_string(),
                interner,
                source_text,
            );
        }
        self.finalize_capture_plans();
        Analysis {
            owner_plans: self.owner_plans,
            function_plans: self.function_plans,
            function_declaration_ids: self.function_declaration_ids,
            function_expr_ids: self.function_expr_ids,
            class_method_ids: self.class_method_ids,
            owner_free_refs: self.function_free_refs,
            function_order: self.function_order,
            script_root_functions,
            script_items: script.statements().statements(),
        }
    }

    fn alloc_function_id(&mut self) -> FunctionId {
        let id = format!("f{}", self.next_function_id);
        self.next_function_id += 1;
        id
    }

    fn collect_root_functions(
        &mut self,
        interner: &Interner,
        source_text: &str,
        items: &'a [StatementListItem],
        owner_strict: bool,
    ) -> Vec<PendingFunction<'a>> {
        let mut functions = Vec::new();
        for item in items {
            let function = match item {
                StatementListItem::Declaration(declaration) => {
                    let Declaration::FunctionDeclaration(function) = declaration.as_ref() else {
                        continue;
                    };
                    function
                }
                StatementListItem::Statement(statement) => {
                    let Statement::Labelled(labelled) = statement.as_ref() else {
                        continue;
                    };
                    let Some(function) = labelled_function_declaration(labelled) else {
                        continue;
                    };
                    function
                }
            };
            let name = function_name(interner, function, None);
            let id = self.alloc_function_id();
            self.function_declaration_ids
                .insert(function_declaration_key(function), id.clone());
            functions.push(PendingFunction {
                id,
                name,
                to_string_representation: CallableToStringRepresentation::ExactSource(
                    function_source_slice(function, source_text),
                ),
                flavor: FunctionFlavor::Ordinary,
                strict: owner_strict || function.body().strict(),
                constructable: true,
                self_binding_name: Some(interner.resolve_expect(function.name().sym()).to_string()),
                parameters: function.parameters(),
                body: function.body(),
                is_expression: false,
                capture_aliases: BTreeMap::new(),
            });
        }
        functions
    }

    fn collect_function_plan(
        &mut self,
        function: PendingFunction<'a>,
        parent_owner_id: String,
        interner: &'a Interner,
        source_text: &'a str,
    ) {
        let owner_id = function.id.clone();
        let root_functions = self.collect_root_functions(
            interner,
            source_text,
            function.body.statements(),
            function.strict,
        );
        let simple_parameter_names = if function.flavor == FunctionFlavor::Ordinary {
            collect_simple_parameter_names(interner, function.parameters)
        } else {
            Vec::new()
        };
        let mut owned_env_slots = BTreeMap::new();
        for (slot, name) in simple_parameter_names.iter().enumerate() {
            owned_env_slots.insert(name.clone(), slot as u32);
        }
        self.owner_plans.insert(
            owner_id.clone(),
            OwnerPlan {
                flavor: function.flavor,
                strict: function.strict,
                parent_owner_id: Some(parent_owner_id.clone()),
                root_bindings: self.collect_owner_bindings(
                    interner,
                    function.parameters.as_ref(),
                    function.self_binding_name.as_deref(),
                    function.flavor == FunctionFlavor::Ordinary,
                    function.flavor == FunctionFlavor::Ordinary,
                    function.flavor == FunctionFlavor::Ordinary,
                    function.body.statements(),
                    &root_functions,
                ),
                function_bindings: root_functions
                    .iter()
                    .map(|nested| (nested.name.clone(), nested.id.clone()))
                    .collect(),
                owned_env_slots,
            },
        );
        self.scan_owner_items(
            &owner_id,
            function.body.statements(),
            interner,
            source_text,
            Some(function.name.as_str()),
            &function.capture_aliases,
        );
        self.function_plans.insert(
            owner_id.clone(),
            FunctionPlan {
                id: owner_id.clone(),
                name: function.name.clone(),
                to_string_representation: function.to_string_representation.clone(),
                flavor: function.flavor,
                strict: function.strict,
                constructable: function.constructable,
                self_binding_name: function.self_binding_name.clone(),
                parent_owner_id,
                parameters: function.parameters,
                body: function.body,
                is_expression: function.is_expression,
                root_functions: root_functions.clone(),
                captures: BTreeMap::new(),
            },
        );
        for nested in root_functions {
            self.collect_function_plan(nested, owner_id.clone(), interner, source_text);
        }
        self.function_order.push(owner_id);
    }

    fn collect_owner_bindings(
        &self,
        interner: &Interner,
        params: &[FormalParameter],
        self_name: Option<&str>,
        has_own_this: bool,
        has_own_arguments: bool,
        has_own_new_target: bool,
        items: &'a [StatementListItem],
        root_functions: &[PendingFunction<'a>],
    ) -> BTreeSet<String> {
        let mut bindings = BTreeSet::new();
        if let Some(self_name) = self_name {
            bindings.insert(self_name.to_string());
        }
        if has_own_this {
            bindings.insert(LEXICAL_THIS_NAME.to_string());
        }
        if has_own_arguments {
            bindings.insert(LEXICAL_ARGUMENTS_NAME.to_string());
        }
        if has_own_new_target {
            bindings.insert(LEXICAL_NEW_TARGET_NAME.to_string());
        }
        for parameter in params {
            let mut parameter_names = Vec::new();
            collect_binding_names(
                interner,
                parameter.variable().binding(),
                &mut parameter_names,
            );
            bindings.extend(parameter_names);
        }
        for function in root_functions {
            bindings.insert(function.name.clone());
        }
        self.collect_declared_bindings_from_items(interner, items, &mut bindings);
        bindings
    }

    fn collect_declared_bindings_from_items(
        &self,
        interner: &Interner,
        items: &'a [StatementListItem],
        bindings: &mut BTreeSet<String>,
    ) {
        for item in items {
            match item {
                StatementListItem::Statement(statement) => {
                    self.collect_declared_bindings_from_statement(interner, statement, bindings);
                }
                StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                    Declaration::Lexical(lexical) => {
                        self.collect_declared_bindings_from_lexical(interner, lexical, bindings);
                    }
                    Declaration::ClassDeclaration(class) => {
                        bindings.insert(interner.resolve_expect(class.name().sym()).to_string());
                    }
                    Declaration::FunctionDeclaration(function) => {
                        bindings.insert(function_name(interner, function, None));
                    }
                    _ => {}
                },
            }
        }
    }

    fn collect_declared_bindings_from_statement(
        &self,
        interner: &Interner,
        statement: &'a Statement,
        bindings: &mut BTreeSet<String>,
    ) {
        match statement {
            Statement::Block(block) => self.collect_declared_bindings_from_items(
                interner,
                block.statement_list().statements(),
                bindings,
            ),
            Statement::If(if_statement) => {
                self.collect_declared_bindings_from_statement(
                    interner,
                    if_statement.body(),
                    bindings,
                );
                if let Some(else_node) = if_statement.else_node() {
                    self.collect_declared_bindings_from_statement(interner, else_node, bindings);
                }
            }
            Statement::WhileLoop(while_loop) => {
                self.collect_declared_bindings_from_statement(
                    interner,
                    while_loop.body(),
                    bindings,
                );
            }
            Statement::DoWhileLoop(do_while) => {
                self.collect_declared_bindings_from_statement(interner, do_while.body(), bindings);
            }
            Statement::ForLoop(for_loop) => {
                if let Some(init) = for_loop.init() {
                    match init {
                        ForLoopInitializer::Var(var) => {
                            self.collect_declared_bindings_from_var(interner, var, bindings);
                        }
                        ForLoopInitializer::Lexical(lexical) => {
                            self.collect_declared_bindings_from_lexical(
                                interner,
                                lexical.declaration(),
                                bindings,
                            );
                        }
                        ForLoopInitializer::Expression(_) => {}
                    }
                }
                self.collect_declared_bindings_from_statement(interner, for_loop.body(), bindings);
            }
            Statement::ForOfLoop(for_of) => {
                match for_of.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        bindings.insert(tdz_binding_storage_name(&source_name));
                    }
                    IterableLoopInitializer::Let(Binding::Pattern(pattern))
                    | IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                        let mut names = Vec::new();
                        collect_binding_names(
                            interner,
                            &Binding::Pattern(pattern.clone()),
                            &mut names,
                        );
                        for source_name in names {
                            bindings.insert(tdz_binding_storage_name(&source_name));
                            bindings.insert(source_name);
                        }
                    }
                    _ => {}
                }
                match for_of.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        bindings.insert(for_of_loop_binding_storage_name(for_of, &source_name));
                    }
                    IterableLoopInitializer::Var(variable) => {
                        if let Binding::Pattern(pattern) = variable.binding() {
                            let mut names = Vec::new();
                            collect_binding_names(
                                interner,
                                &Binding::Pattern(pattern.clone()),
                                &mut names,
                            );
                            bindings.extend(names);
                        }
                    }
                    _ => {}
                }
                self.collect_declared_bindings_from_statement(interner, for_of.body(), bindings);
            }
            Statement::ForInLoop(for_in) => {
                match for_in.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        bindings.insert(tdz_binding_storage_name(&source_name));
                    }
                    _ => {}
                }
                match for_in.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        bindings.insert(for_in_loop_binding_storage_name(for_in, &source_name));
                    }
                    _ => {}
                }
                self.collect_declared_bindings_from_statement(interner, for_in.body(), bindings);
            }
            Statement::Switch(switch) => {
                for case in switch.cases() {
                    self.collect_declared_bindings_from_items(
                        interner,
                        case.body().statements(),
                        bindings,
                    );
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = labelled_base_statement(labelled) {
                    self.collect_declared_bindings_from_statement(interner, statement, bindings);
                }
            }
            Statement::Var(var) => self.collect_declared_bindings_from_var(interner, var, bindings),
            Statement::Expression(_)
            | Statement::Empty
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {}
        }
    }

    fn collect_declared_bindings_from_var(
        &self,
        interner: &Interner,
        declaration: &'a VarDeclaration,
        bindings: &mut BTreeSet<String>,
    ) {
        for declarator in declaration.0.as_ref() {
            if let Binding::Identifier(identifier) = declarator.binding() {
                bindings.insert(interner.resolve_expect(identifier.sym()).to_string());
            }
        }
    }

    fn collect_declared_bindings_from_lexical(
        &self,
        interner: &Interner,
        declaration: &'a LexicalDeclaration,
        bindings: &mut BTreeSet<String>,
    ) {
        let list = match declaration {
            LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => list,
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => return,
        };
        for declarator in list.as_ref() {
            if let Binding::Identifier(identifier) = declarator.binding() {
                bindings.insert(interner.resolve_expect(identifier.sym()).to_string());
            }
        }
    }

    fn scan_owner_items(
        &mut self,
        owner_id: &str,
        items: &'a [StatementListItem],
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
    ) {
        let mut refs = BTreeMap::new();
        for item in items {
            self.scan_item(
                owner_id,
                item,
                interner,
                source_text,
                self_name,
                capture_aliases,
                &mut refs,
            );
        }
        if owner_id != SCRIPT_OWNER_ID {
            self.function_free_refs.insert(owner_id.to_string(), refs);
        }
    }

    fn scan_item(
        &mut self,
        owner_id: &str,
        item: &'a StatementListItem,
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        match item {
            StatementListItem::Statement(statement) => {
                self.scan_statement(
                    owner_id,
                    statement,
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                Declaration::Lexical(lexical) => {
                    let list = match lexical {
                        LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => list,
                        LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => return,
                    };
                    for declarator in list.as_ref() {
                        if let Some(init) = declarator.init() {
                            self.scan_expression(
                                owner_id,
                                init,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                        }
                    }
                }
                Declaration::FunctionDeclaration(function) => {
                    self.collect_non_root_function_declaration(
                        owner_id,
                        function,
                        interner,
                        source_text,
                        capture_aliases,
                    );
                }
                Declaration::ClassDeclaration(class) => {
                    if let Some(super_ref) = class.super_ref() {
                        self.scan_expression(
                            owner_id,
                            super_ref,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                    if let Some(constructor) = class.constructor() {
                        self.collect_class_constructor_owner_plan(
                            owner_id,
                            constructor,
                            interner,
                            source_text,
                        );
                    }
                    self.collect_class_method_owner_plans(
                        owner_id,
                        class.elements(),
                        interner,
                        source_text,
                    );
                }
                _ => {}
            },
        }
    }

    fn collect_non_root_function_declaration(
        &mut self,
        owner_id: &str,
        function: &'a FunctionDeclaration,
        interner: &'a Interner,
        source_text: &'a str,
        capture_aliases: &BTreeMap<String, String>,
    ) {
        let key = function_declaration_key(function);
        if self.function_declaration_ids.contains_key(&key) {
            return;
        }
        let id = self.alloc_function_id();
        self.function_declaration_ids.insert(key, id.clone());
        let name = function_name(interner, function, None);
        let pending = PendingFunction {
            id,
            name: name.clone(),
            to_string_representation: CallableToStringRepresentation::ExactSource(
                function_source_slice(function, source_text),
            ),
            flavor: FunctionFlavor::Ordinary,
            strict: self
                .owner_plans
                .get(owner_id)
                .is_some_and(|owner| owner.strict)
                || function.body().strict(),
            constructable: true,
            self_binding_name: Some(name),
            parameters: function.parameters(),
            body: function.body(),
            is_expression: false,
            capture_aliases: capture_aliases.clone(),
        };
        self.collect_function_plan(pending, owner_id.to_string(), interner, source_text);
    }

    fn collect_class_method_owner_plans(
        &mut self,
        parent_owner_id: &str,
        elements: &'a [ClassElement],
        interner: &'a Interner,
        source_text: &'a str,
    ) {
        for element in elements {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };
            let key = class_method_key(method);
            if self.class_method_ids.contains_key(&key) {
                continue;
            }
            let id = self.alloc_function_id();
            self.class_method_ids.insert(key, id.clone());
            let root_functions = self.collect_root_functions(
                interner,
                source_text,
                method.body().statements(),
                self.owner_plans
                    .get(parent_owner_id)
                    .is_some_and(|owner| owner.strict)
                    || method.body().strict(),
            );
            let name = match method.name() {
                ClassElementName::PropertyName(PropertyName::Literal(name)) => {
                    interner.resolve_expect(name.sym()).to_string()
                }
                ClassElementName::PrivateName(name) => private_name_key(interner, *name),
                _ => "<class-method>".to_string(),
            };
            self.owner_plans.insert(
                id.clone(),
                OwnerPlan {
                    flavor: FunctionFlavor::Ordinary,
                    strict: self
                        .owner_plans
                        .get(parent_owner_id)
                        .is_some_and(|owner| owner.strict)
                        || method.body().strict(),
                    parent_owner_id: Some(parent_owner_id.to_string()),
                    root_bindings: self.collect_owner_bindings(
                        interner,
                        method.parameters().as_ref(),
                        None,
                        true,
                        true,
                        true,
                        method.body().statements(),
                        &root_functions,
                    ),
                    function_bindings: root_functions
                        .iter()
                        .map(|nested| (nested.name.clone(), nested.id.clone()))
                        .collect(),
                    owned_env_slots: BTreeMap::new(),
                },
            );
            self.scan_owner_items(
                &id,
                method.body().statements(),
                interner,
                source_text,
                Some(name.as_str()),
                &BTreeMap::new(),
            );
            for nested in root_functions {
                self.collect_function_plan(nested, id.clone(), interner, source_text);
            }
        }
    }

    fn collect_class_constructor_owner_plan(
        &mut self,
        parent_owner_id: &str,
        constructor: &'a FunctionExpression,
        interner: &'a Interner,
        source_text: &'a str,
    ) {
        let key = class_constructor_key(constructor);
        if self.class_method_ids.contains_key(&key) {
            return;
        }
        let id = self.alloc_function_id();
        self.class_method_ids.insert(key, id.clone());
        let root_functions = self.collect_root_functions(
            interner,
            source_text,
            constructor.body().statements(),
            self.owner_plans
                .get(parent_owner_id)
                .is_some_and(|owner| owner.strict)
                || constructor.body().strict(),
        );
        self.owner_plans.insert(
            id.clone(),
            OwnerPlan {
                flavor: FunctionFlavor::Ordinary,
                strict: self
                    .owner_plans
                    .get(parent_owner_id)
                    .is_some_and(|owner| owner.strict)
                    || constructor.body().strict(),
                parent_owner_id: Some(parent_owner_id.to_string()),
                root_bindings: self.collect_owner_bindings(
                    interner,
                    constructor.parameters().as_ref(),
                    None,
                    true,
                    true,
                    true,
                    constructor.body().statements(),
                    &root_functions,
                ),
                function_bindings: root_functions
                    .iter()
                    .map(|nested| (nested.name.clone(), nested.id.clone()))
                    .collect(),
                owned_env_slots: BTreeMap::new(),
            },
        );
        self.scan_owner_items(
            &id,
            constructor.body().statements(),
            interner,
            source_text,
            Some("constructor"),
            &BTreeMap::new(),
        );
        for nested in root_functions {
            self.collect_function_plan(nested, id.clone(), interner, source_text);
        }
    }

    fn record_ref(
        &self,
        owner_id: &str,
        source_name: String,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        let alias = capture_aliases.get(&source_name);
        let owner_has_alias = alias.is_some_and(|alias| {
            self.owner_plans
                .get(owner_id)
                .is_some_and(|owner| owner.root_bindings.contains(alias))
        });
        let storage_name = if owner_has_alias
            || self
                .owner_plans
                .get(owner_id)
                .is_none_or(|owner| !owner.root_bindings.contains(&source_name))
        {
            alias.cloned().unwrap_or_else(|| source_name.clone())
        } else {
            source_name.clone()
        };
        refs.entry(storage_name).or_insert(source_name);
    }

    fn scan_statement(
        &mut self,
        owner_id: &str,
        statement: &'a Statement,
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        match statement {
            Statement::Expression(expression) => {
                self.scan_expression(
                    owner_id,
                    expression,
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Statement::Block(block) => {
                for item in block.statement_list().statements() {
                    self.scan_item(
                        owner_id,
                        item,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Statement::If(if_statement) => {
                self.scan_expression(
                    owner_id,
                    if_statement.cond(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_statement(
                    owner_id,
                    if_statement.body(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                if let Some(else_node) = if_statement.else_node() {
                    self.scan_statement(
                        owner_id,
                        else_node,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Statement::WhileLoop(while_loop) => {
                self.scan_expression(
                    owner_id,
                    while_loop.condition(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_statement(
                    owner_id,
                    while_loop.body(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Statement::DoWhileLoop(do_while) => {
                self.scan_statement(
                    owner_id,
                    do_while.body(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_expression(
                    owner_id,
                    do_while.cond(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Statement::ForLoop(for_loop) => {
                if let Some(init) = for_loop.init() {
                    match init {
                        ForLoopInitializer::Expression(expr) => {
                            self.scan_expression(
                                owner_id,
                                expr,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                        }
                        ForLoopInitializer::Var(var) => {
                            for declarator in var.0.as_ref() {
                                if let Some(init) = declarator.init() {
                                    self.scan_expression(
                                        owner_id,
                                        init,
                                        interner,
                                        source_text,
                                        self_name,
                                        capture_aliases,
                                        refs,
                                    );
                                }
                            }
                        }
                        ForLoopInitializer::Lexical(lexical) => {
                            let declaration = lexical.declaration();
                            let list = match declaration {
                                LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => {
                                    list
                                }
                                LexicalDeclaration::Using(_)
                                | LexicalDeclaration::AwaitUsing(_) => return,
                            };
                            for declarator in list.as_ref() {
                                if let Some(init) = declarator.init() {
                                    self.scan_expression(
                                        owner_id,
                                        init,
                                        interner,
                                        source_text,
                                        self_name,
                                        capture_aliases,
                                        refs,
                                    );
                                }
                            }
                        }
                    }
                }
                if let Some(condition) = for_loop.condition() {
                    self.scan_expression(
                        owner_id,
                        condition,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
                if let Some(update) = for_loop.final_expr() {
                    self.scan_expression(
                        owner_id,
                        update,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
                self.scan_statement(
                    owner_id,
                    for_loop.body(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Statement::Switch(switch) => {
                self.scan_expression(
                    owner_id,
                    switch.val(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                for case in switch.cases() {
                    if let Some(condition) = case.condition() {
                        self.scan_expression(
                            owner_id,
                            condition,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                    for item in case.body().statements() {
                        self.scan_item(
                            owner_id,
                            item,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = labelled_base_statement(labelled) {
                    self.scan_statement(
                        owner_id,
                        statement,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Statement::Var(var) => {
                for declarator in var.0.as_ref() {
                    if let Some(init) = declarator.init() {
                        self.scan_expression(
                            owner_id,
                            init,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                }
            }
            Statement::Return(ret) => {
                if let Some(target) = ret.target() {
                    self.scan_expression(
                        owner_id,
                        target,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Statement::Throw(throw) => {
                self.scan_expression(
                    owner_id,
                    throw.target(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Statement::Try(try_statement) => {
                for item in try_statement.block().statement_list().statements() {
                    self.scan_item(
                        owner_id,
                        item,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
                if let Some(catch) = try_statement.catch() {
                    for item in catch.block().statement_list().statements() {
                        self.scan_item(
                            owner_id,
                            item,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                }
                if let Some(finally_block) = try_statement.finally() {
                    for item in finally_block.block().statement_list().statements() {
                        self.scan_item(
                            owner_id,
                            item,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                }
            }
            Statement::ForOfLoop(for_of) => {
                let mut head_aliases = capture_aliases.clone();
                match for_of.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        head_aliases
                            .insert(source_name.clone(), tdz_binding_storage_name(&source_name));
                    }
                    IterableLoopInitializer::Let(Binding::Pattern(pattern))
                    | IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                        let mut names = Vec::new();
                        collect_binding_names(
                            interner,
                            &Binding::Pattern(pattern.clone()),
                            &mut names,
                        );
                        for source_name in names {
                            head_aliases.insert(
                                source_name.clone(),
                                tdz_binding_storage_name(&source_name),
                            );
                        }
                    }
                    _ => {}
                }
                self.scan_expression(
                    owner_id,
                    for_of.iterable(),
                    interner,
                    source_text,
                    self_name,
                    &head_aliases,
                    refs,
                );
                let mut body_aliases = capture_aliases.clone();
                match for_of.initializer() {
                    IterableLoopInitializer::Let(Binding::Identifier(identifier))
                    | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                        let source_name = interner.resolve_expect(identifier.sym()).to_string();
                        body_aliases.insert(
                            source_name.clone(),
                            for_of_loop_binding_storage_name(for_of, &source_name),
                        );
                    }
                    IterableLoopInitializer::Let(Binding::Pattern(pattern))
                    | IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                        let mut names = Vec::new();
                        collect_binding_names(
                            interner,
                            &Binding::Pattern(pattern.clone()),
                            &mut names,
                        );
                        for source_name in names {
                            body_aliases.insert(source_name.clone(), source_name);
                        }
                    }
                    IterableLoopInitializer::Var(variable) => {
                        if let Binding::Pattern(pattern) = variable.binding() {
                            let mut names = Vec::new();
                            collect_binding_names(
                                interner,
                                &Binding::Pattern(pattern.clone()),
                                &mut names,
                            );
                            for source_name in names {
                                body_aliases.insert(source_name.clone(), source_name);
                            }
                        }
                    }
                    _ => {}
                }
                self.scan_statement(
                    owner_id,
                    for_of.body(),
                    interner,
                    source_text,
                    self_name,
                    &body_aliases,
                    refs,
                );
            }
            Statement::ForInLoop(for_in) => {
                let mut head_aliases = capture_aliases.clone();
                if let IterableLoopInitializer::Let(Binding::Identifier(identifier))
                | IterableLoopInitializer::Const(Binding::Identifier(identifier)) =
                    for_in.initializer()
                {
                    let source_name = interner.resolve_expect(identifier.sym()).to_string();
                    head_aliases
                        .insert(source_name.clone(), tdz_binding_storage_name(&source_name));
                }
                self.scan_expression(
                    owner_id,
                    for_in.target(),
                    interner,
                    source_text,
                    self_name,
                    &head_aliases,
                    refs,
                );
                let mut body_aliases = capture_aliases.clone();
                if let IterableLoopInitializer::Let(Binding::Identifier(identifier))
                | IterableLoopInitializer::Const(Binding::Identifier(identifier)) =
                    for_in.initializer()
                {
                    let source_name = interner.resolve_expect(identifier.sym()).to_string();
                    body_aliases.insert(
                        source_name.clone(),
                        for_in_loop_binding_storage_name(for_in, &source_name),
                    );
                }
                self.scan_statement(
                    owner_id,
                    for_in.body(),
                    interner,
                    source_text,
                    self_name,
                    &body_aliases,
                    refs,
                );
            }
            Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::Empty
            | Statement::With(_) => {}
        }
    }

    fn scan_expression(
        &mut self,
        owner_id: &str,
        expression: &'a Expression,
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        match expression {
            Expression::Identifier(identifier) => {
                let name = interner.resolve_expect(identifier.sym()).to_string();
                if name == "arguments" {
                    let owner = self.owner_plans.get(owner_id);
                    let source_binds_arguments =
                        owner.is_some_and(|owner| owner.root_bindings.contains("arguments"));
                    if !source_binds_arguments {
                        if owner.is_some_and(|owner| owner.flavor == FunctionFlavor::Arrow) {
                            self.record_ref(
                                owner_id,
                                LEXICAL_ARGUMENTS_NAME.to_string(),
                                capture_aliases,
                                refs,
                            );
                        } else if owner_id != SCRIPT_OWNER_ID {
                            self.record_ref(
                                owner_id,
                                LEXICAL_ARGUMENTS_NAME.to_string(),
                                capture_aliases,
                                refs,
                            );
                        } else {
                            self.record_ref(owner_id, name, capture_aliases, refs);
                        }
                        return;
                    }
                }
                self.record_ref(owner_id, name, capture_aliases, refs);
            }
            Expression::Parenthesized(expression) => {
                self.scan_expression(
                    owner_id,
                    expression.expression(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::ArrayLiteral(array) => {
                for element in array.as_ref().iter().flatten() {
                    self.scan_expression(
                        owner_id,
                        element,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::ObjectLiteral(object) => {
                for property in object.properties() {
                    match property {
                        PropertyDefinition::Property(name, value) => {
                            self.scan_property_name(
                                owner_id,
                                name,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                            self.scan_expression(
                                owner_id,
                                value,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                        }
                        PropertyDefinition::SpreadObject(value) => {
                            self.scan_expression(
                                owner_id,
                                value,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                        }
                        PropertyDefinition::MethodDefinition(method) => {
                            self.scan_property_name(
                                owner_id,
                                method.name(),
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                            let key = object_method_key(method);
                            if !self.function_expr_ids.contains_key(&key) {
                                let id = self.alloc_function_id();
                                self.function_expr_ids.insert(key, id.clone());
                                let name = method
                                    .name()
                                    .prop_name()
                                    .map(|identifier| {
                                        interner.resolve_expect(identifier.sym()).to_string()
                                    })
                                    .unwrap_or_else(|| "<method>".to_string());
                                let pending = PendingFunction {
                                    id,
                                    name,
                                    to_string_representation:
                                        CallableToStringRepresentation::ExactSource(
                                            object_method_source_slice(method, source_text),
                                        ),
                                    flavor: FunctionFlavor::Ordinary,
                                    strict: self
                                        .owner_plans
                                        .get(owner_id)
                                        .is_some_and(|owner| owner.strict)
                                        || method.body().strict(),
                                    constructable: false,
                                    self_binding_name: None,
                                    parameters: method.parameters(),
                                    body: method.body(),
                                    is_expression: true,
                                    capture_aliases: capture_aliases.clone(),
                                };
                                self.collect_function_plan(
                                    pending,
                                    owner_id.to_string(),
                                    interner,
                                    source_text,
                                );
                            }
                        }
                        PropertyDefinition::IdentifierReference(identifier) => {
                            self.record_ref(
                                owner_id,
                                interner.resolve_expect(identifier.sym()).to_string(),
                                capture_aliases,
                                refs,
                            );
                        }
                        PropertyDefinition::CoverInitializedName(_, value) => {
                            self.scan_expression(
                                owner_id,
                                value,
                                interner,
                                source_text,
                                self_name,
                                capture_aliases,
                                refs,
                            );
                        }
                    }
                }
            }
            Expression::Unary(unary) => {
                self.scan_expression(
                    owner_id,
                    unary.target(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::Binary(binary) => {
                self.scan_expression(
                    owner_id,
                    binary.lhs(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_expression(
                    owner_id,
                    binary.rhs(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::Assign(assign) => {
                match assign.lhs() {
                    AssignTarget::Identifier(identifier) => {
                        self.record_ref(
                            owner_id,
                            interner.resolve_expect(identifier.sym()).to_string(),
                            capture_aliases,
                            refs,
                        );
                    }
                    AssignTarget::Access(access) => {
                        self.scan_property_access(
                            owner_id,
                            access,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                    _ => {}
                }
                self.scan_expression(
                    owner_id,
                    assign.rhs(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::Update(update) => {
                if let UpdateTarget::Identifier(identifier) = update.target() {
                    self.record_ref(
                        owner_id,
                        interner.resolve_expect(identifier.sym()).to_string(),
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::Call(call) => {
                self.scan_expression(
                    owner_id,
                    call.function(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                for arg in call.args() {
                    self.scan_expression(
                        owner_id,
                        arg,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::PropertyAccess(access) => {
                self.scan_property_access(
                    owner_id,
                    access,
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::FunctionExpression(function) => {
                let key = function_expression_key(function);
                if !self.function_expr_ids.contains_key(&key) {
                    let id = self.alloc_function_id();
                    self.function_expr_ids.insert(key, id.clone());
                    let self_binding_name = function
                        .name()
                        .map(|identifier| interner.resolve_expect(identifier.sym()).to_string());
                    let pending = PendingFunction {
                        id,
                        name: self_binding_name
                            .clone()
                            .unwrap_or_else(|| "<anonymous>".to_string()),
                        to_string_representation: CallableToStringRepresentation::ExactSource(
                            function_expression_source_slice(function, source_text),
                        ),
                        flavor: FunctionFlavor::Ordinary,
                        strict: self
                            .owner_plans
                            .get(owner_id)
                            .is_some_and(|owner| owner.strict)
                            || function.body().strict(),
                        constructable: true,
                        self_binding_name,
                        parameters: function.parameters(),
                        body: function.body(),
                        is_expression: true,
                        capture_aliases: capture_aliases.clone(),
                    };
                    self.collect_function_plan(
                        pending,
                        owner_id.to_string(),
                        interner,
                        source_text,
                    );
                }
            }
            Expression::ArrowFunction(function) => {
                let key = arrow_function_key(function);
                if !self.function_expr_ids.contains_key(&key) {
                    let id = self.alloc_function_id();
                    self.function_expr_ids.insert(key, id.clone());
                    let pending = PendingFunction {
                        id,
                        name: function
                            .name()
                            .map(|identifier| interner.resolve_expect(identifier.sym()).to_string())
                            .unwrap_or_else(|| "<arrow>".to_string()),
                        to_string_representation: CallableToStringRepresentation::ExactSource(
                            arrow_function_source_slice(function, source_text),
                        ),
                        flavor: FunctionFlavor::Arrow,
                        strict: self
                            .owner_plans
                            .get(owner_id)
                            .is_some_and(|owner| owner.strict)
                            || function.body().strict(),
                        constructable: false,
                        self_binding_name: None,
                        parameters: function.parameters(),
                        body: function.body(),
                        is_expression: true,
                        capture_aliases: capture_aliases.clone(),
                    };
                    self.collect_function_plan(
                        pending,
                        owner_id.to_string(),
                        interner,
                        source_text,
                    );
                }
            }
            Expression::ClassExpression(class) => {
                if let Some(super_ref) = class.super_ref() {
                    self.scan_expression(
                        owner_id,
                        super_ref,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
                if let Some(constructor) = class.constructor() {
                    self.collect_class_constructor_owner_plan(
                        owner_id,
                        constructor,
                        interner,
                        source_text,
                    );
                }
                self.collect_class_method_owner_plans(
                    owner_id,
                    class.elements(),
                    interner,
                    source_text,
                );
            }
            Expression::SuperCall(call) => {
                for arg in call.arguments() {
                    self.scan_expression(
                        owner_id,
                        arg,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::This(_) => {
                if self
                    .owner_plans
                    .get(owner_id)
                    .is_some_and(|owner| owner.flavor == FunctionFlavor::Arrow)
                {
                    self.record_ref(
                        owner_id,
                        LEXICAL_THIS_NAME.to_string(),
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::NewTarget(_) => {
                if self
                    .owner_plans
                    .get(owner_id)
                    .is_some_and(|owner| owner.flavor == FunctionFlavor::Arrow)
                {
                    self.record_ref(
                        owner_id,
                        LEXICAL_NEW_TARGET_NAME.to_string(),
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::New(new_expr) => {
                self.scan_expression(
                    owner_id,
                    new_expr.constructor(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                for arg in new_expr.arguments() {
                    self.scan_expression(
                        owner_id,
                        arg,
                        interner,
                        source_text,
                        self_name,
                        capture_aliases,
                        refs,
                    );
                }
            }
            Expression::TemplateLiteral(template) => {
                for element in template.elements() {
                    if let TemplateElement::Expr(expr) = element {
                        self.scan_expression(
                            owner_id,
                            expr,
                            interner,
                            source_text,
                            self_name,
                            capture_aliases,
                            refs,
                        );
                    }
                }
            }
            Expression::Conditional(conditional) => {
                self.scan_expression(
                    owner_id,
                    conditional.condition(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_expression(
                    owner_id,
                    conditional.if_true(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
                self.scan_expression(
                    owner_id,
                    conditional.if_false(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
            Expression::AsyncArrowFunction(_)
            | Expression::Literal(_)
            | Expression::RegExpLiteral(_)
            | Expression::GeneratorExpression(_)
            | Expression::AsyncFunctionExpression(_)
            | Expression::AsyncGeneratorExpression(_)
            | Expression::ImportCall(_)
            | Expression::Optional(_)
            | Expression::TaggedTemplate(_)
            | Expression::ImportMeta(_)
            | Expression::BinaryInPrivate(_)
            | Expression::Await(_)
            | Expression::Yield(_)
            | Expression::FormalParameterList(_)
            | Expression::Debugger => {}
            Expression::Spread(spread) => {
                self.scan_expression(
                    owner_id,
                    spread.target(),
                    interner,
                    source_text,
                    self_name,
                    capture_aliases,
                    refs,
                );
            }
        }
        let _ = self_name;
    }

    fn scan_property_name(
        &mut self,
        owner_id: &str,
        name: &'a PropertyName,
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        if let PropertyName::Computed(expr) = name {
            self.scan_expression(
                owner_id,
                expr,
                interner,
                source_text,
                self_name,
                capture_aliases,
                refs,
            );
        }
    }

    fn scan_property_access(
        &mut self,
        owner_id: &str,
        access: &'a PropertyAccess,
        interner: &'a Interner,
        source_text: &'a str,
        self_name: Option<&str>,
        capture_aliases: &BTreeMap<String, String>,
        refs: &mut BTreeMap<String, String>,
    ) {
        let PropertyAccess::Simple(access) = access else {
            return;
        };
        self.scan_expression(
            owner_id,
            access.target(),
            interner,
            source_text,
            self_name,
            capture_aliases,
            refs,
        );
        if let PropertyAccessField::Expr(expr) = access.field() {
            self.scan_expression(
                owner_id,
                expr,
                interner,
                source_text,
                self_name,
                capture_aliases,
                refs,
            );
        }
    }

    fn finalize_capture_plans(&mut self) {
        let mut owned_names = BTreeMap::<String, BTreeSet<String>>::new();
        let function_ids = self.function_order.clone();
        for function_id in function_ids {
            let Some(function) = self.function_plans.get(&function_id).cloned() else {
                continue;
            };
            let local_bindings = self
                .owner_plans
                .get(&function.id)
                .map(|owner| owner.root_bindings.clone())
                .unwrap_or_default();
            let free_refs = self
                .function_free_refs
                .get(&function.id)
                .cloned()
                .unwrap_or_default();
            let mut captures = BTreeMap::new();
            for (name, source_name) in free_refs {
                if local_bindings.contains(&name) {
                    continue;
                }
                let Some(owner_id) = self.resolve_capture_owner(&function.parent_owner_id, &name)
                else {
                    continue;
                };
                owned_names
                    .entry(owner_id.clone())
                    .or_default()
                    .insert(name.clone());
                captures.insert(
                    name,
                    CaptureBindingPlan {
                        owner_id,
                        source_name,
                        slot: 0,
                        hops: 0,
                    },
                );
            }
            if let Some(plan) = self.function_plans.get_mut(&function.id) {
                plan.captures = captures;
            }
        }

        for owner_id in self.class_method_ids.values() {
            let local_bindings = self
                .owner_plans
                .get(owner_id)
                .map(|owner| owner.root_bindings.clone())
                .unwrap_or_default();
            let free_refs = self
                .function_free_refs
                .get(owner_id)
                .cloned()
                .unwrap_or_default();
            for (name, _) in free_refs {
                if local_bindings.contains(&name) {
                    continue;
                }
                if let Some(capture_owner_id) = self.resolve_capture_owner(owner_id, &name) {
                    owned_names
                        .entry(capture_owner_id)
                        .or_default()
                        .insert(name);
                }
            }
        }

        for (owner_id, names) in owned_names {
            let Some(owner) = self.owner_plans.get_mut(&owner_id) else {
                continue;
            };
            let mut next_slot = owner
                .owned_env_slots
                .values()
                .copied()
                .max()
                .map(|slot| slot + 1)
                .unwrap_or(0);
            for name in names {
                owner.owned_env_slots.entry(name).or_insert_with(|| {
                    let slot = next_slot;
                    next_slot += 1;
                    slot
                });
            }
        }

        let function_ids = self.function_order.clone();
        for function_id in function_ids {
            let Some(function) = self.function_plans.get(&function_id).cloned() else {
                continue;
            };
            let mut captures = function.captures;
            let names: Vec<String> = captures.keys().cloned().collect();
            for name in names {
                if let Some(capture) = captures.get_mut(&name) {
                    capture.hops = self.capture_hops(&function.id, &capture.owner_id);
                    capture.slot = self.owner_plans[&capture.owner_id].owned_env_slots[&name];
                }
            }
            if let Some(plan) = self.function_plans.get_mut(&function_id) {
                plan.captures = captures;
            }
        }
    }

    fn resolve_capture_owner(&self, start_owner_id: &str, name: &str) -> Option<String> {
        let mut owner_id = Some(start_owner_id.to_string());
        while let Some(current) = owner_id {
            let owner = self.owner_plans.get(&current)?;
            if owner.root_bindings.contains(name) {
                return Some(current);
            }
            owner_id = owner.parent_owner_id.clone();
        }
        None
    }

    fn capture_hops(&self, current_owner_id: &str, target_owner_id: &str) -> u32 {
        let mut hops = 0;
        let mut env_owner = self.effective_env_owner(current_owner_id);
        while let Some(current) = env_owner {
            if current == target_owner_id {
                return hops;
            }
            env_owner = self.next_env_owner(&current);
            hops += 1;
        }
        0
    }

    fn effective_env_owner(&self, owner_id: &str) -> Option<String> {
        let mut current = Some(owner_id.to_string());
        while let Some(owner_id) = current {
            let owner = self.owner_plans.get(&owner_id)?;
            if !owner.owned_env_slots.is_empty() {
                return Some(owner_id);
            }
            current = owner.parent_owner_id.clone();
        }
        None
    }

    fn next_env_owner(&self, owner_id: &str) -> Option<String> {
        let mut current = self.owner_plans.get(owner_id)?.parent_owner_id.clone();
        while let Some(parent_id) = current {
            let owner = self.owner_plans.get(&parent_id)?;
            if !owner.owned_env_slots.is_empty() {
                return Some(parent_id);
            }
            current = owner.parent_owner_id.clone();
        }
        None
    }
}
