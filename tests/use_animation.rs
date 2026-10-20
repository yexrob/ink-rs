//! Ported from Ink test/use-animation.tsx (8.0.0, commit 26d2c3f).
//! Fake-clock advances replace delay/tickAsync; jitter bounds become exact values.
//! JS timer-call counts are not applicable; runtime tests cover the shared deadline.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use ink_rs::{
    Animation, AnimationOptions, Element, Options, Scope, State, Text, component, testing,
};

thread_local! {
    static VALUES: RefCell<Vec<Animation>> = const { RefCell::new(Vec::new()) };
    static SHOW: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

fn options(interval: u64, active: bool) -> AnimationOptions {
    AnimationOptions {
        interval: ms(interval),
        active,
    }
}

fn counter(cx: &mut Scope, options: &AnimationOptions) -> Element {
    let animation = cx.use_animation(*options);
    VALUES.with_borrow_mut(|values| values.push(animation));
    Text::new(animation.frame.to_string()).into()
}

fn latest() -> Animation {
    VALUES.with_borrow(|values| *values.last().unwrap())
}

fn triple() -> (u64, Duration, Duration) {
    let value = latest();
    (value.frame, value.time, value.delta)
}

fn dual(cx: &mut Scope, options: &[AnimationOptions; 2]) -> Element {
    let a = cx.use_animation(options[0]);
    let b = cx.use_animation(options[1]);
    Text::new(format!("{},{}", a.frame, b.frame)).into()
}

fn children(_: &mut Scope, props: &(bool, u64)) -> Element {
    Element::fragment([
        component(counter, options(50, true)),
        if props.0 {
            Text::new(",").into()
        } else {
            Element::fragment(Vec::<Element>::new())
        },
        if props.0 {
            component(counter, options(props.1, true))
        } else {
            Element::fragment(Vec::<Element>::new())
        },
    ])
}

fn optional_child(cx: &mut Scope, _: &()) -> Element {
    let show = cx.use_state(|| true);
    SHOW.set(Some(show));
    if show.get() {
        component(counter, options(50, true))
    } else {
        Text::new("gone").into()
    }
}

#[test]
fn frame_increments_over_time() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(20));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(80));
    assert_eq!(app.last_frame(), "2");
}

#[test]
fn does_not_update_when_is_active_is_false() {
    let mut app = testing::render(component(counter, options(50, false)));
    app.advance(ms(20));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "0");
}

#[test]
fn multiple_animations_with_the_same_interval_stay_in_sync() {
    let mut app = testing::render(component(dual, [options(50, true); 2]));
    app.advance(ms(20));
    assert_eq!(app.last_frame(), "0,0");
    app.advance(ms(100));
    assert_eq!(app.last_frame(), "2,2");
}

#[test]
fn multiple_animations_with_the_same_interval_share_one_timer() {
    let mut app = testing::render(component(dual, [options(50, true); 2]));
    app.advance(ms(100));
    assert_eq!(app.last_frame(), "2,2");
}

#[test]
fn animations_with_different_intervals_still_use_the_shared_timer() {
    let mut app = testing::render(component(dual, [options(50, true), options(80, true)]));
    app.advance(ms(170));
    assert_eq!(app.last_frame(), "3,2");
}

#[test]
fn shared_timer_is_cleaned_up_and_recreated_after_the_last_animation_unmounts() {
    testing::render(component(counter, options(50, true))).unmount();
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "2");
}

fn remaining_child(interval: u64) {
    let mut app = testing::render(component(children, (true, interval)));
    app.advance(ms(120));
    assert_eq!(
        app.last_frame().replace('\n', ""),
        if interval == 50 { "2,2" } else { "2,1" }
    );
    app.rerender(component(children, (false, interval)));
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "4");
}

#[test]
fn shared_timer_stays_alive_while_another_same_interval_animation_remains_mounted() {
    remaining_child(50);
}

#[test]
fn shared_timer_stays_alive_while_another_different_interval_animation_remains_mounted() {
    remaining_child(80);
}

#[test]
fn inactive_animations_do_not_start_the_shared_timer_until_one_becomes_active() {
    let mut app = testing::render(component(dual, [options(50, false); 2]));
    app.advance(ms(100));
    assert_eq!(app.last_frame(), "0,0");
    app.rerender(component(dual, [options(50, true), options(50, false)]));
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "2,0");
}

#[test]
fn cleans_up_on_unmount() {
    let mut app = testing::render(component(optional_child, ()));
    app.advance(ms(80));
    SHOW.get().unwrap().set(false);
    app.settle();
    let frames = app.frames().len();
    let renders = VALUES.with_borrow(Vec::len);
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "gone");
    assert_eq!(app.frames().len(), frames);
    assert_eq!(VALUES.with_borrow(Vec::len), renders);
}

#[test]
fn no_timer_leak_when_all_animations_are_inactive() {
    let mut app = testing::render(component(counter, options(50, false)));
    app.rerender(component(counter, options(50, true)));
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "2");
    app.rerender(component(counter, options(50, false)));
    let frames = app.frames().len();
    app.advance(ms(120));
    assert_eq!(app.frames().len(), frames);
    assert_eq!(app.last_frame(), "2");
}

#[test]
fn frame_catches_up_when_the_shared_timer_is_delayed() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(220));
    assert_eq!(app.last_frame(), "4");
}

#[test]
fn resets_frame_when_is_active_toggles_from_false_to_true() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(130));
    assert_eq!(app.last_frame(), "2");
    app.rerender(component(counter, options(50, false)));
    app.advance(ms(50));
    app.rerender(component(counter, options(50, true)));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(120));
    assert_eq!(app.last_frame(), "2");
}

#[test]
fn resets_frame_when_interval_changes() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(130));
    assert_eq!(app.last_frame(), "2");
    app.rerender(component(counter, options(200, true)));
    assert_eq!(app.last_frame(), "0");
}

#[test]
fn time_and_delta_reset_to_0_when_interval_changes() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(200));
    assert_eq!(triple(), (4, ms(200), ms(50)));
    app.rerender(component(counter, options(200, true)));
    assert_eq!(triple(), (0, ms(0), ms(0)));
    assert_eq!(app.last_frame(), "0");
}

#[test]
fn time_and_delta_reset_to_0_when_animation_is_resumed() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(200));
    assert_eq!(triple(), (4, ms(200), ms(50)));
    app.rerender(component(counter, options(50, false)));
    app.rerender(component(counter, options(50, true)));
    assert_eq!(triple(), (0, ms(0), ms(0)));
}

#[test]
fn different_intervals_advance_at_different_rates() {
    let mut app = testing::render(component(dual, [options(50, true), options(200, true)]));
    app.advance(ms(300));
    assert_eq!(app.last_frame(), "6,1");
}

#[test]
fn defaults_to_100ms_interval() {
    let mut app = testing::render(component(counter, AnimationOptions::default()));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(250));
    assert_eq!(app.last_frame(), "2");
}

#[test]
fn clamps_oversized_finite_interval_to_the_timer_maximum() {
    let mut app = testing::render(component(
        counter,
        AnimationOptions {
            interval: Duration::MAX,
            active: true,
        },
    ));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(1000));
    assert_eq!(app.last_frame(), "0");
    // Ink checks only the first second; the boundary distinguishes clamping
    // from retaining Duration::MAX, which would overflow Instant arithmetic.
    app.advance(ms(2_147_483_647 - 1000));
    assert_eq!(app.last_frame(), "1");
}

#[test]
fn clamps_zero_interval_to_1ms() {
    let mut app = testing::render(component(counter, options(0, true)));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(5));
    assert_eq!(app.last_frame(), "5");
}

fn live_cadence(max_fps: u32) {
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        component(counter, options(8, true)),
        Options {
            max_fps,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(latest().frame, 0);
    app.advance(ms(25));
    assert_eq!(latest().frame, 3);
}

#[test]
fn max_fps_does_not_speed_up_animation_state() {
    // Ink debug bypasses maxFps; exercise the live path with a 1 ms window.
    live_cadence(1000);
}

#[test]
fn low_max_fps_caps_animation_rerenders() {
    VALUES.with_borrow_mut(Vec::clear);
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        component(counter, options(10, true)),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(VALUES.with_borrow(Vec::len), 1);
    app.advance(ms(35));
    assert_eq!(VALUES.with_borrow(Vec::len), 1);
    app.advance(ms(1000));
    assert_eq!(VALUES.with_borrow(Vec::len), 2);
}

#[test]
fn max_fps_0_does_not_affect_animation_cadence() {
    live_cadence(0);
}

#[test]
fn delta_accounts_for_throttled_ticks() {
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        component(counter, options(20, true)),
        Options {
            max_fps: 5,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(latest().delta, ms(0));
    app.advance(ms(350));
    assert_eq!(latest().delta, ms(200));
}

#[test]
fn pausing_animation_stops_ticks_before_the_next_frame() {
    let mut app = testing::render(component(counter, options(8, true)));
    app.advance(ms(25));
    assert_eq!(app.last_frame(), "3");
    app.rerender(component(counter, options(8, false)));
    assert_eq!(app.last_frame(), "3");
    app.advance(ms(25));
    assert_eq!(app.last_frame(), "3");
}

#[test]
fn changing_interval_unsubscribes_stale_ticks_before_reset() {
    let mut app = testing::render(component(counter, options(8, true)));
    app.advance(ms(25));
    assert_eq!(app.last_frame(), "3");
    app.rerender(component(counter, options(200, true)));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(17));
    assert_eq!(app.last_frame(), "0");
}

#[test]
fn animations_advance_in_debug_mode_when_interactive_is_false() {
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        component(counter, options(8, true)),
        Options {
            interactive: Some(false),
            max_fps: 120,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(latest().frame, 0);
    app.advance(ms(25));
    app.unmount();
    // No runtime debug option: non-interactive output is written on unmount.
    assert_eq!(terminal.writes().concat(), "2\n");
}

fn delayed(cx: &mut Scope, mount: &bool) -> Element {
    let show = cx.use_state(|| false);
    SHOW.set(Some(show));
    Element::fragment([
        component(counter, options(20, true)),
        Text::new(",").into(),
        if *mount && !show.get() {
            Text::new("-").into()
        } else {
            component(counter, options(20, show.get()))
        },
    ])
}

fn delayed_start(mount: bool) {
    let mut app = testing::render(component(delayed, mount));
    app.advance(ms(20));
    SHOW.get().unwrap().set(true);
    app.settle();
    app.advance(ms(5));
    assert_eq!(app.last_frame().replace('\n', ""), "1,0");
    app.advance(ms(40));
    assert_eq!(app.last_frame().replace('\n', ""), "3,2");
}

#[test]
fn newly_mounted_animations_do_not_inherit_elapsed_time() {
    delayed_start(true);
}

#[test]
fn newly_activated_animations_do_not_inherit_elapsed_time() {
    delayed_start(false);
}

#[test]
fn rerendering_with_the_same_interval_does_not_reset_the_frame() {
    fn parent(cx: &mut Scope, revision: &u8) -> Element {
        // A changed unrelated prop forces a render despite component memoization.
        let _ = revision;
        counter(cx, &options(20, true))
    }
    let mut app = testing::render(component(parent, 0));
    app.advance(ms(50));
    assert_eq!(app.last_frame(), "2");
    app.rerender(component(parent, 1));
    assert_eq!(app.last_frame(), "2");
    app.advance(ms(10));
    assert_eq!(triple(), (3, ms(60), ms(20)));
}

#[test]
fn time_increases_with_each_tick() {
    let mut app = testing::render(component(counter, options(50, true)));
    assert_eq!(latest().time, ms(0));
    app.advance(ms(80));
    assert_eq!(latest().time, ms(50));
    app.advance(ms(80));
    assert_eq!(latest().time, ms(150));
}

#[test]
fn delta_approximates_interval_on_each_tick() {
    let mut app = testing::render(component(counter, options(50, true)));
    assert_eq!(latest().delta, ms(0));
    app.advance(ms(80));
    assert_eq!(latest().delta, ms(50));
    app.advance(ms(80));
    assert_eq!(latest().delta, ms(50));
}

#[test]
fn reset_resets_frame_time_and_delta_to_0() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(200));
    assert_eq!(triple(), (4, ms(200), ms(50)));
    latest().reset();
    app.advance(ms(1));
    assert_eq!(triple(), (0, ms(0), ms(0)));
    app.advance(ms(100));
    assert_eq!(triple(), (2, ms(100), ms(50)));
}

#[test]
fn reset_is_a_stable_function_reference() {
    let mut app = testing::render(component(counter, options(50, true)));
    let original = latest();
    app.advance(ms(200));
    assert_eq!(latest().frame, 4);
    original.reset();
    app.settle();
    assert_eq!(triple(), (0, ms(0), ms(0)));
    app.advance(ms(50));
    assert_eq!(triple(), (1, ms(50), ms(50)));
}

#[test]
fn reset_clears_paused_animation_values_without_resuming() {
    let mut app = testing::render(component(counter, options(50, true)));
    app.advance(ms(200));
    assert_eq!(triple(), (4, ms(200), ms(50)));
    app.rerender(component(counter, options(50, false)));
    latest().reset();
    app.advance(ms(1));
    assert_eq!(triple(), (0, ms(0), ms(0)));
    app.advance(ms(100));
    assert_eq!(triple(), (0, ms(0), ms(0)));
    app.rerender(component(counter, options(50, true)));
    assert_eq!(triple(), (0, ms(0), ms(0)));
    app.advance(ms(100));
    assert_eq!(triple(), (2, ms(100), ms(50)));
}

#[test]
fn unmount_before_first_tick_cleans_up_without_error() {
    let mut app = testing::render(component(optional_child, ()));
    assert_eq!(app.last_frame(), "0");
    SHOW.get().unwrap().set(false);
    app.settle();
    let writes = app.frames().len();
    let renders = VALUES.with_borrow(Vec::len);
    app.advance(ms(200));
    assert_eq!(app.frames().len(), writes);
    assert_eq!(VALUES.with_borrow(Vec::len), renders);
}

#[test]
fn frame_resets_to_0_on_each_resume_across_multiple_cycles() {
    let mut app = testing::render(component(counter, options(50, true)));
    for _ in 0..3 {
        app.advance(ms(120));
        assert_eq!(app.last_frame(), "2");
        app.rerender(component(counter, options(50, false)));
        app.rerender(component(counter, options(50, true)));
        assert_eq!(app.last_frame(), "0");
    }
}

#[test]
fn is_active_false_from_mount_never_starts_a_timer_or_advances_the_frame() {
    let mut app = testing::render(component(counter, options(50, false)));
    assert_eq!(app.last_frame(), "0");
    app.advance(ms(500));
    assert_eq!(app.last_frame(), "0");
    assert_eq!(app.frames().len(), 1);
}
