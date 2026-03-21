fn view(&self) -> iced::Element<'_, Message> {
    column![
        // NOTE: flex-wrap: Wrap — Iced Column does not support wrapping
        container(text("50vw x 20vh").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(50.0) /* 50vw — no viewport units in Iced */)
            .height(Length::Fixed(20.0) /* 20vh — no viewport units in Iced */)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.98, 0.71, 0.68).into()),
                ..Default::default()
            }),
        container(text("75vw x 30vh").size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))
            .width(Length::Fixed(75.0) /* 75vw — no viewport units in Iced */)
            .height(Length::Fixed(30.0) /* 30vh — no viewport units in Iced */)
            .padding(8.0)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.70, 0.80, 0.89).into()),
                ..Default::default()
            }),
    ]
    .spacing(8.0)
    .align_x(Horizontal::Left)
    .width(Length::Fill)
    .padding(12.0)
    .into()
}
