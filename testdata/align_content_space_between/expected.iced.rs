fn view(&self) -> iced::Element<'_, Message> {
    row![
        // NOTE: align-content: SpaceBetween — no Iced equivalent
        container(text("A").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("B").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("C").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            }),
        container(text("D").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("E").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(1.00, 0.85, 0.65).into()),
                ..Default::default()
            }),
        container(text("F").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(60.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(1.00, 1.00, 0.80).into()),
                ..Default::default()
            }),
    ]
    .wrap()
    .spacing(8.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .height(Length::Fixed(280.0))
    .padding(12.0)
    .into()
}
