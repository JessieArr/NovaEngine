use crate::domain::loadout::ShipDefinition;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ShipCatalogFile {
    ships: Vec<ShipDefinition>,
}

#[derive(Debug, Clone)]
pub struct ShipCatalog {
    by_id: BTreeMap<String, ShipDefinition>,
}

impl ShipCatalog {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        let parsed: ShipCatalogFile =
            ron::de::from_str(input).map_err(|err| format!("failed to parse ship RON: {err}"))?;

        let mut by_id = BTreeMap::new();
        let mut seen_ids = HashSet::new();

        for ship in parsed.ships {
            ship.validate()?;
            if !seen_ids.insert(ship.id.clone()) {
                return Err(format!("duplicate ship id '{}'", ship.id));
            }
            by_id.insert(ship.id.clone(), ship);
        }

        Ok(Self { by_id })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read ship catalog '{}': {err}", path.display()))?;
        Self::from_ron_str(&text)
    }

    pub fn get(&self, id: &str) -> Option<&ShipDefinition> {
        self.by_id.get(id)
    }
}
