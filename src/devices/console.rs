use crate::devices::{
    cursor::Cursor,
    vga_buffer::{Writer, BUFFER_HEIGHT, BUFFER_WIDTH},
};
use crate::println;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Default)]
pub struct Console {
    cursor: Cursor,
    writer: Writer,
}

impl Console {
    pub fn print_char_and_move_cursor(&mut self, character: char) {
        match character {
            '\n' => {
                self.move_to_next_line();
            }
            _ => {
                let (row, col) = self.cursor.position;

                self.writer.write_char_at(row, col, character);

                let (new_row, new_col) = if col < BUFFER_WIDTH - 1 {
                    (row, col + 1)
                } else {
                    (row + 1, 0)
                };

                if new_row >= BUFFER_HEIGHT {
                    self.scroll_up();
                } else {
                    self.cursor.set_cursor_position(new_row, new_col);
                }
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.writer.clear();
        self.cursor.set_cursor_position(BUFFER_HEIGHT - 1, 0);
    }

    fn scroll_up(&mut self) {
        self.writer.new_line();
        self.cursor.set_cursor_position(BUFFER_HEIGHT - 1, 0);
    }

    fn move_to_next_line(&mut self) {
        let (row, _) = self.cursor.position;

        if row < BUFFER_HEIGHT - 1 {
            self.cursor.set_cursor_position(row + 1, 0);
        } else {
            self.scroll_up();
        }
    }

    pub fn backspace(&mut self) {
        let (row, col) = self.cursor.position;

        if col > 0 {
            self.cursor.set_cursor_position(row, col - 1);
            self.writer.write_char_at(row, col - 1, ' ');
        } else if row > 0 {
            self.cursor.set_cursor_position(row - 1, BUFFER_WIDTH - 1);
            self.writer.write_char_at(row - 1, BUFFER_WIDTH - 1, ' ');
        }
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::default());
}
