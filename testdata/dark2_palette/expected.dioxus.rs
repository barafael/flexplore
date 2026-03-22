use dioxus::prelude::*;

fn FlexLayout() -> Element {
    rsx! {
        div {
            display: "flex",
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
                width: "80.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(27, 158, 119)",
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
                width: "80.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(217, 95, 2)",
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
                width: "80.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(117, 112, 179)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "C"
            }
        }
    }
}
