fn view(&self) -> iced::Element<'_, Message> {
    row![
        container(text("grow-1").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fill)
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("grow-2").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::FillPortion(2))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("fixed").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(100.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            })
            // NOTE: flex-shrink: 0 — no Iced equivalent,
    ]
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
