use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleKind {
    ShieldGenerator,
    Gun,
    MissileLauncher,
    Sensor,
    Utility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleArchetype {
    pub id: String,
    pub display_name: String,
    pub kind: ModuleKind,
    pub max_hp: u32,
    #[serde(default)]
    pub attributes: BTreeMap<String, f32>,
}

impl ModuleArchetype {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("module id cannot be empty".to_owned());
        }
        if self.max_hp == 0 {
            return Err(format!("module '{}' has invalid max_hp=0", self.id));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub archetype_id: String,
    pub hp: u32,
}

impl ModuleInstance {
    pub fn from_archetype(archetype: &ModuleArchetype) -> Self {
        Self {
            archetype_id: archetype.id.clone(),
            hp: archetype.max_hp,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.hp == 0
    }
}
