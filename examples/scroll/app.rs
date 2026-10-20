// Ink examples/scroll.

use ink_rs::{
    BorderStyle, Element, FlexDirection, Overflow, Scope, Styled, Text, TextWrap, View, component,
};

pub fn app() -> Element {
    component(scroll, ())
}

fn scroll(cx: &mut Scope, _: &()) -> Element {
    let viewport_ref = cx.use_node_ref();
    let content_ref = cx.use_node_ref();
    let viewport = cx.use_box_metrics(viewport_ref);
    let content = cx.use_box_metrics(content_ref);
    let offset = cx.use_state(|| (0_u32, 0_u32));
    let max_top = content.height.saturating_sub(viewport.client_height);
    let max_left = content.width.saturating_sub(viewport.client_width);
    let app = cx.use_app();
    cx.use_input(true, move |input, key| {
        if input == "q" {
            app.exit();
        }
        offset.update(|(top, left)| {
            *top = (*top).min(max_top);
            *left = (*left).min(max_left);
            if key.up_arrow {
                *top = top.saturating_sub(1);
            }
            if key.down_arrow {
                *top = (*top + 1).min(max_top);
            }
            if key.left_arrow {
                *left = left.saturating_sub(2);
            }
            if key.right_arrow {
                *left = (*left + 2).min(max_left);
            }
        });
    });
    let (top, left) = offset.get();
    let (top, left) = (top.min(max_top), left.min(max_left));
    // Natural content height and an explicit width provide both scroll axes.
    let content_view = View::new()
        .node_ref(content_ref)
        .flex_direction(FlexDirection::Column)
        .flex_shrink(0.0)
        .width(140)
        .children((0..40).map(|i| {
            Text::new(format!(
                "Line {:02} {}",
                i + 1,
                if i % 5 == 0 {
                    format!(
                        "\u{25c0} marker {} end-of-line-{} \u{25b6}",
                        "\u{b7}".repeat(100),
                        i + 1
                    )
                } else {
                    String::new()
                }
            ))
            .wrap(TextWrap::TruncateEnd)
            .key(i)
        }));
    let status = format!(
        "scrollTop={top}/{max_top} scrollLeft={left}/{max_left} \
         client={}x{} content={}x{} (arrows to scroll, q to quit)",
        viewport.client_width, viewport.client_height, content.width, content.height
    );
    View::new()
        .flex_direction(FlexDirection::Column)
        .child(
            View::new()
                .node_ref(viewport_ref)
                .height(10)
                .overflow(Overflow::Hidden)
                .content_offset_y(top as i16)
                .content_offset_x(left as i16)
                .flex_direction(FlexDirection::Column)
                .border_style(BorderStyle::Round)
                .child(content_view),
        )
        .child(Text::new(status).dim_color(true))
        .into()
}
