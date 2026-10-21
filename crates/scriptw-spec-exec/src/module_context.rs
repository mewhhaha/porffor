//! Entry and filesystem module admission under the selected loader policy.
use super::*;

/// Builds every Script-capable context under the session's closed loader
/// policy. Boa otherwise defaults to a `SimpleModuleLoader` rooted at the
/// process working directory, which would let dynamic source escape a replay's
/// serialized inputs.
pub(super) fn build_host_context(
    can_block: bool,
    module_loading_policy: ModuleLoadingPolicy,
) -> JsResult<Context> {
    match module_loading_policy {
        ModuleLoadingPolicy::Filesystem => Context::builder()
            .job_executor(Rc::new(SimpleJobExecutor::new()))
            .module_loader(Rc::new(oracle_exception::AmbientModuleLoader::new()?))
            .can_block(can_block)
            .build(),
        ModuleLoadingPolicy::RejectAll => Context::builder()
            .job_executor(Rc::new(SimpleJobExecutor::new()))
            .module_loader(Rc::new(oracle_exception::DisabledModuleLoader))
            .can_block(can_block)
            .build(),
        ModuleLoadingPolicy::Embedded(graph) => {
            EmbeddedGraphModuleLoader::context(graph, can_block).map(|(context, _)| context)
        }
    }
}

pub(super) fn root_source_name(
    source: &str,
    filename: Option<&str>,
    goal: EmbeddedModuleGoal,
    policy: &ModuleLoadingPolicy,
) -> Result<Option<String>, ExecutionError> {
    match policy {
        ModuleLoadingPolicy::Filesystem | ModuleLoadingPolicy::RejectAll => {
            Ok(filename.map(str::to_owned))
        }
        ModuleLoadingPolicy::Embedded(graph) => {
            let entry = graph.entry();
            if entry.goal() != goal
                || entry.source() != source
                || filename.is_some_and(|identity| identity != entry.identity())
            {
                return Err(ExecutionError::new(
                    "embedded execution source, goal and identity must match the graph entry",
                ));
            }
            Ok(Some(entry.identity().to_owned()))
        }
    }
}

/// The actual Script producer for both public entry consumers owns entry
/// admission together with parsing, so neither caller separately mints a root
/// Script whose locator/source disagrees with the shared embedded policy.
pub(super) fn parse_entry_script(
    source: &str,
    filename: Option<&str>,
    policy: &ModuleLoadingPolicy,
    context: &mut Context,
) -> Result<Script, ExecutionError> {
    let filename = root_source_name(source, filename, EmbeddedModuleGoal::Script, policy)?;
    let source = match policy {
        ModuleLoadingPolicy::Filesystem | ModuleLoadingPolicy::RejectAll => source,
        ModuleLoadingPolicy::Embedded(graph) => graph.entry().source(),
    };
    Script::parse(source_with_name(source, filename.as_deref()), None, context)
        .map_err(|error| ExecutionError::entry_parse(error, context))
}

/// Both Module entry consumers obtain a parsed/cache-owned root here. The
/// Embedded arm reads only the validated source owner; its parsed root cannot
/// be omitted from the loader cache before graph loading or import.meta.
pub(super) fn prepare_module_context(
    source: &str,
    filename: Option<&str>,
    host: &ModuleHostConfig,
    argv: &[String],
    can_block: bool,
) -> Result<(Context, Module, Option<PathBuf>), ExecutionError> {
    match &host.module_loading_policy {
        ModuleLoadingPolicy::Filesystem => {
            let module_path = normalize_module_path(filename).or_else(|| host.test_path.clone());
            let loader = Rc::new(Test262ModuleLoader::new(
                host.module_root.as_deref(),
                module_path.as_deref(),
            ));
            let mut context = Context::builder()
                .job_executor(Rc::new(SimpleJobExecutor::new()))
                .module_loader(loader.clone())
                .can_block(can_block)
                .build()
                .map_err(|err| ExecutionError::new(err.to_string()))?;
            install_host_globals(&mut context, argv)?;
            let module = Module::parse(source_with_name(source, filename), None, &mut context)
                .map_err(|err| ExecutionError::entry_parse(err, &mut context))?;
            loader.insert(
                module_path.clone().unwrap_or_else(|| loader.entry_path()),
                LoadedModuleKind::Source,
                module.clone(),
            );
            Ok((context, module, module_path))
        }
        ModuleLoadingPolicy::RejectAll => Err(ExecutionError::new(
            "module loading disabled by host policy",
        )),
        ModuleLoadingPolicy::Embedded(graph) => {
            let identity = root_source_name(
                source,
                filename,
                EmbeddedModuleGoal::Module,
                &host.module_loading_policy,
            )?
            .expect("an Embedded root has an immutable declared identity");
            let (mut context, loader) =
                EmbeddedGraphModuleLoader::context(Arc::clone(graph), can_block)
                    .map_err(|err| ExecutionError::new(err.to_string()))?;
            install_host_globals(&mut context, argv)?;
            let module = loader
                .entry_module(&mut context)
                .map_err(|err| ExecutionError::entry_parse(err, &mut context))?;
            Ok((context, module, Some(PathBuf::from(identity))))
        }
    }
}

#[derive(Debug, Default)]
struct Test262ModuleLoader {
    root: PathBuf,
    entry_path: PathBuf,
    module_map: RefCell<BTreeMap<(PathBuf, LoadedModuleKind), Module>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LoadedModuleKind {
    Source,
    Json,
    Text,
    Bytes,
}

impl Test262ModuleLoader {
    fn new(module_root: Option<&Path>, test_path: Option<&Path>) -> Self {
        let entry_path = test_path
            .map(normalize_absolute_path)
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from("main.mjs")));
        let root = module_root
            .map(normalize_absolute_path)
            .or_else(|| entry_path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from(".")));
        Self {
            root,
            entry_path,
            module_map: RefCell::new(BTreeMap::new()),
        }
    }

    fn entry_path(&self) -> PathBuf {
        self.entry_path.clone()
    }

    fn insert(&self, path: PathBuf, kind: LoadedModuleKind, module: Module) {
        self.module_map.borrow_mut().insert((path, kind), module);
    }

    fn get(&self, path: &Path, kind: LoadedModuleKind) -> Option<Module> {
        self.module_map
            .borrow()
            .get(&(path.to_path_buf(), kind))
            .cloned()
    }

    fn resolve_path(&self, referrer: Referrer, specifier: &JsString) -> JsResult<PathBuf> {
        let specifier = specifier.to_std_string_escaped();
        let specifier_path = Path::new(&specifier);
        if specifier_path.is_absolute() {
            return Ok(normalize_absolute_path(specifier_path));
        }

        let base_dir = if specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier == "."
            || specifier == ".."
        {
            referrer
                .path()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| {
                    self.entry_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.root.clone())
                })
        } else {
            self.root.clone()
        };

        Ok(normalize_absolute_path(&base_dir.join(specifier_path)))
    }

    fn request_kind(request: &ModuleRequest) -> JsResult<LoadedModuleKind> {
        let mut type_value = None;
        for attribute in request.attributes() {
            if attribute.key().to_std_string_escaped() == "type" {
                type_value = Some(attribute.value().to_std_string_escaped());
                break;
            }
        }

        match type_value.as_deref() {
            None | Some("") => Ok(LoadedModuleKind::Source),
            Some("json") => Ok(LoadedModuleKind::Json),
            Some("text") => Ok(LoadedModuleKind::Text),
            Some("bytes") => Ok(LoadedModuleKind::Bytes),
            Some(other) => Err(JsNativeError::syntax()
                .with_message(format!("unsupported import attribute type `{other}`"))
                .into()),
        }
    }
}

impl ModuleLoader for Test262ModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let result = (|| {
            let specifier = request.specifier();
            let short_path = specifier.to_std_string_escaped();
            let path = self.resolve_path(referrer, &specifier)?;
            let kind = Self::request_kind(&request)?;
            if let Some(module) = self.get(&path, kind) {
                return Ok(module);
            }

            let module = match kind {
                LoadedModuleKind::Source => {
                    let source = std::fs::read_to_string(&path).map_err(|err| {
                        JsNativeError::typ()
                            .with_message(format!("could not open file `{short_path}`"))
                            .with_cause(boa_engine::JsError::from_opaque(
                                js_string!(err.to_string()).into(),
                            ))
                    })?;
                    let should_consider_harness = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| !name.contains("_FIXTURE"))
                        .unwrap_or(true);
                    let inject_harness = should_consider_harness
                        && (source.contains("assert.")
                            || source.contains("assert(")
                            || source.contains("Test262Error"));
                    let source = if inject_harness {
                        format!("{TEST262_STA_GLOBAL}\n{TEST262_ASSERT_GLOBAL}\n{source}")
                    } else {
                        source
                    };
                    let path_string = path.to_string_lossy().into_owned();
                    Module::parse(
                        source_with_name(&source, Some(&path_string)),
                        None,
                        &mut context.borrow_mut(),
                    )
                    .map_err(|err| {
                        JsNativeError::syntax()
                            .with_message(format!("could not parse module `{short_path}`"))
                            .with_cause(err)
                    })?
                }
                LoadedModuleKind::Json => {
                    let source = std::fs::read_to_string(&path).map_err(|err| {
                        JsNativeError::typ()
                            .with_message(format!("could not open file `{short_path}`"))
                            .with_cause(boa_engine::JsError::from_opaque(
                                js_string!(err.to_string()).into(),
                            ))
                    })?;
                    Module::parse_json(js_string!(source), &mut context.borrow_mut()).map_err(
                        |err| {
                            JsNativeError::syntax()
                                .with_message(format!("could not parse module `{short_path}`"))
                                .with_cause(err)
                        },
                    )?
                }
                LoadedModuleKind::Text => {
                    let source = std::fs::read_to_string(&path).map_err(|err| {
                        JsNativeError::typ()
                            .with_message(format!("could not open file `{short_path}`"))
                            .with_cause(boa_engine::JsError::from_opaque(
                                js_string!(err.to_string()).into(),
                            ))
                    })?;
                    Module::from_value_as_default(
                        JsValue::from(js_string!(source)),
                        &mut context.borrow_mut(),
                    )
                }
                LoadedModuleKind::Bytes => {
                    let source = std::fs::read(&path).map_err(|err| {
                        JsNativeError::typ()
                            .with_message(format!("could not open file `{short_path}`"))
                            .with_cause(boa_engine::JsError::from_opaque(
                                js_string!(err.to_string()).into(),
                            ))
                    })?;
                    let bytes = AlignedVec::from_iter(0, source);
                    let value = {
                        let mut context = context.borrow_mut();
                        let array_buffer =
                            JsArrayBuffer::from_byte_block_immutable(bytes, &mut context)?;
                        let array = JsUint8Array::from_array_buffer(array_buffer, &mut context)?;
                        JsValue::from(array)
                    };
                    Module::from_value_as_default(value, &mut context.borrow_mut())
                }
            };
            self.insert(path, kind, module.clone());
            Ok(module)
        })();
        result.map_err(|error| oracle_exception::host_failure(error, &mut context.borrow_mut()))
    }
}

pub(super) fn source_with_name<'a>(
    source_text: &'a str,
    filename: Option<&'a str>,
) -> Source<'a, boa_engine::parser::source::UTF8Input<&'a [u8]>> {
    let source = Source::from_bytes(source_text);
    match filename {
        Some(path) => source.with_path(Path::new(path)),
        None => source,
    }
}

fn normalize_module_path(filename: Option<&str>) -> Option<PathBuf> {
    filename
        .map(PathBuf::from)
        .map(|path| normalize_absolute_path(&path))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
