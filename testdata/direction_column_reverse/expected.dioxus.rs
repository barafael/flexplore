use dioxus::prelude::*;

fn FlexLayout() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column-reverse",
            flex_wrap: "wrap",
            align_items: "flex-start",
            align_content: "flex-start",
            row_gap: "8.0px",
            column_gap: "8.0px",
            flex_grow: "1",
            width: "100.0%",
            min_height: "0.0px",
            padding: "12.0px",
            background: "rgba(28, 28, 43, 1)",
            box_sizing: "border-box",
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "200.0px",
                height: "60.0px",
                padding: "8.0px",
                background: "rgb(251, 180, 174)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "A"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "120.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(179, 205, 227)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "B"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "60.0px",
                height: "40.0px",
                padding: "8.0px",
                background: "rgb(204, 235, 197)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "C"
            }
        }
    }
}
