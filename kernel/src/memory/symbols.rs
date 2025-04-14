use crate::memory::{bcmp, memcmp, memcpy, memmove, memset};
use alloc::collections::BTreeMap;
use alloc::string::String;
use lazy_static::lazy_static;
use log::debug;
use spinning_top::Spinlock;

lazy_static! {
    pub static ref SYMBOL_REGISTRY: Spinlock<SymbolRegistry> = {
        let mut registry = SymbolRegistry::new();
        registry.initialize();

        Spinlock::new(registry)
    };
}

pub struct SymbolRegistry {
    symbols: BTreeMap<String, usize>,
}

impl SymbolRegistry {
    pub const fn new() -> Self {
        Self {
            symbols: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.register("memset", memset as usize);
        self.register("memcpy", memcpy as usize);
        self.register("memmove", memmove as usize);
        self.register("memcmp", memcmp as usize);
        self.register("bcmp", bcmp as usize);

        debug!(
            "Symbol registry initialized with {} entries",
            self.symbols.len()
        );
    }

    pub fn register(&mut self, name: &str, addr: usize) {
        self.symbols.insert(String::from(name), addr);
    }

    pub fn resolve(&self, name: &str) -> Option<usize> {
        self.symbols.get(name).copied()
    }

    pub fn dump_symbols(&self) {
        debug!("Registered symbols:");
        for (name, addr) in &self.symbols {
            debug!("  {}: {:#x}", name, addr);
        }
    }
}
