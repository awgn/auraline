use owo_colors::Style;

pub fn build_color_style(theme: Option<&str>) -> Style {
    if let Some(theme) = theme {
        if let Some(tuple) = parse_true_color(theme) {
            Style::new().bold().truecolor(tuple.0, tuple.1, tuple.2)
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
                _ => Style::new()
            }
        }
    } else {
        Style::new()
    }
}

pub fn build_bold_style() -> Style {
    Style::new().bold()
}

fn parse_true_color(input: &str) -> Option<(u8, u8, u8)> {
    input
        .split(',')
        .map(|s| s.parse::<u8>())
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .and_then(|v| v.try_into().ok())
        .map(|r: [u8; 3]| (r[0], r[1], r[2]))
}
