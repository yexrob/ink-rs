/// Border drawing characters. The named styles match the cli-boxes set that
/// Ink uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BorderStyle {
    /// `┌─┐│┘─└│`
    Single,
    /// `╔═╗║╝═╚║`
    Double,
    /// `╭─╮│╯─╰│`
    Round,
    /// `┏━┓┃┛━┗┃`
    Bold,
    /// `╓─╖║╜─╙║`
    SingleDouble,
    /// `╒═╕│╛═╘│`
    DoubleSingle,
    /// `+-+|+-+|`
    Classic,
    /// `↘↓↙←↖↑↗→`
    Arrow,
    Custom(BoxChars),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoxChars {
    pub top_left: char,
    pub top: char,
    pub top_right: char,
    pub right: char,
    pub bottom_right: char,
    pub bottom: char,
    pub bottom_left: char,
    pub left: char,
}

impl BorderStyle {
    pub fn chars(self) -> BoxChars {
        let [
            top_left,
            top,
            top_right,
            right,
            bottom_right,
            bottom,
            bottom_left,
            left,
        ] = match self {
            BorderStyle::Single => ['┌', '─', '┐', '│', '┘', '─', '└', '│'],
            BorderStyle::Double => ['╔', '═', '╗', '║', '╝', '═', '╚', '║'],
            BorderStyle::Round => ['╭', '─', '╮', '│', '╯', '─', '╰', '│'],
            BorderStyle::Bold => ['┏', '━', '┓', '┃', '┛', '━', '┗', '┃'],
            BorderStyle::SingleDouble => ['╓', '─', '╖', '║', '╜', '─', '╙', '║'],
            BorderStyle::DoubleSingle => ['╒', '═', '╕', '│', '╛', '═', '╘', '│'],
            BorderStyle::Classic => ['+', '-', '+', '|', '+', '-', '+', '|'],
            BorderStyle::Arrow => ['↘', '↓', '↙', '←', '↖', '↑', '↗', '→'],
            BorderStyle::Custom(chars) => return chars,
        };
        BoxChars {
            top_left,
            top,
            top_right,
            right,
            bottom_right,
            bottom,
            bottom_left,
            left,
        }
    }
}
