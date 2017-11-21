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

    pub fn focus_color(self, app: &::app::App) -> Color {
        match self {
            Mode::Text => Color::Cyan,
            Mode::File => {
                if app.input.buffer.is_empty() {
                    Color::Yellow
                } else {
                    match fs::metadata(Mode::strip_file_uri(&app.input.buffer))
                        .map(|meta| meta.is_file()) {
                        Ok(true) => Color::Green,
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
    /// For File mode: return file content.
    pub fn data_for_input(self, input: &String) -> Result<Vec<u8>> {
        if input.is_empty() {
            Err("Message is empty")?
        } else {
            match self {
                Mode::Text => Ok(input.clone().into_bytes()),
                Mode::File => {
                    let mut content = Vec::new();
                    File::open(Mode::strip_file_uri(input))?
                        .read_to_end(&mut content)?;
                    Ok(content)
                }
            }
        }
    }
}
