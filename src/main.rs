use eframe::egui;

struct NovaApp;

impl eframe::App for NovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading("Nova Engine");
                ui.add_space(16.0);

                if ui.button("Play").clicked() {
                    // TODO: Start game flow when gameplay state is implemented.
                }

                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Nova Engine",
        options,
        Box::new(|_cc| Ok(Box::new(NovaApp))),
    )
}
