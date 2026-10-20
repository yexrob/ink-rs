#![cfg(feature = "macros")]

use ink_rs::{
    Color, ColorLevel, Element, FlexDirection, Newline, RenderOptions, Scope, Spacer, Static, Text,
    Transform, View, component, render_to_string, rsx,
};

fn assert_rendered(actual: impl Into<Element>, expected: impl Into<Element>, text: &str) {
    let options = RenderOptions {
        color_level: ColorLevel::None,
        ..Default::default()
    };
    let actual = render_to_string(actual, options);
    assert_eq!(actual, render_to_string(expected, options));
    assert_eq!(actual, text);
}

#[test]
fn nesting_styles_and_captures() {
    let total = 3;
    let __ink_rsx_builder = "user binding";
    let actual = rsx! {
        ink_rs::View(flex_direction: FlexDirection::Column, margin_top: 1,) {
            View {
                View(width: 14) { Text(bold: true) { "Test Suites:" } }
                Text(color: Color::Green) { "{total} total" }
            }
            Text { "{{done}} " {__ink_rsx_builder} }
        }
    };
    let expected = {
        use ink_rs::Styled;
        View::new()
            .flex_direction(FlexDirection::Column)
            .margin_top(1)
            .child(
                View::new()
                    .child(
                        View::new()
                            .width(14)
                            .child(Text::new("Test Suites:").bold(true)),
                    )
                    .child(Text::new(format!("{total} total")).color(Color::Green)),
            )
            .child(Text::new("{done} ").child(__ink_rsx_builder))
    };
    assert_rendered(
        actual,
        expected,
        "\nTest Suites:  3 total\n{done} user binding",
    );
}

#[test]
fn positional_arguments_and_default_text() {
    let items = vec!["log"];
    let content = "bare";
    let actual = rsx! {
        View(flex_direction: FlexDirection::Column) {
            Static(items.clone(), |item, _| Text::new(*item).into())
            Text("a")
            Transform(|line, _| line.to_uppercase()) { "x" }
            Text { "a" "b" }
            Text(content, bold: true)
        }
    };
    let expected = {
        use ink_rs::Styled;
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(items, |item, _| Text::new(*item).into()))
            .child(Text::new("a"))
            .child(Transform::new(|line, _| line.to_uppercase()).child("x"))
            .child(Text::default().child("a").child("b"))
            .child(Text::new(content).bold(true))
    };
    assert_rendered(actual, expected, "log\na\nX\nab\nbare");
}

#[test]
fn branches_and_keyed_loops() {
    for (choice, label) in [(0, "zero"), (1, "one"), (2, "other")] {
        let actual = rsx! {
            View(flex_direction: FlexDirection::Column) {
                if choice == 0 {
                    Text { "zero" }
                    View { Text { "extra" } }
                } else if choice == 1 {
                    View { Text { "one" } }
                } else {
                    Text { "other" }
                }
                if choice == 0 { Text { "optional" } }
                Text {
                    if choice == 0 {
                        "zero"
                        Text { "+" }
                    } else if choice == 1 {
                        Text { "one" }
                    } else {
                        {String::from("other")}
                    }
                    if choice == 0 { "!" }
                }
                for (index, value) in ["a", "b"].iter().enumerate() {
                    Text(key: index, bold: true) { "{value}" }
                }
                if let Some(value) = Some("end") { Text { "{value}" } }
            }
        };
        let expected = {
            use ink_rs::Styled;
            let mut view = View::new().flex_direction(FlexDirection::Column);
            view = if choice == 0 {
                view.child(Text::new("zero"))
                    .child(View::new().child(Text::new("extra")))
            } else if choice == 1 {
                view.child(View::new().child(Text::new("one")))
            } else {
                view.child(Text::new("other"))
            };
            if choice == 0 {
                view = view.child(Text::new("optional"));
            }
            let mut text = Text::default();
            text = if choice == 0 {
                text.child("zero").child(Text::new("+"))
            } else if choice == 1 {
                text.child(Text::new("one"))
            } else {
                text.child(String::from("other"))
            };
            if choice == 0 {
                text = text.child("!");
            }
            view = view.child(text);
            for (index, value) in ["a", "b"].iter().enumerate() {
                view = view.child(Text::new(*value).bold(true).key(index));
            }
            if let Some(value) = Some("end") {
                view = view.child(Text::new(value));
            }
            view
        };
        let text = if choice == 0 {
            "zero\nextra\noptional\nzero+!\na\nb\nend".to_owned()
        } else {
            format!("{label}\n{label}\na\nb\nend")
        };
        assert_rendered(actual, expected, &text);
    }
}

#[test]
fn expression_children_and_leaf_elements() {
    fn greeting(_: &mut Scope, name: &&str) -> Element {
        Text::new(*name).into()
    }

    let optional = Some(Text::new("option"));
    let elements = vec![Element::from(Text::new("vec"))];
    let actual = rsx! {
        View(flex_direction: FlexDirection::Column) {
            {optional.clone()}
            {elements.clone()}
            {component(greeting, "component")}
            View(width: 5) { Text { "L" } Spacer Text { "R" } }
            Text { "first" Newline "last" }
        }
    };
    let expected = {
        use ink_rs::Styled;
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(optional)
            .child(elements)
            .child(component(greeting, "component"))
            .child(
                View::new()
                    .width(5)
                    .child(Text::new("L"))
                    .child(Spacer)
                    .child(Text::new("R")),
            )
            .child(Text::new("first").child(Newline::new()).child("last"))
    };
    assert_rendered(
        actual,
        expected,
        "option\nvec\ncomponent\nL   R\nfirst\nlast",
    );
}

#[test]
fn keyed_root_and_fragment() {
    let keyed: Element = rsx! { Text(key: "root", bold: true) { "a" } };
    let expression: Text = rsx! { {Text::new("b")} }.child("c");
    let actual: Element = rsx! {
        {keyed}
        {expression}
    };
    let expected = Element::fragment([
        Text::new("a").bold(true).key("root"),
        Element::from(Text::new("b").child("c")),
    ]);
    assert_rendered(actual, expected, "a\nbc");
}
