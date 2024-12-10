use crate::devices::console::CONSOLE;
use crate::println;
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::{hlt, port::Port};

lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore //Ignore map ctrl + a-z
        ));
    pub static ref CHAR_BUFFER: Mutex<Vec<char>> = Mutex::new(Vec::new());
}

// Reads a scancode from the keyboard port
pub fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

pub fn process_scancode(scancode: u8) {
    // Ignore KeyUp events (scancode >= 0x80)
    if scancode & 0x80 != 0 {
        return;
    }

    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let mut buffer = CHAR_BUFFER.lock();
            let mut console = CONSOLE.lock();

            match key {
                DecodedKey::Unicode(character) => {
                    if character == '\x08' {
                        if !buffer.is_empty() {
                            buffer.pop();
                            console.backspace();
                        }
                        return;
                    }

                    buffer.push(character);
                    console.print_char_and_move_cursor(character);
                }
                DecodedKey::RawKey(..) => {}
            }
        } else {
            println!("failed to decode key event");
        }
    }
}

pub fn read_char_nonblocking() -> Option<char> {
    let mut buffer = CHAR_BUFFER.lock();
    buffer.pop()
}

pub fn read_char() -> char {
    loop {
        let mut buffer = CHAR_BUFFER.lock();
        if let Some(c) = buffer.pop() {
            return c;
        }
    }
}

pub fn read_line() -> String {
    {
        let mut console = CONSOLE.lock();
        console.print_char_and_move_cursor('>');
        console.print_char_and_move_cursor(' ');
    }

    loop {
        let newline_found = {
            let buffer = CHAR_BUFFER.lock();
            buffer.last() == Some(&'\n')
        };

        if newline_found {
            let result: String = {
                let mut buffer = CHAR_BUFFER.lock();
                let collected = buffer.iter().collect();
                buffer.clear();
                collected
            };

            let trimmed_result = result.trim_end_matches('\n');
            return String::from(trimmed_result);
        }

        hlt();
    }
}
