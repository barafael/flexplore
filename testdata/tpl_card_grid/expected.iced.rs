fn view(&self) -> iced::Element<'_, Message> {
    row![
        // NOTE: flex-wrap: Wrap — call .wrap() on the Row for wrapping support
        container(text("card-1").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("card-2").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("card-3").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            }),
        container(text("card-4").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("card-5").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(1.00, 0.85, 0.65).into()),
                ..Default::default()
            }),
        container(text("card-6").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(170.0))
            .height(Length::Fixed(80.0))
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(1.00, 1.00, 0.80).into()),
                ..Default::default()
            }),
    ]
    .spacing(16.0)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .padding(16.0)
    .into()
}
