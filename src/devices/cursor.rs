use x86_64::instructions::port::Port;

use crate::devices::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, WRITER};

static START_POSITION: (usize, usize) = (BUFFER_HEIGHT - 1, 0);

pub struct Cursor {
    pub position: (usize, usize),
    command_port: Port<u8>,
    data_port: Port<u8>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: START_POSITION,
            command_port: Port::new(0x3D4),
            data_port: Port::new(0x3D5),
        }
    }
}

impl Cursor {
    pub fn handle_ansi_escape(&mut self, sequence: &str) {
        match sequence {
            "[2J" => self.clear_screen(),
            "[?25h" => self.show_cursor(),
            "[?25l" => self.hide_cursor(),
            _ => {}
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                Self::write_char_at(row, col, ' ');
            }
        }
        self.set_cursor_position(START_POSITION.0, START_POSITION.1);
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            panic!("Cursor position out of bounds!");
        }

        let position: u16 = (row * BUFFER_WIDTH + col) as u16;

        unsafe {
            self.command_port.write(0x0F);
            self.data_port.write((position & 0xFF) as u8);

            self.command_port.write(0x0E);
            self.data_port.write((position >> 8) as u8);
        }

        self.position = (row, col);
    }

    pub fn show_cursor(&mut self) {
        unsafe {
            self.command_port.write(0x0A);
            let cursor_start: u8 = self.data_port.read();
            self.data_port.write(cursor_start & 0xDF);
        }
    }

    pub fn hide_cursor(&mut self) {
        unsafe {
            self.command_port.write(0x0A);
            let cursor_start: u8 = self.data_port.read();
            self.data_port.write(cursor_start | 0x20);
        }
    }

    pub fn write_char_at(row: usize, col: usize, character: char) {
        let mut writer = WRITER.lock();
        writer.write_char_at(row, col, character)
    }
}
