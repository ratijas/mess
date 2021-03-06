use imports::*;

#[derive(Default)]
pub struct StatusBar<'a> {
    message: &'a str,
    error: bool
}


impl<'a> StatusBar<'a> {
    pub fn message(mut self, message: &'a str) -> StatusBar<'a> {
        self.message = message;
        self
    }
    pub fn error(mut self, error: bool) -> StatusBar<'a> {
        self.error = error;
        self
    }
}

/// 2 rows height widget
impl<'a> Widget for StatusBar<'a> {
    fn draw(&self, area: &Rect, buf: &mut Buffer) {
        if area.height != 2 { panic!("status bar does not fit"); }

        if self.message.is_empty() {
            let help = "  \
                 {mod=invert ^O} File/Text              \
    {mod=invert F1}-{mod=invert 3} Noise                \
                 {mod=invert F5} Refresh                \
                 {mod=invert ^C} Exit                   \
                 {mod=invert Enter} Send message        ";

            Paragraph::default()
                .text(help)
                .raw(false)
                .block(Block::default()
                    .border_style(Style::default().fg(Color::Reset))
                    .borders(border::LEFT | border::RIGHT | border::BOTTOM))
                .draw(area, buf);
        } else {
            let color = if self.error { Color::Red } else { Color::Green };

            Paragraph::default()
                .text(self.message)
                .raw(true)
                .block(Block::default()
                    .border_style(Style::default().fg(color))
                    .borders(border::LEFT | border::RIGHT | border::BOTTOM))
                .draw(area, buf);
        }
    }
}