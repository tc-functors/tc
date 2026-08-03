use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

fn color_from_fg(code: u32) -> Option<Color> {
    match code {
        30 => Some(Color::Black),
        31 => Some(Color::Red),
        32 => Some(Color::Green),
        33 => Some(Color::Yellow),
        34 => Some(Color::Blue),
        35 => Some(Color::Magenta),
        36 => Some(Color::Cyan),
        37 => Some(Color::White),
        // 90-97 bright
        90 => Some(Color::DarkGray),
        91 => Some(Color::LightRed),
        92 => Some(Color::LightGreen),
        93 => Some(Color::LightYellow),
        94 => Some(Color::LightBlue),
        95 => Some(Color::LightMagenta),
        96 => Some(Color::LightCyan),
        97 => Some(Color::Gray),
        _ => None,
    }
}

fn color_from_bg(code: u32) -> Option<Color> {
    match code {
        40 => Some(Color::Black),
        41 => Some(Color::Red),
        42 => Some(Color::Green),
        43 => Some(Color::Yellow),
        44 => Some(Color::Blue),
        45 => Some(Color::Magenta),
        46 => Some(Color::Cyan),
        47 => Some(Color::White),
        // 100-107 bright
        100 => Some(Color::DarkGray),
        101 => Some(Color::LightRed),
        102 => Some(Color::LightGreen),
        103 => Some(Color::LightYellow),
        104 => Some(Color::LightBlue),
        105 => Some(Color::LightMagenta),
        106 => Some(Color::LightCyan),
        107 => Some(Color::Gray),
        _ => None,
    }
}

fn apply_sgr(style: &mut Style, params: &[u32]) {
    if params.is_empty() {
        return;
    }

    for &p in params {
        match p {
            0 => {
                *style = Style::default();
            }
            1 => {
                let _ = style.add_modifier(Modifier::BOLD);
            }
            2 => {
                let _ = style.remove_modifier(Modifier::BOLD);
            }
            3 => {
                let _ = style.add_modifier(Modifier::ITALIC);
            }
            4 => {
                let _ = style.add_modifier(Modifier::UNDERLINED);
            }
            5 => {
                let _ = style.add_modifier(Modifier::SLOW_BLINK);
            }
            30..=37 | 90..=97 => {
                if let Some(c) = color_from_fg(p) {
                    *style = style.fg(c);
                }
            }
            40..=47 | 100..=107 => {
                if let Some(c) = color_from_bg(p) {
                    *style = style.bg(c);
                }
            }
            _ => {
                // ignore unsupported SGR codes for this small converter
            }
        }
    }
}

/// Convert a string containing ANSI SGR escape sequences into Ratatui spans.
///
/// Supports: ESC [ ... m sequences (e.g. "\x1b[31mRED\x1b[0m").

pub fn ansi_to_spans(input: &str) -> Vec<Span<'static>> {

    let bytes = input.as_bytes();
    let mut i = 0;

    let mut current = String::new();
    let mut spans: Vec<Span<'static>> = Vec::new();


    let mut style = Style::default();

    let flush = |current: &mut String, spans: &mut Vec<Span<'static>>, style: &Style| {
        if !current.is_empty() {
            let text = std::mem::take(current);
            spans.push(Span::styled(text, *style));
        }
    };

    while i < bytes.len() {

        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            flush(&mut current, &mut spans, &style);

            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i] != b'm' {
                i += 1;
            }
            if i >= bytes.len() {
                current.push_str(&input[start - 2..]);
                break;
            }

            let inside = &input[start..i];
            // parse params separated by ';'
            let params: Vec<u32> = inside
                .split(';')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();

            apply_sgr(&mut style, &params);

            i += 1;
        } else {
            // normal character
            current.push(bytes[i] as char);
            i += 1;
        }
    }

    if !current.is_empty() {
        let text = std::mem::take(&mut current);
        spans.push(Span::styled(text, style));
    }

    spans
}
