fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let layout = egui::Layout::left_to_right(egui::Align::Min)
                .with_cross_justify(true);
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(28, 28, 43))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_width(120.0);
                        ui.set_min_height(ui.available_height());
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                        let layout = egui::Layout::top_down(egui::Align::Min)
                            .with_cross_justify(true);
                        ui.with_layout(layout, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(251, 180, 174))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(40.0, 44.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("nav-1").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(179, 205, 227))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(40.0, 44.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("nav-2").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(204, 235, 197))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(40.0, 44.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("nav-3").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                });
                        });
                        // NOTE: flex-shrink: 0 — no egui equivalent
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(222, 203, 228))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("content").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-grow: 1 — no egui equivalent; use ui.available_size();
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
