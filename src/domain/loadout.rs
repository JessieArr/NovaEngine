use crate::domain::ship::MAX_ARMOR_LAYERS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipDefinition {
    pub id: String,
    pub display_name: String,
    pub grid_size: usize,
    pub max_modules: usize,
}

impl ShipDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("ship id cannot be empty".to_owned());
        }
        if self.grid_size == 0 {
            return Err(format!("ship '{}' has invalid grid_size=0", self.id));
        }

        let slot_capacity = self.grid_size * self.grid_size;
        if self.max_modules == 0 || self.max_modules > slot_capacity {
            return Err(format!(
                "ship '{}' has invalid max_modules={}, capacity={slot_capacity}",
                self.id, self.max_modules
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotLoadout {
    pub x: usize,
    pub y: usize,
    pub armor_layers: u8,
    pub module_id: Option<String>,
}

impl SlotLoadout {
    pub fn validate(&self) -> Result<(), String> {
        if self.armor_layers > MAX_ARMOR_LAYERS {
            return Err(format!(
                "slot ({}, {}) has invalid armor_layers={} (max={MAX_ARMOR_LAYERS})",
                self.x, self.y, self.armor_layers
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipLoadout {
    pub id: String,
    pub ship_id: String,
    pub slots: Vec<SlotLoadout>,
}

impl ShipLoadout {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("loadout id cannot be empty".to_owned());
        }
        if self.ship_id.trim().is_empty() {
            return Err(format!("loadout '{}' has empty ship_id", self.id));
        }

        for slot in &self.slots {
            slot.validate()?;
        }

        Ok(())
    }
}
