fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
            let layout = egui::Layout::left_to_right(egui::Align::Min)
                .with_main_justify(true) // approximate SpaceBetween;
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(251, 180, 174))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(80.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("A").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(179, 205, 227))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(40.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("B").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(204, 235, 197))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(100.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("C").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(222, 203, 228))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(60.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("D").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
