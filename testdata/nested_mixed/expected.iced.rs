fn view(&self) -> iced::Element<'_, Message> {
    row![
        container(text("A").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        column![
            // NOTE: flex-wrap: Wrap — Iced Column does not support wrapping
            container(text("X").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                    ..Default::default()
                }),
            container(text("Y").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                    ..Default::default()
                }),
        ]
        .spacing(8.0)
        .align_x(Horizontal::Left)
        .width(Length::Fixed(200.0))
        .padding(12.0),
        container(text("B").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                ..Default::default()
            }),
    ]
    .wrap()
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
