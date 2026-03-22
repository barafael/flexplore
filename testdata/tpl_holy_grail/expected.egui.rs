fn build_ui(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 28, 43))
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let layout = egui::Layout::top_down(egui::Align::Min)
                .with_cross_justify(true);
            ui.with_layout(layout, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(251, 180, 174))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(40.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("header").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-shrink: 0 — no egui equivalent;
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(28, 28, 43))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        let layout = egui::Layout::left_to_right(egui::Align::Min)
                            .with_cross_justify(true);
                        ui.with_layout(layout, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(179, 205, 227))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(120.0, 40.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("sidebar-left").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                })
                                // NOTE: flex-shrink: 0 — no egui equivalent;
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(204, 235, 197))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("content").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                })
                                // NOTE: flex-grow: 1 — no egui equivalent; use ui.available_size();
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(222, 203, 228))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(120.0, 40.0));
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("sidebar-right").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                                    });
                                })
                                // NOTE: flex-shrink: 0 — no egui equivalent;
                        });
                        // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
                    });
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(254, 217, 166))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(40.0, 60.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("footer").size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));
                        });
                    })
                    // NOTE: flex-shrink: 0 — no egui equivalent;
            });
            // NOTE: flex-grow: 1 — use ui.available_size() to fill parent
        })
}
