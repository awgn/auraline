use std::fmt::Display;

use colored::Color;
use colored::ColoredString;
use colored::Colorize;

pub trait ColorizeExt {
    fn color<C: Into<Color> + ToString + Display>(self, c: Option<C>) -> Option<ColoredString>;
    fn bold(self) -> Option<ColoredString>;
}

impl<S> ColorizeExt for Option<S>
where
    S: Into<ColoredString>,
{
    fn color<C>(self, c: Option<C>) -> Option<ColoredString>
    where
        C: Into<Color>,
        C: ToString + Display,
    {
        self.map(|this| {
            if let Some(color) = c {
                match parse_true_color(&color.to_string()) {
                    Some((r, g, b)) => this.into().truecolor(r, g, b),
                    None => this.into().color(color),
                }
            } else {
                this.into()
            }
        })
    }

    fn bold(self) -> Option<ColoredString> {
        self.map(|s| s.into().bold())
    }
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
