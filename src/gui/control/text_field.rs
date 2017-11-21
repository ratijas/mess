use imports::*;


/// Single line of editable text
#[derive(Default)]
pub struct TextField {
    pub buffer: String,
    /// byte cursor.
    /// ranges from 0 to `buffer` boundary inclusive.
    /// `buffer[cursor]` may be an out of bounds error.
    pub cursor: usize,
}

impl TextField {
    pub fn new() -> TextField {
        Default::default()
    }
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

#[allow(unused)]
impl TextField {
    pub fn buffer(mut self, buffer: String) -> TextField {
        self.buffer = buffer;
        self
    }
    pub fn cursor(mut self, cursor: usize) -> TextField {
        assert!(self.buffer.is_char_boundary(cursor));
        self.cursor = cursor;
        self
    }
}

impl TextField {
    fn char_before_cursor(&self) -> Option<char> {
        char_before_index(&self.buffer, self.cursor)
    }
    fn char_at_cursor(&self) -> Option<char> {
        char_at_index(&self.buffer, self.cursor)
    }
    fn word_before_cursor(&self) -> &str {
        word_before_index(&self.buffer, self.cursor)
    }
}

fn char_before_index(s: &str, index: usize) -> Option<char> {
    s[..index].chars().rev().next()
}

fn char_at_index(s: &str, index: usize) -> Option<char> {
    s[index..].chars().next()
}

fn word_before_index(s: &str, index: usize) -> &str {
    let len_s: usize = s[..index]
        .chars()
        .rev()
        .take_while(|&ch| ch.is_whitespace())
        .map(|ch| ch.len_utf8())
        .sum();

    let len_w: usize = s[..index - len_s]
        .chars()
        .rev()
        .take_while(|&ch| !ch.is_whitespace())
        .map(|ch| ch.len_utf8())
        .sum();

    let start = index - len_s - len_w;

    &s[start..index]
}

impl Responder for TextField {
    fn handle_event(&mut self, event: Event) -> bool {
        if let Event::Key(key) = event {
            match key {
                Key::Char(ch) => {
                    self.buffer.insert(self.cursor, ch);
                    self.cursor += ch.len_utf8();
                }
                Key::Backspace => {
                    if self.cursor > 0 {
                        let len = self.char_before_cursor().unwrap().len_utf8();
                        self.buffer.drain((self.cursor - len)..self.cursor);
                        self.cursor -= len;
                    }
                }
                Key::Left => {
                    if let Some(ch) = self.char_before_cursor() {
                        self.cursor -= ch.len_utf8();
                    }
                }
                Key::Right => {
                    if let Some(ch) = self.char_at_cursor() {
                        self.cursor += ch.len_utf8();
                    }
                }
                Key::Ctrl('k') => {
                    self.buffer.truncate(self.cursor);
                }
                Key::Ctrl('u') => {
                    self.buffer.clear();
                    self.cursor = 0;
                }
                Key::Ctrl('w') => {
                    let cursor = self.cursor - self.word_before_cursor().len();

                    self.buffer.drain(cursor..self.cursor);
                    self.cursor = cursor;
                }
                _ => { return false; }
            }
            true
        } else {
            false
        }
    }
}
