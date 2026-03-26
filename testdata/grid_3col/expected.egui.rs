fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            egui::Grid::new("grid")
                .num_columns(3)
                .show(ui, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(251, 180, 174))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-1").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(179, 205, 227))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-2").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(204, 235, 197))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-3").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    ui.end_row();
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(222, 203, 228))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-4").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(254, 217, 166))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-5").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(255, 255, 204))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(egui::RichText::new("cell-6").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                            });
                        });
                    ui.end_row();
                });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
