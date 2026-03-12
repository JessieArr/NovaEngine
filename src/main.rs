use eframe::egui;
use rand::{Rng, RngExt};
use std::time::Instant;

const SQUARE_SIZE: f32 = 80.0;
const SIDE_MARGIN: f32 = 48.0;
const SHOT_FADE_SECONDS: f32 = 1.25;
const HIT_CHANCE: f32 = 0.6;
const MISS_OFFSET_MIN: f32 = 12.0;
const MISS_OFFSET_MAX: f32 = 52.0;
const MAX_HEALTH: u8 = 100;
const SHOT_DAMAGE: u8 = 10;
const HEALTH_BAR_HEIGHT: f32 = 8.0;
const HEALTH_BAR_GAP: f32 = 10.0;
const SQUARE_FADE_SECONDS: f32 = 1.0;

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
    left_health: u8,
    right_health: u8,
    left_destroyed_at: Option<Instant>,
    right_destroyed_at: Option<Instant>,
}

impl Default for NovaApp {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            shot_lines: Vec::new(),
            left_health: MAX_HEALTH,
            right_health: MAX_HEALTH,
            left_destroyed_at: None,
            right_destroyed_at: None,
        }
    }
}

fn random_point_in_rect(rect: egui::Rect, rng: &mut impl Rng) -> egui::Pos2 {
    egui::pos2(
        rng.random_range(rect.left()..=rect.right()),
        rng.random_range(rect.top()..=rect.bottom()),
    )
}

fn random_point_near_rect_outside(rect: egui::Rect, rng: &mut impl Rng) -> egui::Pos2 {
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

fn fade_alpha(destroyed_at: Option<Instant>, now: Instant) -> u8 {
    if let Some(destroyed_at) = destroyed_at {
        let elapsed = (now - destroyed_at).as_secs_f32();
        let opacity = (1.0 - (elapsed / SQUARE_FADE_SECONDS)).clamp(0.0, 1.0);
        (opacity * 255.0) as u8
    } else {
        255
    }
}

fn is_still_fading(destroyed_at: Option<Instant>, now: Instant) -> bool {
    destroyed_at
        .map(|destroyed_at| (now - destroyed_at).as_secs_f32() < SQUARE_FADE_SECONDS)
        .unwrap_or(false)
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
                            self.left_health = MAX_HEALTH;
                            self.right_health = MAX_HEALTH;
                            self.left_destroyed_at = None;
                            self.right_destroyed_at = None;
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
                    let square_size = egui::vec2(SQUARE_SIZE, SQUARE_SIZE);

                    let left_center = egui::pos2(
                        bounds.left() + SIDE_MARGIN + (SQUARE_SIZE * 0.5),
                        bounds.center().y,
                    );
                    let right_center = egui::pos2(
                        bounds.right() - SIDE_MARGIN - (SQUARE_SIZE * 0.5),
                        bounds.center().y,
                    );

                    let left_square = egui::Rect::from_center_size(left_center, square_size);
                    let right_square = egui::Rect::from_center_size(right_center, square_size);

                    if fire_clicked {
                        let mut rng = rand::rng();
                        let fired_at = Instant::now();

                        let left_hit = rng.random_bool(HIT_CHANCE as f64);
                        let left_target = if left_hit {
                            random_point_in_rect(right_square, &mut rng)
                        } else {
                            random_point_near_rect_outside(right_square, &mut rng)
                        };

                        let right_hit = rng.random_bool(HIT_CHANCE as f64);
                        let right_target = if right_hit {
                            random_point_in_rect(left_square, &mut rng)
                        } else {
                            random_point_near_rect_outside(left_square, &mut rng)
                        };

                        if left_hit {
                            self.right_health = self.right_health.saturating_sub(SHOT_DAMAGE);
                            if self.right_health == 0 && self.right_destroyed_at.is_none() {
                                self.right_destroyed_at = Some(fired_at);
                            }
                        }
                        if right_hit {
                            self.left_health = self.left_health.saturating_sub(SHOT_DAMAGE);
                            if self.left_health == 0 && self.left_destroyed_at.is_none() {
                                self.left_destroyed_at = Some(fired_at);
                            }
                        }

                        self.shot_lines.push(ShotLine {
                            start: left_center,
                            end: left_target,
                            fired_at,
                        });
                        self.shot_lines.push(ShotLine {
                            start: right_center,
                            end: right_target,
                            fired_at,
                        });
                    }

                    let left_alpha = fade_alpha(self.left_destroyed_at, now);
                    let right_alpha = fade_alpha(self.right_destroyed_at, now);

                    ui.painter().rect_filled(
                        left_square,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(80, 180, 255, left_alpha),
                    );
                    ui.painter().rect_filled(
                        right_square,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(80, 255, 160, right_alpha),
                    );

                    let left_bar = egui::Rect::from_min_size(
                        egui::pos2(
                            left_square.left(),
                            left_square.top() - HEALTH_BAR_GAP - HEALTH_BAR_HEIGHT,
                        ),
                        egui::vec2(SQUARE_SIZE, HEALTH_BAR_HEIGHT),
                    );
                    let right_bar = egui::Rect::from_min_size(
                        egui::pos2(
                            right_square.left(),
                            right_square.top() - HEALTH_BAR_GAP - HEALTH_BAR_HEIGHT,
                        ),
                        egui::vec2(SQUARE_SIZE, HEALTH_BAR_HEIGHT),
                    );

                    ui.painter()
                        .rect_filled(
                            left_bar,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(185, 35, 35, left_alpha),
                        );
                    ui.painter()
                        .rect_filled(
                            right_bar,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(185, 35, 35, right_alpha),
                        );

                    let left_green_width = SQUARE_SIZE * (self.left_health as f32 / MAX_HEALTH as f32);
                    let right_green_width = SQUARE_SIZE * (self.right_health as f32 / MAX_HEALTH as f32);

                    let left_green_bar = egui::Rect::from_min_size(
                        left_bar.left_top(),
                        egui::vec2(left_green_width, HEALTH_BAR_HEIGHT),
                    );
                    let right_green_bar = egui::Rect::from_min_size(
                        right_bar.left_top(),
                        egui::vec2(right_green_width, HEALTH_BAR_HEIGHT),
                    );

                    ui.painter()
                        .rect_filled(
                            left_green_bar,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(40, 210, 80, left_alpha),
                        );
                    ui.painter()
                        .rect_filled(
                            right_green_bar,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(40, 210, 80, right_alpha),
                        );

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

                let left_fading = is_still_fading(self.left_destroyed_at, now);
                let right_fading = is_still_fading(self.right_destroyed_at, now);
                if !self.shot_lines.is_empty() || left_fading || right_fading {
                    ctx.request_repaint();
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Nova Engine",
        options,
        Box::new(|_cc| Ok(Box::new(NovaApp::default()))),
    )
}
