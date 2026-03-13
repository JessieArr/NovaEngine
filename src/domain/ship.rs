use crate::domain::module::ModuleInstance;

pub const MAX_ARMOR_LAYERS: u8 = 3;
pub const ARMOR_HP_PER_LAYER: u32 = 25;

#[derive(Debug, Clone)]
pub struct Slot {
    pub armor_layers: u8,
    pub armor_hp: u32,
    pub module: Option<ModuleInstance>,
}

impl Slot {
    pub fn empty(armor_layers: u8) -> Self {
        let layers = armor_layers.min(MAX_ARMOR_LAYERS);
        Self {
            armor_layers: layers,
            armor_hp: u32::from(layers) * ARMOR_HP_PER_LAYER,
            module: None,
        }
    }

    pub fn with_module(armor_layers: u8, module: ModuleInstance) -> Self {
        let mut slot = Self::empty(armor_layers);
        slot.module = Some(module);
        slot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitReport {
    pub slot_index: usize,
    pub target_had_module: bool,
    pub armor_damage: u32,
    pub module_damage: u32,
    pub module_destroyed: bool,
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub size: usize,
    pub slots: Vec<Slot>,
}

impl Ship {
    pub fn new(size: usize) -> Self {
        let slot_count = size * size;
        let mut slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            slots.push(Slot::empty(0));
        }
        Self { size, slots }
    }

    pub fn index_of(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.size && y < self.size {
            Some((y * self.size) + x)
        } else {
            None
        }
    }

    pub fn slot(&self, x: usize, y: usize) -> Option<&Slot> {
        self.index_of(x, y).and_then(|idx| self.slots.get(idx))
    }

    pub fn slot_mut(&mut self, x: usize, y: usize) -> Option<&mut Slot> {
        self.index_of(x, y).and_then(move |idx| self.slots.get_mut(idx))
    }

    pub fn set_slot(&mut self, x: usize, y: usize, slot: Slot) -> Result<(), String> {
        let idx = self
            .index_of(x, y)
            .ok_or_else(|| format!("slot coordinates out of bounds: ({x}, {y})"))?;
        self.slots[idx] = slot;
        Ok(())
    }

    pub fn apply_hit(&mut self, slot_index: usize, damage: u32) -> Option<HitReport> {
        let slot = self.slots.get_mut(slot_index)?;
        let target_had_module = slot.module.is_some();
        if !target_had_module {
            return Some(HitReport {
                slot_index,
                target_had_module,
                armor_damage: 0,
                module_damage: 0,
                module_destroyed: false,
            });
        }

        let armor_damage = slot.armor_hp.min(damage);
        slot.armor_hp -= armor_damage;
        let mut remaining = damage - armor_damage;

        let mut module_damage = 0;
        let mut module_destroyed = false;

        if remaining > 0 {
            if let Some(module) = &mut slot.module {
                module_damage = module.hp.min(remaining);
                module.hp -= module_damage;
                remaining -= module_damage;
                if module.hp == 0 {
                    module_destroyed = true;
                }
            }
        }

        let _ = remaining;

        Some(HitReport {
            slot_index,
            target_had_module,
            armor_damage,
            module_damage,
            module_destroyed,
        })
    }

    pub fn total_modules(&self) -> usize {
        self.slots.iter().filter(|slot| slot.module.is_some()).count()
    }

    pub fn destroyed_modules(&self) -> usize {
        self.slots
            .iter()
            .filter_map(|slot| slot.module.as_ref())
            .filter(|module| module.is_destroyed())
            .count()
    }

    pub fn destroyed_module_ratio(&self) -> f32 {
        let total = self.total_modules();
        if total == 0 {
            0.0
        } else {
            self.destroyed_modules() as f32 / total as f32
        }
    }

    pub fn is_exploded(&self) -> bool {
        let total = self.total_modules();
        if total == 0 {
            return false;
        }
        self.destroyed_modules() * 4 >= total * 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::module::ModuleInstance;

    fn module_with_hp(id: &str, hp: u32) -> ModuleInstance {
        ModuleInstance {
            archetype_id: id.to_owned(),
            hp,
        }
    }

    #[test]
    fn armor_absorbs_damage_before_module_hp() {
        let mut ship = Ship::new(1);
        ship.set_slot(0, 0, Slot::with_module(2, module_with_hp("gun", 50)))
            .expect("slot should be valid");

        let report = ship.apply_hit(0, 30).expect("slot index should be valid");
        let slot = ship.slot(0, 0).expect("slot should exist");
        let module = slot.module.as_ref().expect("module should exist");

        assert_eq!(report.armor_damage, 30);
        assert_eq!(report.module_damage, 0);
        assert_eq!(module.hp, 50);
        assert_eq!(slot.armor_hp, 20);
    }

    #[test]
    fn module_destroys_when_hp_reaches_zero() {
        let mut ship = Ship::new(1);
        ship.set_slot(0, 0, Slot::with_module(0, module_with_hp("sensor", 40)))
            .expect("slot should be valid");

        let report = ship.apply_hit(0, 40).expect("slot index should be valid");
        let slot = ship.slot(0, 0).expect("slot should exist");
        let module = slot.module.as_ref().expect("module should exist");

        assert_eq!(report.module_damage, 40);
        assert!(report.module_destroyed);
        assert_eq!(module.hp, 0);
    }

    #[test]
    fn explodes_when_destroyed_modules_reach_75_percent() {
        let mut ship = Ship::new(2);
        ship.set_slot(0, 0, Slot::with_module(0, module_with_hp("m1", 0)))
            .expect("slot should be valid");
        ship.set_slot(1, 0, Slot::with_module(0, module_with_hp("m2", 0)))
            .expect("slot should be valid");
        ship.set_slot(0, 1, Slot::with_module(0, module_with_hp("m3", 0)))
            .expect("slot should be valid");
        ship.set_slot(1, 1, Slot::with_module(0, module_with_hp("m4", 10)))
            .expect("slot should be valid");

        assert_eq!(ship.destroyed_modules(), 3);
        assert_eq!(ship.total_modules(), 4);
        assert!(ship.is_exploded());
    }
}
