use crate::domain::module::{ModuleArchetype, ModuleInstance};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ModuleCatalogFile {
    modules: Vec<ModuleArchetype>,
}

#[derive(Debug, Clone)]
pub struct ModuleCatalog {
    by_id: BTreeMap<String, ModuleArchetype>,
}

impl ModuleCatalog {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        let parsed: ModuleCatalogFile =
            ron::de::from_str(input).map_err(|err| format!("failed to parse module RON: {err}"))?;

        let mut by_id = BTreeMap::new();
        let mut seen_ids = HashSet::new();

        for module in parsed.modules {
            module.validate()?;

            if !seen_ids.insert(module.id.clone()) {
                return Err(format!("duplicate module id '{}'", module.id));
            }

            by_id.insert(module.id.clone(), module);
        }

        Ok(Self { by_id })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read module catalog '{}': {err}", path.display()))?;
        Self::from_ron_str(&text)
    }

    pub fn get(&self, id: &str) -> Option<&ModuleArchetype> {
        self.by_id.get(id)
    }

    pub fn spawn_instance(&self, id: &str) -> Result<ModuleInstance, String> {
        let archetype = self
            .get(id)
            .ok_or_else(|| format!("unknown module id '{id}'"))?;
        Ok(ModuleInstance::from_archetype(archetype))
    }

    pub fn module_count(&self) -> usize {
        self.by_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleCatalog;

    #[test]
    fn parses_valid_module_catalog() {
        let ron = r#"
(
    modules: [
        (
            id: "gun_light",
            display_name: "Light Gun",
            kind: Gun,
            max_hp: 80,
            attributes: { "range": 12.0, "fire_rate": 1.8 },
        ),
        (
            id: "sensor_basic",
            display_name: "Basic Sensor",
            kind: Sensor,
            max_hp: 60,
            attributes: { "scan_range": 30.0 },
        ),
    ],
)
"#;

        let catalog = ModuleCatalog::from_ron_str(ron).expect("catalog should parse");
        assert_eq!(catalog.module_count(), 2);
        assert!(catalog.get("gun_light").is_some());
    }

    #[test]
    fn rejects_duplicate_ids() {
        let ron = r#"
(
    modules: [
        (id: "dup", display_name: "A", kind: Gun, max_hp: 10, attributes: {}),
        (id: "dup", display_name: "B", kind: Sensor, max_hp: 10, attributes: {}),
    ],
)
"#;

        let err = ModuleCatalog::from_ron_str(ron).expect_err("catalog should reject duplicates");
        assert!(err.contains("duplicate module id"));
    }
}
