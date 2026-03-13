mod data;
mod domain;

use crate::data::module_catalog::ModuleCatalog;
use crate::domain::module::ModuleKind;
use crate::domain::ship::Ship;
use eframe::egui;
use rand::{rngs::ThreadRng, RngExt};
use std::time::Instant;

const SHIP_BG_SIZE: f32 = 260.0;
const MODULE_INSET: f32 = 24.0;
const MODULE_GAP: f32 = 8.0;
const SHOT_FADE_SECONDS: f32 = 1.25;
const HIT_CHANCE: f32 = 0.6;
const SHOT_DAMAGE: u32 = 10;
const MISS_OFFSET_MIN: f32 = 12.0;
const MISS_OFFSET_MAX: f32 = 52.0;

struct ShotLine {
    start: egui::Pos2,
    end: egui::Pos2,
    fired_at: Instant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Menu,
    Sandbox,
}

struct NovaApp {
    screen: Screen,
    shot_lines: Vec<ShotLine>,
    module_catalog: Option<ModuleCatalog>,
    left_ship_base: Option<Ship>,
    right_ship_base: Option<Ship>,
    left_ship: Option<Ship>,
    right_ship: Option<Ship>,
}

impl NovaApp {
    fn new(module_catalog: Option<ModuleCatalog>, left_ship: Option<Ship>, right_ship: Option<Ship>) -> Self {
        Self {
            screen: Screen::Menu,
            shot_lines: Vec::new(),
            module_catalog,
            left_ship_base: left_ship.clone(),
            right_ship_base: right_ship.clone(),
            left_ship,
            right_ship,
        }
    }

    fn slot_rect(ship_rect: egui::Rect, ship_size: usize, slot_index: usize) -> egui::Rect {
        let x = slot_index % ship_size;
        let y = slot_index / ship_size;
        let inner = ship_rect.shrink(MODULE_INSET);
        let ship_size_f = ship_size as f32;
        let cell_w = (inner.width() - ((ship_size_f - 1.0) * MODULE_GAP)) / ship_size_f;
        let cell_h = (inner.height() - ((ship_size_f - 1.0) * MODULE_GAP)) / ship_size_f;
        let min = egui::pos2(
            inner.left() + (x as f32 * (cell_w + MODULE_GAP)),
            inner.top() + (y as f32 * (cell_h + MODULE_GAP)),
        );
        egui::Rect::from_min_size(min, egui::vec2(cell_w, cell_h))
    }

    fn random_point_in_rect(rect: egui::Rect, rng: &mut ThreadRng) -> egui::Pos2 {
        egui::pos2(
            rng.random_range(rect.left()..=rect.right()),
            rng.random_range(rect.top()..=rect.bottom()),
        )
    }

    fn random_point_near_rect_outside(rect: egui::Rect, rng: &mut ThreadRng) -> egui::Pos2 {
        let offset = rng.random_range(MISS_OFFSET_MIN..=MISS_OFFSET_MAX);
        match rng.random_range(0..4) {
            0 => egui::pos2(
                rect.left() - offset,
                rng.random_range((rect.top() - offset)..=(rect.bottom() + offset)),
            ),
            1 => egui::pos2(
                rect.right() + offset,
                rng.random_range((rect.top() - offset)..=(rect.bottom() + offset)),
            ),
            2 => egui::pos2(
                rng.random_range((rect.left() - offset)..=(rect.right() + offset)),
                rect.top() - offset,
            ),
            _ => egui::pos2(
                rng.random_range((rect.left() - offset)..=(rect.right() + offset)),
                rect.bottom() + offset,
            ),
        }
    }
}

impl eframe::App for NovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Menu => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(120.0);
                        ui.heading("Nova Engine");
                        ui.add_space(16.0);

                        if ui.button("Play").clicked() {
                            // TODO: Start game flow when gameplay state is implemented.
                        }

                        if ui.button("Sandbox").clicked() {
                            self.screen = Screen::Sandbox;
                            self.shot_lines.clear();
                            self.left_ship = self.left_ship_base.clone();
                            self.right_ship = self.right_ship_base.clone();
                        }

                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            }
            Screen::Sandbox => {
                let now = Instant::now();
                self.shot_lines.retain(|line| {
                    (now - line.fired_at).as_secs_f32() < SHOT_FADE_SECONDS
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut fire_clicked = false;
                    ui.horizontal(|ui| {
                        fire_clicked = ui.button("Fire").clicked();

                        if ui.button("Back").clicked() {
                            self.screen = Screen::Menu;
                        }
                    });

                    ui.add_space(8.0);
                    let bounds = ui.available_rect_before_wrap();

                    let (left_ship, right_ship) = if let (Some(left_ship), Some(right_ship)) =
                        (&self.left_ship, &self.right_ship)
                    {
                        (left_ship, right_ship)
                    } else {
                        ui.label("No ship loaded for sandbox.");
                        return;
                    };

                    let center_y = bounds.center().y;
                    let left_ship_rect = egui::Rect::from_center_size(
                        egui::pos2(bounds.center().x - (SHIP_BG_SIZE * 0.8), center_y),
                        egui::vec2(SHIP_BG_SIZE, SHIP_BG_SIZE),
                    );
                    let right_ship_rect = egui::Rect::from_center_size(
                        egui::pos2(bounds.center().x + (SHIP_BG_SIZE * 0.8), center_y),
                        egui::vec2(SHIP_BG_SIZE, SHIP_BG_SIZE),
                    );

                    ui.painter().rect_filled(
                        left_ship_rect,
                        4.0,
                        egui::Color32::from_rgb(45, 55, 72),
                    );
                    ui.painter().rect_stroke(
                        left_ship_rect,
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(95, 105, 130)),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().rect_filled(
                        right_ship_rect,
                        4.0,
                        egui::Color32::from_rgb(45, 55, 72),
                    );
                    ui.painter().rect_stroke(
                        right_ship_rect,
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(95, 105, 130)),
                        egui::StrokeKind::Inside,
                    );

                    let mut left_gun_origins = Vec::new();
                    let mut left_slot_rects = Vec::with_capacity(left_ship.slots.len());
                    for (index, slot) in left_ship.slots.iter().enumerate() {
                        let cell_rect = Self::slot_rect(left_ship_rect, left_ship.size, index);
                        left_slot_rects.push(cell_rect);
                        let mut fill = egui::Color32::from_rgb(70, 80, 95);
                        if let Some(module) = &slot.module {
                            let kind = self
                                .module_catalog
                                .as_ref()
                                .and_then(|catalog| catalog.get(&module.archetype_id))
                                .map(|archetype| &archetype.kind);

                            fill = match kind {
                                Some(ModuleKind::Gun) => egui::Color32::from_rgb(215, 130, 80),
                                Some(ModuleKind::ShieldGenerator) => egui::Color32::from_rgb(80, 155, 215),
                                Some(ModuleKind::MissileLauncher) => egui::Color32::from_rgb(195, 95, 95),
                                Some(ModuleKind::Sensor) => egui::Color32::from_rgb(95, 185, 150),
                                Some(ModuleKind::Utility) => egui::Color32::from_rgb(130, 120, 210),
                                None => egui::Color32::from_rgb(120, 120, 120),
                            };

                            if module.is_destroyed() {
                                fill = egui::Color32::from_rgb(55, 55, 55);
                            } else if matches!(kind, Some(ModuleKind::Gun)) {
                                left_gun_origins.push(cell_rect.center());
                            }
                        }

                        ui.painter().rect_filled(cell_rect, 2.0, fill);
                        ui.painter().rect_stroke(
                            cell_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 25, 25)),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let mut right_gun_origins = Vec::new();
                    let mut right_slot_rects = Vec::with_capacity(right_ship.slots.len());
                    for (index, slot) in right_ship.slots.iter().enumerate() {
                        let cell_rect = Self::slot_rect(right_ship_rect, right_ship.size, index);
                        right_slot_rects.push(cell_rect);
                        let mut fill = egui::Color32::from_rgb(70, 80, 95);
                        if let Some(module) = &slot.module {
                            let kind = self
                                .module_catalog
                                .as_ref()
                                .and_then(|catalog| catalog.get(&module.archetype_id))
                                .map(|archetype| &archetype.kind);

                            fill = match kind {
                                Some(ModuleKind::Gun) => egui::Color32::from_rgb(215, 130, 80),
                                Some(ModuleKind::ShieldGenerator) => egui::Color32::from_rgb(80, 155, 215),
                                Some(ModuleKind::MissileLauncher) => egui::Color32::from_rgb(195, 95, 95),
                                Some(ModuleKind::Sensor) => egui::Color32::from_rgb(95, 185, 150),
                                Some(ModuleKind::Utility) => egui::Color32::from_rgb(130, 120, 210),
                                None => egui::Color32::from_rgb(120, 120, 120),
                            };

                            if module.is_destroyed() {
                                fill = egui::Color32::from_rgb(55, 55, 55);
                            } else if matches!(kind, Some(ModuleKind::Gun)) {
                                right_gun_origins.push(cell_rect.center());
                            }
                        }

                        ui.painter().rect_filled(cell_rect, 2.0, fill);
                        ui.painter().rect_stroke(
                            cell_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 25, 25)),
                            egui::StrokeKind::Inside,
                        );
                    }

                    if fire_clicked {
                        let fired_at = Instant::now();
                        let mut rng = rand::rng();

                        if let (Some(left_ship), Some(right_ship)) =
                            (&mut self.left_ship, &mut self.right_ship)
                        {
                            for start in left_gun_origins {
                                if right_slot_rects.is_empty() {
                                    continue;
                                }

                                let target_idx = rng.random_range(0..right_slot_rects.len());
                                let target_rect = right_slot_rects[target_idx];
                                let hit = rng.random_bool(HIT_CHANCE as f64);
                                let end = if hit {
                                    Self::random_point_in_rect(target_rect, &mut rng)
                                } else {
                                    Self::random_point_near_rect_outside(target_rect, &mut rng)
                                };

                                self.shot_lines.push(ShotLine {
                                    start,
                                    end,
                                    fired_at,
                                });

                                if hit {
                                    let _ = right_ship.apply_hit(target_idx, SHOT_DAMAGE);
                                }
                            }

                            for start in right_gun_origins {
                                if left_slot_rects.is_empty() {
                                    continue;
                                }

                                let target_idx = rng.random_range(0..left_slot_rects.len());
                                let target_rect = left_slot_rects[target_idx];
                                let hit = rng.random_bool(HIT_CHANCE as f64);
                                let end = if hit {
                                    Self::random_point_in_rect(target_rect, &mut rng)
                                } else {
                                    Self::random_point_near_rect_outside(target_rect, &mut rng)
                                };

                                self.shot_lines.push(ShotLine {
                                    start,
                                    end,
                                    fired_at,
                                });

                                if hit {
                                    let _ = left_ship.apply_hit(target_idx, SHOT_DAMAGE);
                                }
                            }
                        }
                    }

                    for line in &self.shot_lines {
                        let elapsed = (now - line.fired_at).as_secs_f32();
                        let alpha = ((1.0 - (elapsed / SHOT_FADE_SECONDS)).clamp(0.0, 1.0) * 255.0) as u8;
                        ui.painter().line_segment(
                            [line.start, line.end],
                            egui::Stroke::new(
                                2.0,
                                egui::Color32::from_rgba_unmultiplied(255, 120, 120, alpha),
                            ),
                        );
                    }
                });

                if !self.shot_lines.is_empty() {
                    ctx.request_repaint();
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let modules = match data::module_catalog::ModuleCatalog::from_path("assets/modules.ron") {
        Ok(modules) => Some(modules),
        Err(err) => {
            eprintln!("Failed to load module catalog: {err}");
            None
        }
    };
    let ships = match data::ship_catalog::ShipCatalog::from_path("assets/ships.ron") {
        Ok(ships) => Some(ships),
        Err(err) => {
            eprintln!("Failed to load ship catalog: {err}");
            None
        }
    };
    let loadouts = match data::loadout_catalog::LoadoutCatalog::from_path("assets/loadouts.ron") {
        Ok(loadouts) => Some(loadouts),
        Err(err) => {
            eprintln!("Failed to load loadout catalog: {err}");
            None
        }
    };

    let sandbox_ship = if let (Some(modules), Some(ships), Some(loadouts)) = (&modules, &ships, &loadouts) {
        match loadouts.instantiate_ship("corvette_single_gun", ships, modules) {
            Ok(ship) => Some(ship),
            Err(err) => {
                eprintln!("Failed to instantiate default loadout: {err}");
                None
            }
        }
    } else {
        None
    };

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Nova Engine",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(NovaApp::new(
                modules,
                sandbox_ship.clone(),
                sandbox_ship,
            )))
        }),
    )
}
