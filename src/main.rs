use eframe::egui;
use std::time::Instant;

const CIRCLE_RADIUS: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Menu,
    Sandbox,
}

struct NovaApp {
    screen: Screen,
    circle_pos: Option<egui::Pos2>,
    circle_velocity: egui::Vec2,
    last_update: Instant,
}

impl Default for NovaApp {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            circle_pos: None,
            circle_velocity: egui::vec2(240.0, 180.0),
            last_update: Instant::now(),
        }
    }
}

impl eframe::App for NovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

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
                            self.circle_pos = None;
                            self.last_update = Instant::now();
                        }

                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            }
            Screen::Sandbox => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let bounds = ui.max_rect();

                    let mut circle_pos = self.circle_pos.unwrap_or(bounds.center());
                    circle_pos += self.circle_velocity * dt;

                    if circle_pos.x - CIRCLE_RADIUS <= bounds.left() && self.circle_velocity.x < 0.0 {
                        circle_pos.x = bounds.left() + CIRCLE_RADIUS;
                        self.circle_velocity.x = -self.circle_velocity.x;
                    } else if circle_pos.x + CIRCLE_RADIUS >= bounds.right()
                        && self.circle_velocity.x > 0.0
                    {
                        circle_pos.x = bounds.right() - CIRCLE_RADIUS;
                        self.circle_velocity.x = -self.circle_velocity.x;
                    }

                    if circle_pos.y - CIRCLE_RADIUS <= bounds.top() && self.circle_velocity.y < 0.0 {
                        circle_pos.y = bounds.top() + CIRCLE_RADIUS;
                        self.circle_velocity.y = -self.circle_velocity.y;
                    } else if circle_pos.y + CIRCLE_RADIUS >= bounds.bottom()
                        && self.circle_velocity.y > 0.0
                    {
                        circle_pos.y = bounds.bottom() - CIRCLE_RADIUS;
                        self.circle_velocity.y = -self.circle_velocity.y;
                    }

                    self.circle_pos = Some(circle_pos);
                    ui.painter()
                        .circle_filled(circle_pos, CIRCLE_RADIUS, egui::Color32::from_rgb(80, 180, 255));

                    if ui.button("Back").clicked() {
                        self.screen = Screen::Menu;
                    }
                });

                ctx.request_repaint();
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
