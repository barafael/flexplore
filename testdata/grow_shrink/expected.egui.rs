fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
            let layout = egui::Layout::left_to_right(egui::Align::Min);
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(251, 180, 174))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(40.0, 80.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("grow-1").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-grow: 1 — no egui equivalent; use ui.available_size();
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(179, 205, 227))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(40.0, 80.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("grow-2").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-grow: 2 — no egui equivalent; use ui.available_size();
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(204, 235, 197))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(100.0, 80.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("fixed").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-shrink: 0 — no egui equivalent;
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
