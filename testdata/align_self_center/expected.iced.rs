fn view(&self) -> iced::Element<'_, Message> {
    row![
        container(text("top").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(100.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("centered").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            })
            // NOTE: align-self: Center — no Iced equivalent,
        container(text("top").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(100.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            }),
    ]
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
    .padding(12.0)
    .into()
}
