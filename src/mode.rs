use imports::*;

/// Represent a mode in which the messenger operates:
/// either sending text messages or sending files.
///
/// In File mode user types in file path instead of text message.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Text,
    File,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Text
    }
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Text => "Text",
            Mode::File => "File",
        }
    }

    pub fn focus_color(self, input: &String) -> Color {
        match self {
            Mode::Text => Color::Cyan,
            Mode::File => {
                if input.is_empty() {
                    Color::Yellow
                } else {
                    let path = Mode::strip_file_uri(input);
                    match File::open(path) {
                        Ok(_) => Color::Green,
                        _ => Color::LightRed,
                    }
                }
            }
        }
    }

    fn strip_file_uri(path: &str) -> &str {
        if path.starts_with("file://") {
            &path["file://".len()..]
        } else {
            path
        }
    }

    /// For Text mode: return the input.
    /// For File mode: return stripped file name.
    pub fn preprocess(self, input: &String) -> Result<String> {
        if input.is_empty() {
            Err("Message is empty")?
        } else {
            match self {
                Mode::Text => Ok(input.clone()),
                Mode::File => {
                    let path = Mode::strip_file_uri(input);
                    File::open(path)?;
                    Ok(path.to_string())
                }
            }
        }
    }
}
