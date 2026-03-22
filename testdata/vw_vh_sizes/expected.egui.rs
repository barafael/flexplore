fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
            let layout = egui::Layout::top_down(egui::Align::Min)
                .with_main_wrap(true);
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(251, 180, 174))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(200.0 /* 50vw */, 60.0 /* 20vh */));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("50vw x 20vh").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(179, 205, 227))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(300.0 /* 75vw */, 90.0 /* 30vh */));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("75vw x 30vh").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
