use imports::*;

pub struct LineEdit<'a> {
    label: &'a str,
    placeholder: &'a str,
    text: &'a str,
    /// byte cursor
    cursor: usize,
    focus: bool,
    focus_color: Color,
}

impl<'a> Default for LineEdit<'a> {
    fn default() -> Self {
        LineEdit {
            label: "",
            placeholder: "",
            text: "",
            cursor: 0,
            focus: false,
            focus_color: Color::Reset,
        }
    }
}

impl<'a> LineEdit<'a> {
    pub fn label(mut self, label: &'a str) -> LineEdit<'a> {
        self.label = label;
        self
    }
    pub fn placeholder(mut self, placeholder: &'a str) -> LineEdit<'a> {
        self.placeholder = placeholder;
        self
    }
    pub fn text(mut self, text: &'a str) -> LineEdit<'a> {
        self.text = text;
        self
    }
    pub fn cursor(mut self, cursor: usize) -> LineEdit<'a> {
        self.cursor = cursor;
        self
    }
    pub fn focus(mut self, focus: bool) -> LineEdit<'a> {
        self.focus = focus;
        self
    }
    pub fn focus_color(mut self, focus_color: Color) -> LineEdit<'a> {
        self.focus_color = focus_color;
        self
    }
}

impl<'a> Widget for LineEdit<'a> {
    fn draw(&self, area: &Rect, buf: &mut Buffer) {
        if area.height != 2 { panic!("LineEdit requires exactly two lines of area"); }

        let color = if self.focus { self.focus_color } else { Color::Reset };

        let label_width = self.label.len() as u16 + 1;  // 1 for left border
        let mut label_area = area.clone();
        label_area.width = label_area.width.min(label_width);

        let padding: u16 = 2;

        Paragraph::default()
            .text(self.label)
            .wrap(false)
            .raw(true)
            .block(Block::default()
                .border_style(Style::default().fg(color))
                .borders(border::LEFT | border::BOTTOM))
            .draw(&label_area, buf);

        let mut text_area = area.clone();
        text_area.x += label_width + padding;
        text_area.width -= label_width + padding;

        if self.text.is_empty() {
            Paragraph::default()
                .text(self.placeholder)
                .style(Style::default().fg(Color::Gray))
                .wrap(false)
                .raw(true)
                .block(Block::default()
                    .border_style(Style::default().fg(color))
                    .borders(border::RIGHT | border::BOTTOM))
                .draw(&text_area, buf);
        } else {
            Paragraph::default()
                .text(self.text)
                .wrap(false)
                .raw(true)
                .block(Block::default()
                    .border_style(Style::default().fg(color))
                    .borders(border::RIGHT | border::BOTTOM))
                .draw(&text_area, buf);
        }

        if self.focus {
            let cursor = self.text[..self.cursor].chars().count();
            let x =
                (cursor as u16 + text_area.left())
                    .min(text_area.right() - 1);
            let y = area.top();
            buf.get_mut(x, y)
               .style
               .modifier = Modifier::Invert;
        }
    }
}