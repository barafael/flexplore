fn view(&self) -> iced::Element<'_, Message> {
    row![
        container(text("logo").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        Space::new(Length::Fill, Length::Shrink),
        row![
            container(text("link-1").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(36.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                    ..Default::default()
                }),
            container(text("link-2").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(36.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                    ..Default::default()
                }),
            container(text("link-3").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(36.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                    ..Default::default()
                }),
        ]
        .spacing(8.0),
        Space::new(Length::Fill, Length::Shrink),
        row![
            container(text("btn-1").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(36.0))
                .height(Length::Fixed(36.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(1.00, 0.85, 0.65).into()),
                    ..Default::default()
                }),
            container(text("btn-2").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                .width(Length::Fixed(36.0))
                .height(Length::Fixed(36.0))
                .padding(8.0)
                .center(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(1.00, 1.00, 0.80).into()),
                    ..Default::default()
                }),
        ]
        .spacing(8.0),
    ]
    .width(Length::Fill)
    .height(Length::Fixed(56.0))
    .padding(12.0)
    .into()
}
