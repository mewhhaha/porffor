//! Boa context-local Module instances backed only by the shared source owner.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use boa_engine::job::SimpleJobExecutor;
use boa_engine::module::{Module, ModuleLoader, ModuleRequest, ModuleRequestPhase, Referrer};
use boa_engine::realm::Realm;
use boa_engine::{js_string, Context, JsNativeError, JsObject, JsResult, JsString};
use lila_runtime::{
    EmbeddedModuleGoal, EmbeddedModuleGraph, EmbeddedModuleKind, EmbeddedModuleReferrer,
    EmbeddedModuleRequest,
};

use crate::source_with_name;

#[derive(Debug)]
pub(crate) struct EmbeddedGraphModuleLoader {
    graph: Arc<EmbeddedModuleGraph>,
    // Each actual Realm owns its parsed instances, while all Realms in this
    // Context share the one interner used by Boa's parser and linker.
    realms: RefCell<Vec<Rc<EmbeddedRealmModules>>>,
}

#[derive(Debug)]
struct EmbeddedRealmModules {
    realm: Realm,
    modules: RefCell<BTreeMap<String, Module>>,
}

impl EmbeddedGraphModuleLoader {
    /// The only Embedded Context factory. The root Realm's cache is registered
    /// after successful construction of its actual parser/linker Context.
    pub(crate) fn context(
        graph: Arc<EmbeddedModuleGraph>,
        can_block: bool,
    ) -> JsResult<(Context, Rc<Self>)> {
        let loader = Rc::new(Self {
            graph,
            realms: RefCell::new(Vec::new()),
        });
        let context = Context::builder()
            .job_executor(Rc::new(SimpleJobExecutor::new()))
            .module_loader(loader.clone())
            .can_block(can_block)
            .build()?;
        loader.register_realm(context.realm().clone());
        Ok((context, loader))
    }

    /// A created Embedded Realm must reuse this actual Context/interner. A
    /// second Context with the same graph would give cached AST symbols a
    /// different meaning when a foreign callback links or resolves exports.
    pub(crate) fn create_realm(
        graph: &Arc<EmbeddedModuleGraph>,
        context: &mut Context,
    ) -> JsResult<Realm> {
        let loader = context
            .downcast_module_loader::<Self>()
            .filter(|loader| Arc::ptr_eq(&loader.graph, graph))
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("embedded Realm requires its session-owned Context loader")
            })?;
        let realm = context.create_realm()?;
        loader.register_realm(realm.clone());
        Ok(realm)
    }

    fn register_realm(&self, realm: Realm) {
        self.realms.borrow_mut().push(Rc::new(EmbeddedRealmModules {
            realm,
            modules: RefCell::new(BTreeMap::new()),
        }));
    }

    fn realm_modules(&self, realm: &Realm) -> JsResult<Rc<EmbeddedRealmModules>> {
        self.realms
            .borrow()
            .iter()
            .find(|owner| &owner.realm == realm)
            .cloned()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("embedded referrer Realm has no Context-owned module cache")
                    .into()
            })
    }

    /// Mint and cache the actual root Module through the same producer as all
    /// imports, so a cycle importing the entry retains its Module identity.
    pub(crate) fn entry_module(&self, context: &mut Context) -> JsResult<Module> {
        if self.graph.entry().goal() != EmbeddedModuleGoal::Module {
            return Err(JsNativeError::typ()
                .with_message("embedded Module execution requires a Module entry")
                .into());
        }
        let owner = self.realm_modules(context.realm())?;
        self.module(self.graph.entry().identity(), &owner, context)
    }

    fn module(
        &self,
        identity: &str,
        owner: &EmbeddedRealmModules,
        context: &mut Context,
    ) -> JsResult<Module> {
        if let Some(module) = owner.modules.borrow().get(identity).cloned() {
            return Ok(module);
        }
        let source = self.graph.module(identity).ok_or_else(|| {
            JsNativeError::typ().with_message("embedded module source is not declared")
        })?;
        let module = match source.kind() {
            EmbeddedModuleKind::SourceText => Module::parse(
                source_with_name(source.source(), Some(source.identity())),
                Some(owner.realm.clone()),
                context,
            )?,
            EmbeddedModuleKind::Json => {
                // Boa's JSON factory takes its Realm from Context. A borrowed
                // foreign import callback must still allocate its data and any
                // parse error in the actual referrer module's Realm.
                let previous = context.enter_realm(owner.realm.clone());
                let result = Module::parse_json(js_string!(source.source()), context)
                    .map_err(|error| crate::oracle_exception::host_failure(error, context));
                context.enter_realm(previous);
                result?
            }
        };
        owner
            .modules
            .borrow_mut()
            .insert(identity.to_owned(), module.clone());
        Ok(module)
    }

    fn module_identity(&self, module: &Module) -> JsResult<String> {
        let owner = self.realm_modules(module.realm())?;
        if let Some(identity) = owner
            .modules
            .borrow()
            .iter()
            .find_map(|(identity, cached)| (cached == module).then(|| identity.clone()))
        {
            return Ok(identity);
        }
        // All Module producers in this Context assign an exact source-owner
        // locator. After the actual Realm/cache fast path, this locator may
        // project only an exact declared source. It is never normalized,
        // joined or inferred from import.meta.url.
        let identity = module
            .path()
            .and_then(|path| path.to_str())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("embedded Module has no exact UTF-8 source identity")
            })?;
        if self.graph.module(identity).is_none() {
            return Err(JsNativeError::typ()
                .with_message("embedded Module source identity is not declared")
                .into());
        }
        Ok(identity.to_owned())
    }

    fn referrer(&self, referrer: &Referrer) -> JsResult<EmbeddedModuleReferrer> {
        match referrer {
            Referrer::Module(module) => self
                .module_identity(module)
                .map(EmbeddedModuleReferrer::Module),
            Referrer::Script(_) => match referrer.path() {
                // The root factory supplies this exact virtual locator after
                // source/goal/identity validation. It is not canonicalized or
                // joined with a specifier; an unknown locator simply has no row.
                Some(path) => path
                    .to_str()
                    .map(|identity| EmbeddedModuleReferrer::Script(identity.to_owned()))
                    .ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("embedded Script locator is not a UTF-8 identity")
                            .into()
                    }),
                None => Ok(EmbeddedModuleReferrer::Unlocated),
            },
            Referrer::Realm(_) => Ok(EmbeddedModuleReferrer::Unlocated),
        }
    }
}

fn exact_js_string(value: &JsString) -> JsResult<String> {
    // Escaping a lone surrogate would alias an actual backslash-u request in
    // the corpus. Such a string has no representable declared UTF-8 row.
    value.to_std_string().map_err(|_| {
        JsNativeError::typ()
            .with_message("embedded module request contains an unpaired UTF-16 surrogate")
            .into()
    })
}

impl ModuleLoader for EmbeddedGraphModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let realm = match &referrer {
            Referrer::Module(module) => module.realm(),
            Referrer::Script(script) => script.realm(),
            Referrer::Realm(realm) => realm,
        }
        .clone();
        // Host request validation, parsing and error materialization all belong
        // to the exact referrer Realm, including borrowed foreign callbacks.
        let mut context = context.borrow_mut();
        let previous = context.enter_realm(realm.clone());
        let result = (|| {
            let owner = self.realm_modules(&realm)?;
            let referrer = self.referrer(&referrer)?;
            let phase = request.phase();
            let specifier = exact_js_string(&request.specifier())?;
            let attributes = request
                .attributes()
                .iter()
                .map(|attribute| {
                    Ok((
                        exact_js_string(&attribute.key())?,
                        exact_js_string(&attribute.value())?,
                    ))
                })
                .collect::<JsResult<Vec<_>>>()?;
            let request =
                EmbeddedModuleRequest::try_new(specifier, attributes).map_err(|error| {
                    JsNativeError::typ()
                        .with_message(format!("invalid embedded module request: {error}"))
                })?;
            let target = self.graph.resolve(&referrer, &request).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("module request is not declared in embedded graph")
            })?;
            // The exact host key is phase-free, but these declared source kinds
            // have no Module Source Object. Refuse both static and dynamic
            // Source requests after original attribute validation/source parsing.
            let module = self.module(target.identity(), &owner, &mut context)?;
            if phase == ModuleRequestPhase::Source {
                return Err(JsNativeError::syntax()
                    .with_message("source phase imports are unavailable for embedded SourceText and JSON modules")
                    .into());
            }
            Ok(module)
        })();
        let result =
            result.map_err(|error| crate::oracle_exception::host_failure(error, &mut context));
        context.enter_realm(previous);
        result
    }

    fn init_import_meta(
        self: Rc<Self>,
        import_meta: &JsObject,
        module: &Module,
        context: &mut Context,
    ) {
        let Ok(identity) = self.module_identity(module) else {
            // Boa's hook is non-fallible. An unowned native Module cannot be
            // produced by this session; expose no fabricated URL for it.
            return;
        };
        let Some(source) = self.graph.module(&identity) else {
            return;
        };
        import_meta
            .create_data_property_or_throw(
                js_string!("url"),
                js_string!(source.meta_url()),
                context,
            )
            .expect("Boa supplies a fresh extensible null-prototype import.meta object");
    }
}

#[cfg(test)]
mod tests;
