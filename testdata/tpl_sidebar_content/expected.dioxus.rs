use dioxus::prelude::*;

fn FlexLayout() -> Element {
    rsx! {
        div {
            display: "flex",
            align_content: "flex-start",
            row_gap: "8.0px",
            flex_grow: "1",
            width: "100.0%",
            height: "100.0%",
            min_height: "0.0px",
            background: "rgba(28, 28, 43, 1)",
            box_sizing: "border-box",
            div {
                display: "flex",
                flex_direction: "column",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "8.0px",
                flex_shrink: "0",
                width: "120.0px",
                min_height: "0.0px",
                padding: "8.0px",
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
                    height: "44.0px",
                    padding: "8.0px",
                    background: "rgb(251, 180, 174)",
                    box_sizing: "border-box",
                    color: "rgba(13, 13, 26, 0.85)",
                    font_size: "26px",
                    "nav-1"
                }
                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    justify_content: "center",
                    align_items: "center",
                    align_content: "flex-start",
                    row_gap: "4.0px",
                    column_gap: "4.0px",
                    height: "44.0px",
                    padding: "8.0px",
                    background: "rgb(179, 205, 227)",
                    box_sizing: "border-box",
                    color: "rgba(13, 13, 26, 0.85)",
                    font_size: "26px",
                    "nav-2"
                }
                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    justify_content: "center",
                    align_items: "center",
                    align_content: "flex-start",
                    row_gap: "4.0px",
                    column_gap: "4.0px",
                    height: "44.0px",
                    padding: "8.0px",
                    background: "rgb(204, 235, 197)",
                    box_sizing: "border-box",
                    color: "rgba(13, 13, 26, 0.85)",
                    font_size: "26px",
                    "nav-3"
                }
            }
            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                align_items: "center",
                align_content: "flex-start",
                row_gap: "4.0px",
                column_gap: "4.0px",
                flex_grow: "1",
                padding: "8.0px",
                background: "rgb(222, 203, 228)",
                box_sizing: "border-box",
                color: "rgba(13, 13, 26, 0.85)",
                font_size: "26px",
                "content"
            }
        }
    }
}
