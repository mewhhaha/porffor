use std::sync::Arc;

pub trait HostHooks: Send + Sync {
    fn shell_name(&self) -> &'static str {
        "porffor-shell"
    }

    fn print_line(&self, _text: &str) {}
}

#[derive(Debug, Default)]
pub struct NullHostHooks;

impl HostHooks for NullHostHooks {}

#[derive(Clone)]
pub struct Realm {
    pub shell_name: String,
    host_hooks: Arc<dyn HostHooks>,
}

impl core::fmt::Debug for Realm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Realm")
            .field("shell_name", &self.shell_name)
            .finish()
    }
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

    pub fn host_hooks(&self) -> &dyn HostHooks {
        &*self.host_hooks
    }

    pub fn build(self) -> Realm {
        Realm {
            shell_name: self.host_hooks.shell_name().to_string(),
            host_hooks: Arc::from(self.host_hooks),
        }
    }
}

impl Realm {
    pub fn host_hooks(&self) -> Arc<dyn HostHooks> {
        Arc::clone(&self.host_hooks)
    }
}
