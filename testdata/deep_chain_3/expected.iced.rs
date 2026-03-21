fn view(&self) -> iced::Element<'_, Message> {
    row![
        // NOTE: flex-wrap: Wrap — call .wrap() on the Row for wrapping support
        row![
            // NOTE: flex-wrap: Wrap — call .wrap() on the Row for wrapping support
            row![
                // NOTE: flex-wrap: Wrap — call .wrap() on the Row for wrapping support
                container(text("leaf").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(50.0))
                    .padding(8.0)
                    .center(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                        ..Default::default()
                    }),
            ]
            .spacing(8.0)
            .align_y(Vertical::Top)
            .width(Length::Fill)
            .padding(12.0),
        ]
        .spacing(8.0)
        .align_y(Vertical::Top)
        .width(Length::Fill)
        .padding(12.0),
    ]
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
