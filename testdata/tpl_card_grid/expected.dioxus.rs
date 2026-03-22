use dioxus::prelude::*;

fn FlexLayout() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_wrap: "wrap",
            align_items: "flex-start",
            align_content: "flex-start",
            row_gap: "16.0px",
            column_gap: "16.0px",
            flex_grow: "1",
            width: "100.0%",
            min_height: "0.0px",
            padding: "16.0px",
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
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(251, 180, 174)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-1"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(179, 205, 227)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-2"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(204, 235, 197)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-3"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(222, 203, 228)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-4"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(254, 217, 166)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-5"
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                width: "170.0px",
                height: "80.0px",
                padding: "8.0px",
                background: "rgb(255, 255, 204)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "card-6"
            }
        }
    }
}
