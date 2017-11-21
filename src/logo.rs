use imports::*;

pub const LOGO_1: &str = include_str!("logo1.txt");
pub const LOGO_2: &str = include_str!("logo2.txt");

pub const SIZE_1: Rect = Rect { x: 0, y: 0, width: 37, height: 5 };
pub const SIZE_2: Rect = Rect { x: 0, y: 0, width: 92, height: 9 };


pub fn logo_for_size(size: Rect) -> String {
    let (logo, inner) = if size.width > SIZE_2.width + 4 && size.height > SIZE_2.height + 2 {
        (LOGO_2, SIZE_2)
    } else {
        (LOGO_1, SIZE_1)
    };
    align_center(size, logo, inner)
}

fn align_center(size: Rect, text: &str, inner: Rect) -> String {
    let left = (size.width - inner.width) as usize / 2;
    let top = (size.height - inner.height) as usize / 2 - 1;
    let mut out = "\n".repeat(top);
    for line in text.lines() {
        out.extend((0..left).map(|_| " "));
        out.push_str(line);
        out.push('\n')
    }
    out
}