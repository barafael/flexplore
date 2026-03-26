fn view(&self) -> iced::Element<'_, Message> {
    // CSS Grid: 3 columns
    // grid-template-columns: 1.0fr 1.0fr 1.0fr
    // grid-template-rows: 60px 1.0fr 40px
    // Approximated with Row/Column — Iced has no CSS Grid support
    row![
        container(text("header").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("sidebar").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            }),
        container(text("main").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.80, 0.92, 0.77).into()),
                ..Default::default()
            }),
        container(text("footer").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.87, 0.80, 0.89).into()),
                ..Default::default()
            }),
    ]
    .wrap()
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
