fn view(&self) -> iced::Element<'_, Message> {
    row![
        column![
            container(text("nav-1").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fill)
                .height(Length::Fixed(44.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                    ..Default::default()
                }),
            container(text("nav-2").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fill)
                .height(Length::Fixed(44.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                    ..Default::default()
                }),
            container(text("nav-3").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fill)
                .height(Length::Fixed(44.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                    ..Default::default()
                }),
        ]
        .spacing(4.0)
        .align_x(Horizontal::Left)
        .width(Length::Fixed(120.0))
        .height(Length::Fill)
        .padding(8.0)
        // NOTE: flex-shrink: 0 — no Iced equivalent,
        container(text("content").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                ..Default::default()
            }),
    ]
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
