fn view(&self) -> iced::Element<'_, Message> {
    column![
        container(text("header").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fill)
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            })
            // NOTE: flex-shrink: 0 — no Iced equivalent,
        row![
            container(text("sidebar-left").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(120.0))
                .height(Length::Fill)
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                    ..Default::default()
                })
                // NOTE: flex-shrink: 0 — no Iced equivalent,
            container(text("content").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                    ..Default::default()
                }),
            container(text("sidebar-right").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(120.0))
                .height(Length::Fill)
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                    ..Default::default()
                })
                // NOTE: flex-shrink: 0 — no Iced equivalent,
        ]
        .align_y(Vertical::Top)
        .height(Length::Fill)
        .width(Length::Fill),
        container(text("footer").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fill)
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(1.00, 0.85, 0.65).into()),
                ..Default::default()
            })
            // NOTE: flex-shrink: 0 — no Iced equivalent,
    ]
    .align_x(Horizontal::Left)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
