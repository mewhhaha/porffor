pub trait HostHooks: Send + Sync {
    fn shell_name(&self) -> &'static str {
        "porffor-shell"
    }
}

#[derive(Debug, Default)]
pub struct NullHostHooks;

impl HostHooks for NullHostHooks {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Realm {
    pub shell_name: String,
}

pub struct RealmBuilder {
    host_hooks: Box<dyn HostHooks>,
}

impl Default for RealmBuilder {
    fn default() -> Self {
        Self {
            host_hooks: Box::<NullHostHooks>::default(),
        }
    }
}

impl RealmBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host_hooks(mut self, host_hooks: Box<dyn HostHooks>) -> Self {
        self.host_hooks = host_hooks;
        self
    }

    pub fn build(self) -> Realm {
        Realm {
            shell_name: self.host_hooks.shell_name().to_string(),
        }
    }
}
