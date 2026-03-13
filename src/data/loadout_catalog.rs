use crate::data::module_catalog::ModuleCatalog;
use crate::data::ship_catalog::ShipCatalog;
use crate::domain::loadout::ShipLoadout;
use crate::domain::ship::{Ship, Slot};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LoadoutCatalogFile {
    loadouts: Vec<ShipLoadout>,
}

#[derive(Debug, Clone)]
pub struct LoadoutCatalog {
    by_id: BTreeMap<String, ShipLoadout>,
}

impl LoadoutCatalog {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        let parsed: LoadoutCatalogFile =
            ron::de::from_str(input).map_err(|err| format!("failed to parse loadout RON: {err}"))?;

        let mut by_id = BTreeMap::new();
        let mut seen_ids = HashSet::new();

        for loadout in parsed.loadouts {
            loadout.validate()?;
            if !seen_ids.insert(loadout.id.clone()) {
                return Err(format!("duplicate loadout id '{}'", loadout.id));
            }
            by_id.insert(loadout.id.clone(), loadout);
        }

        Ok(Self { by_id })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read loadout catalog '{}': {err}", path.display()))?;
        Self::from_ron_str(&text)
    }

    pub fn get(&self, id: &str) -> Option<&ShipLoadout> {
        self.by_id.get(id)
    }

    pub fn instantiate_ship(
        &self,
        loadout_id: &str,
        ship_catalog: &ShipCatalog,
        module_catalog: &ModuleCatalog,
    ) -> Result<Ship, String> {
        let loadout = self
            .get(loadout_id)
            .ok_or_else(|| format!("unknown loadout id '{loadout_id}'"))?;

        let ship_def = ship_catalog
            .get(&loadout.ship_id)
            .ok_or_else(|| format!("unknown ship id '{}'", loadout.ship_id))?;

        let mut ship = Ship::new(ship_def.grid_size);
        let mut used_indices = HashSet::new();
        let mut module_count = 0usize;

        for slot_cfg in &loadout.slots {
            let idx = ship
                .index_of(slot_cfg.x, slot_cfg.y)
                .ok_or_else(|| {
                    format!(
                        "loadout '{}' has slot outside ship grid: ({}, {})",
                        loadout.id, slot_cfg.x, slot_cfg.y
                    )
                })?;

            if !used_indices.insert(idx) {
                return Err(format!(
                    "loadout '{}' has duplicate slot entry at ({}, {})",
                    loadout.id, slot_cfg.x, slot_cfg.y
                ));
            }

            let slot = if let Some(module_id) = &slot_cfg.module_id {
                module_count += 1;
                let module = module_catalog.spawn_instance(module_id)?;
                Slot::with_module(slot_cfg.armor_layers, module)
            } else {
                Slot::empty(slot_cfg.armor_layers)
            };

            ship.set_slot(slot_cfg.x, slot_cfg.y, slot)?;
        }

        if module_count > ship_def.max_modules {
            return Err(format!(
                "loadout '{}' has {module_count} modules, but ship '{}' supports at most {}",
                loadout.id, ship_def.id, ship_def.max_modules
            ));
        }

        Ok(ship)
    }
}

#[cfg(test)]
mod tests {
    use super::LoadoutCatalog;
    use crate::data::module_catalog::ModuleCatalog;
    use crate::data::ship_catalog::ShipCatalog;

    #[test]
    fn builds_corvette_single_gun_loadout() {
        let ship_ron = r#"
(
    ships: [
        (id: "corvette", display_name: "Corvette", grid_size: 1, max_modules: 1),
    ],
)
"#;
        let module_ron = r#"
(
    modules: [
        (
            id: "gun",
            display_name: "Gun",
            kind: Gun,
            max_hp: 80,
            attributes: {},
        ),
    ],
)
"#;
        let loadout_ron = r#"
(
    loadouts: [
        (
            id: "corvette_single_gun",
            ship_id: "corvette",
            slots: [
                (x: 0, y: 0, armor_layers: 1, module_id: Some("gun")),
            ],
        ),
    ],
)
"#;

        let ships = ShipCatalog::from_ron_str(ship_ron).expect("ship catalog should parse");
        let modules = ModuleCatalog::from_ron_str(module_ron).expect("module catalog should parse");
        let loadouts = LoadoutCatalog::from_ron_str(loadout_ron).expect("loadout catalog should parse");

        let ship = loadouts
            .instantiate_ship("corvette_single_gun", &ships, &modules)
            .expect("loadout should instantiate");
        assert_eq!(ship.size, 1);
        assert_eq!(ship.total_modules(), 1);

        let slot = ship.slot(0, 0).expect("slot should exist");
        assert_eq!(slot.armor_layers, 1);
        assert!(slot.module.is_some());
    }
}
