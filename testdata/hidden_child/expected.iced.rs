fn view(&self) -> iced::Element<'_, Message> {
    row![
        // NOTE: flex-wrap: Wrap — call .wrap() on the Row for wrapping support
        container(text("visible").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("hidden").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            })
            // NOTE: hidden — Iced has no visibility modifier; conditionally include this widget,
    ]
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
