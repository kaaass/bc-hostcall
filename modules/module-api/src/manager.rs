use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rpc::abi;
use crate::module::WasmModule;

pub struct ModuleManager {
    modules: Mutex<Cell<HashMap<abi::LinkHint, Arc<WasmModule>>>>
}

impl ModuleManager {
    pub fn new() -> Self {
        ModuleManager {
            modules: Mutex::new(Cell::new(HashMap::new()))
        }
    }

    pub fn resolve(&self, link_hint: &abi::LinkHint) -> Option<Arc<WasmModule>> {
        let mut modules = self.modules.lock().unwrap();

        modules.get_mut().get(link_hint).cloned()
    }

    pub fn register(&self, link_hint: abi::LinkHint, module: Arc<WasmModule>) -> Option<Arc<WasmModule>> {
        let mut modules = self.modules.lock().unwrap();

        modules.get_mut().insert(link_hint, module)
    }

    pub fn unregister(&self, link_hint: &abi::LinkHint) -> Option<Arc<WasmModule>> {
        let mut modules = self.modules.lock().unwrap();

        modules.get_mut().remove(link_hint)
    }

    pub fn list_modules(&self) -> Vec<abi::LinkHint> {
        let mut modules = self.modules.lock().unwrap();

        modules.get_mut().keys().cloned().collect()
    }
}
