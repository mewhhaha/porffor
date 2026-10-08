//! One canonical compiled graph, reusable against any Realm's module cache.

use super::*;

#[derive(Clone)]
pub(crate) struct ModuleGraphPlan {
    graph: lila_ir::ModuleExecutionGraphIr,
}

impl ModuleGraphPlan {
    pub(crate) fn from_script(script: &ScriptIr) -> Result<Option<Self>, EmitError> {
        let mut plan: Option<Self> = None;
        for body in script.executable_script_bodies() {
            for statement in &body.statements {
                let StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ModuleExecutionGraph(graph),
                    ..
                }) = statement
                else {
                    continue;
                };
                if graph.initializer().is_none() {
                    return Err(EmitError::unsupported(
                        "root module graph requires its reusable initializer",
                    ));
                }
                if graph.realm_import_dispatcher().is_none() {
                    return Err(EmitError::unsupported(
                        "reusable module graph requires its intrinsic import-job dispatcher",
                    ));
                }
                if let Some(previous) = &plan {
                    if previous.graph != **graph {
                        return Err(EmitError::unsupported(
                            "one Realm module registry requires one canonical compiled graph",
                        ));
                    }
                } else {
                    plan = Some(Self {
                        graph: (**graph).clone(),
                    });
                }
            }
        }
        Ok(plan)
    }

    pub(crate) fn record_count(&self) -> u32 {
        self.graph.record_count()
    }
    pub(crate) fn initializer(&self) -> &FunctionId {
        self.graph
            .initializer()
            .expect("plan validated a reusable initializer")
    }
    pub(crate) fn realm_requests(
        &self,
    ) -> &BTreeMap<lila_ir::ModuleRequestKeyIr, lila_ir::RealmModuleResolutionIr> {
        self.graph.realm_requests()
    }
    pub(crate) fn realm_import_dispatcher(&self) -> &FunctionId {
        self.graph
            .realm_import_dispatcher()
            .expect("canonical graph owns a Realm dispatcher")
    }
}
