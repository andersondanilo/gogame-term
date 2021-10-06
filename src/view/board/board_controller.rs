use super::BoardTable;
use crate::core::helpers::get_column_number;
use crate::view::board::OptCoords;
use crate::view::events::{Event, EventHandler, EventSideEffect};
use termion::event::Key;

pub struct BoardController<'a> {
    board_table: BoardTable<'a>,
    pub next_move_input: String,
    char_range: [char; 19],
    number_range: [char; 10],
}

impl<'a> BoardController<'a> {
    pub fn new(board_table: BoardTable<'a>) -> Self {
        BoardController {
            board_table,
            next_move_input: "".to_string(),
            char_range: [
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
                'R', 'S', 'T',
            ],
            number_range: ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        }
    }

    pub fn update_highlight_coords(&mut self) {
        self.board_table
            .change_highlight_coord(parse_input_coords(self.next_move_input.clone()))
    }

    pub fn borrow_board_table(&mut self) -> &mut BoardTable<'a> {
        &mut self.board_table
    }
}

impl<'a> EventHandler<Key> for BoardController<'a> {
    fn on_event(&mut self, event: Event<Key>) -> EventSideEffect {
        match event {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Ctrl('c') => EventSideEffect::QuitApp,
                Key::Char(char) if self.char_range.contains(&char) => {
                    if self.next_move_input.is_empty() {
                        self.next_move_input.push(char);
                    }

                    self.update_highlight_coords();

                    EventSideEffect::None
                }
                Key::Char(char) if self.number_range.contains(&char) => {
                    if !self.next_move_input.is_empty() && self.next_move_input.len() < 3 {
                        self.next_move_input.push(char);
                    }

                    self.update_highlight_coords();

                    EventSideEffect::None
                }
                Key::Backspace => {
                    if !self.next_move_input.is_empty() {
                        self.next_move_input.pop();
                    }

                    self.update_highlight_coords();

                    EventSideEffect::None
                }
                Key::Char('\n') => {
                    self.next_move_input = "".to_string();

                    self.update_highlight_coords();

                    EventSideEffect::None
                }
                _ => EventSideEffect::None,
            },
            Event::Tick => EventSideEffect::None,
        }
    }
}

fn parse_input_coords(mut input: String) -> OptCoords {
    let col: Option<u8> = if !input.is_empty() {
        Some(get_column_number(input.remove(0)))
    } else {
        None
    };

    let row: Option<u8> = if !input.is_empty() {
        input.parse().ok()
    } else {
        None
    };

    OptCoords { col, row }
}
