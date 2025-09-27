use owo_colors::Style;
use smallvec::SmallVec;
use smol_str::{SmolStr, SmolStrBuilder};

pub fn build_color_style(theme: Option<&str>) -> Style {
    if let Some(theme) = theme {
        if let Some(tuple) = parse_true_color(theme) {
            Style::new().truecolor(tuple.0, tuple.1, tuple.2)
        } else {
            match theme {
                "black" => Style::new().black(),
                "red" => Style::new().red(),
                "green" => Style::new().green(),
                "yellow" => Style::new().yellow(),
                "blue" => Style::new().blue(),
                "magenta" => Style::new().magenta(),
                "cyan" => Style::new().cyan(),
                "white" => Style::new().white(),
                "purple" => Style::new().purple(),
                "bright_black" => Style::new().bright_black(),
                "bright_red" => Style::new().bright_red(),
                "bright_green" => Style::new().bright_green(),
                "bright_yellow" => Style::new().bright_yellow(),
                "bright_purple" => Style::new().bright_purple(),
                "bright_blue" => Style::new().bright_blue(),
                "bright_magenta" => Style::new().bright_magenta(),
                "bright_cyan" => Style::new().bright_cyan(),
                "bright_white" => Style::new().bright_white(),
                _ => Style::new(),
            }
        }
    } else {
        Style::new()
    }
}

fn parse_true_color(input: &str) -> Option<(u8, u8, u8)> {
    input
        .split(',')
        .map(|s| s.trim().parse::<u8>())
        .collect::<Result<SmallVec<[u8; 3]>, _>>()
        .ok()
        .and_then(|v| (v.len() == 3).then_some((v[0], v[1], v[2])))
}

pub fn to_superscript(s: &str) -> SmolStr {
    const SUPERSCRIPT_DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut builder = SmolStrBuilder::new();
    for c in s.chars() {
        builder.push(if c.is_ascii_digit() {
            SUPERSCRIPT_DIGITS[c as usize - '0' as usize]
        } else {
            c
        });
    }
    builder.finish()
}
