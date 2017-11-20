use super::*;
use ::imports::*;


pub trait Responder {
    /// return true if this controller has captured the event during bubble down or bubble up phase.
    fn handle_event(&mut self, event: Event) -> bool;
}

/// Single line of editable text
#[derive(Default)]
pub struct TextField {
    buffer: String,
    cursor: usize,
}

impl TextField {
    pub fn new() -> TextField {
        Default::default()
    }
    pub fn buffer(mut self, buffer: String) -> TextField {
        self.buffer = buffer;
        self
    }
    pub fn cursor(mut self, cursor: usize) -> TextField {
        self.cursor = cursor;
        self
    }
}

impl Responder for TextField {
    fn handle_event(&mut self, event: Event) -> bool {
        if let Event::Key(key) = event {
            match key {
                Key::Char(ch) => {
                    self.buffer.insert(self.cursor, ch);
                    self.cursor += 1;
                }
                Key::Backspace => {
                    if self.cursor > 0 {
                        self.buffer.remove(self.cursor - 1);
                        self.cursor -= 1;
                    }
                }
                Key::Left => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                }
                Key::Right => {
                    if self.cursor + 1 <= self.buffer.len() {
                        self.cursor += 1;
                    }
                }
                _ => { return false; }
            }
            true
        } else {
            false
        }
    }
}
