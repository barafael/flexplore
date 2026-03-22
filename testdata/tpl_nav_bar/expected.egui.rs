fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let layout = egui::Layout::left_to_right(egui::Align::Center)
                .with_main_justify(true); // approximate SpaceBetween
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(251, 180, 174))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(48.0, 48.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("logo").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(28, 28, 43))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                        let layout = egui::Layout::left_to_right(egui::Align::Center);
                        ui.with_layout(layout, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(179, 205, 227))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(80.0, 36.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("link-1").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(204, 235, 197))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(80.0, 36.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("link-2").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(222, 203, 228))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(80.0, 36.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("link-3").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                        });
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(28, 28, 43))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                        let layout = egui::Layout::left_to_right(egui::Align::Center);
                        ui.with_layout(layout, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(254, 217, 166))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(36.0, 36.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("btn-1").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(255, 255, 204))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(36.0, 36.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("btn-2").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                        });
                    });
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
