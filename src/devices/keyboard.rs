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

            let debug_string: String = buffer.iter().collect();
            println!("CHAR_BUFFER: [{}]", debug_string);
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
                DecodedKey::RawKey(raw_key) => match raw_key {
                    pc_keyboard::KeyCode::Backspace => {
                        println!("Backspace pressed");
                        if !buffer.is_empty() {
                            buffer.pop();
                            console.backspace();
                        }
                    }

                    pc_keyboard::KeyCode::Return => {
                        buffer.push('\n');
                        console.print_char_and_move_cursor('\n');
                    }

                    _ => {}
                },
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
    let mut buffer = String::new();

    loop {
        if let Some(char) = read_char_nonblocking() {
            println!("char: {}", char);
            match char {
                '\n' => {
                    break;
                }
                '\x18' => {
                    buffer.pop();
                }
                '\0' => {
                    continue;
                }
                _ => {
                    buffer.push(char);
                }
            }
        } else {
            hlt();
        }
    }

    buffer
}
