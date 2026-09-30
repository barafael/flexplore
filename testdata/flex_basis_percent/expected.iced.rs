fn view(&self) -> iced::Element<'_, Message> {
    row![
        container(text("50%").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            })
            // NOTE: flex-basis: 50% — no Iced equivalent,
        container(text("25%-b").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            })
            // NOTE: flex-basis: 25% — no Iced equivalent,
        container(text("25%-c").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            })
            // NOTE: flex-basis: 25% — no Iced equivalent,
    ]
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
