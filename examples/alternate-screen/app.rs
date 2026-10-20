// Ink examples/alternate-screen: unicorn snake on the alternate screen.

use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::rc::Rc;
use std::time::Duration;

use ink_rs::{
    AnimationOptions, Color, Element, FlexDirection, JustifyContent, Scope, Styled, Text, View,
    component,
};

pub const BOARD_WIDTH: i16 = 20;

pub const BOARD_HEIGHT: i16 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(self) -> Point {
        let (x, y) = match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        };
        Point { x, y }
    }
}

#[derive(Clone)]
pub struct Game {
    pub snake: Vec<Point>,
    pub food: Point,
    pub score: usize,
    pub game_over: bool,
    pub won: bool,
    pub frame: usize,
}

pub enum Action {
    Tick(Direction),
    Restart,
}

fn random_position(exclude: &[Point]) -> Point {
    let mut random = RandomState::new().build_hasher().finish().max(1);
    random ^= random << 13;
    random ^= random >> 7;
    random ^= random << 17;
    // Pick among free cells instead of retrying occupied cells near a full board.
    let free: Vec<_> = (0..BOARD_HEIGHT)
        .flat_map(|y| (0..BOARD_WIDTH).map(move |x| Point { x, y }))
        .filter(|point| !exclude.contains(point))
        .collect();
    free[random as usize % free.len()]
}

impl Game {
    fn new() -> Self {
        let snake = vec![
            Point { x: 10, y: 7 },
            Point { x: 9, y: 7 },
            Point { x: 8, y: 7 },
        ];
        Self {
            food: random_position(&snake),
            snake,
            score: 0,
            game_over: false,
            won: false,
            frame: 0,
        }
    }
}

pub fn reduce(game: &mut Game, action: Action) {
    let direction = match action {
        Action::Restart => {
            *game = Game::new();
            return;
        }
        Action::Tick(direction) => direction,
    };
    if game.game_over {
        return;
    }
    let offset = direction.offset();
    let head = Point {
        x: game.snake[0].x + offset.x,
        y: game.snake[0].y + offset.y,
    };
    let eats = head == game.food;
    let collision_length = game.snake.len() - usize::from(!eats);
    if !(0..BOARD_WIDTH).contains(&head.x)
        || !(0..BOARD_HEIGHT).contains(&head.y)
        || game.snake[..collision_length].contains(&head)
    {
        game.game_over = true;
        game.won = false;
        return;
    }
    game.snake.insert(0, head);
    if eats {
        game.score += 1;
    } else {
        game.snake.pop();
    }
    game.frame += 1;
    if eats {
        game.won = game.snake.len() == (BOARD_WIDTH * BOARD_HEIGHT) as usize;
        game.game_over = game.won;
        if !game.won {
            game.food = random_position(&game.snake);
        }
    }
}

fn board(game: &Game) -> String {
    let border = "\u{2500}".repeat(BOARD_WIDTH as usize * 2);
    let mut rows = vec![format!("\u{250c}{border}\u{2510}")];
    for y in 0..BOARD_HEIGHT {
        let mut row = String::from("\u{2502}");
        for x in 0..BOARD_WIDTH {
            let point = Point { x, y };
            row.push_str(if point == game.snake[0] {
                "\u{1f984}"
            } else if game.snake.contains(&point) {
                "\u{2728}"
            } else if point == game.food {
                "\u{1f308}"
            } else {
                "  "
            });
        }
        row.push('\u{2502}');
        rows.push(row);
    }
    rows.push(format!("\u{2514}{border}\u{2518}"));
    rows.join("\n")
}

pub fn app() -> Element {
    component(snake, ())
}

fn snake(cx: &mut Scope, _: &()) -> Element {
    let app = cx.use_app();
    let columns = cx.use_window_size().columns;
    let game = cx.use_state(Game::new);
    let direction = cx.use_memo((), || Rc::new(Cell::new(Direction::Right)));
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(150),
        ..AnimationOptions::default()
    });
    let tick_direction = direction.clone();
    // One game step per delivered frame, not a replay of missed ticks.
    cx.use_effect(tick.frame, move || {
        if tick.frame > 0 {
            game.update(|game| reduce(game, Action::Tick(tick_direction.get())));
        }
    });
    cx.use_input(true, move |input, key| {
        if input == "q" {
            app.exit();
        }
        if input == "r" && game.get().game_over {
            direction.set(Direction::Right);
            game.update(|game| reduce(game, Action::Restart));
            return;
        }
        let game = game.get();
        if game.game_over {
            return;
        }
        let (head, neck) = (game.snake[0], game.snake[1]);
        // Validate against the neck, not another direction queued in this read.
        if key.up_arrow && head.y <= neck.y {
            direction.set(Direction::Up);
        } else if key.down_arrow && head.y >= neck.y {
            direction.set(Direction::Down);
        } else if key.left_arrow && head.x <= neck.x {
            direction.set(Direction::Left);
        } else if key.right_arrow && head.x >= neck.x {
            direction.set(Direction::Right);
        }
    });
    let game = game.get();
    let colors = [
        Color::Red,
        Color::Rgb(255, 127, 0),
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
    ];
    let footer = if game.game_over {
        Text::new(if game.won { "You Win! " } else { "Game Over! " })
            .bold(true)
            .color(Color::Red)
            .child(Text::new("r: restart | q: quit").dim_color(true))
    } else {
        Text::new("Arrow keys: move | Eat \u{1f308} to grow | q: quit").dim_color(true)
    };
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding_y(1)
        .child(
            View::new().justify_content(JustifyContent::Center).child(
                Text::new("\u{1f984} Unicorn Snake \u{1f984}")
                    .bold(true)
                    .color(colors[game.frame % colors.len()]),
            ),
        )
        .child(
            View::new()
                .justify_content(JustifyContent::Center)
                .margin_top(1)
                .child(
                    Text::new(format!("Score: {}", game.score))
                        .bold(true)
                        .color(Color::Yellow),
                ),
        )
        .child(
            View::new()
                .margin_left((columns.saturating_sub(42) / 2) as i16)
                .margin_top(1)
                .child(Text::new(board(&game))),
        )
        .child(
            View::new()
                .justify_content(JustifyContent::Center)
                .margin_top(1)
                .child(footer),
        )
        .into()
}
