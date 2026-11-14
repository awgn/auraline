use std::fmt::{self, Display, Write};

use owo_colors::Style;
use smol_str::SmolStr;

use crate::options::ShellMode;

pub struct Chunk<T> {
    icon: Option<(SmolStr, Style)>,
    info: Option<(T, Style)>,
    shell: ShellMode,
}

#[derive(Default, Debug)]
pub struct Unit;
impl Display for Unit {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub struct Adjoin<T, V>(pub (T, V));
impl<T, V> Display for Adjoin<T, V>
where
    T: Display,
    V: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.0 .0, self.0 .1)
    }
}

impl<T, V> Default for Adjoin<T, V>
where
    T: Default,
    V: Default,
{
    fn default() -> Self {
        Self((T::default(), V::default()))
    }
}

impl<T> Chunk<T> {
    pub fn new(icon: impl Into<SmolStr>, info: T) -> Self {
        Self {
            icon: Some((icon.into(), Style::new())),
            info: Some((info, Style::new())),
            shell: ShellMode::None,
        }
    }

    pub fn icon(icon: impl Into<SmolStr>) -> Self {
        Self {
            icon: Some((icon.into(), Style::new())),
            info: None,
            shell: ShellMode::None,
        }
    }

    pub fn info(info: T) -> Self {
        Self {
            icon: None,
            info: Some((info, Style::new())),
            shell: ShellMode::None,
        }
    }

    pub fn with_style(mut self, icon_s: Style, info_s: Style, shell: ShellMode) -> Self {
        if let Some((_, ref mut s)) = self.icon {
            *s = icon_s;
        }
        if let Some((_, ref mut s)) = self.info {
            *s = info_s;
        }
        self.shell = shell;
        self
    }
}

struct ZshEscapeWriter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl Write for ZshEscapeWriter<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while let Some(idx) = s.find('%') {
            self.0.write_str(&s[..idx])?;
            self.0.write_str("%%")?;
            s = &s[idx + 1..];
        }
        self.0.write_str(s)
    }
}

fn fmt_styled<V: Display>(
    f: &mut fmt::Formatter<'_>,
    val: &V,
    style: &Style,
    shell: ShellMode,
) -> fmt::Result {
    if style.is_plain() {
        return match shell {
            ShellMode::Zsh => write!(ZshEscapeWriter(f), "{val}"),
            ShellMode::None | ShellMode::Bash => write!(f, "{val}"),
        };
    }

    match shell {
        ShellMode::None => {
            style.fmt_prefix(f)?;
            write!(f, "{val}")?;
            style.fmt_suffix(f)
        }
        ShellMode::Bash => {
            f.write_str("\x01")?;
            style.fmt_prefix(f)?;
            f.write_str("\x02")?;
            write!(f, "{val}")?;
            f.write_str("\x01")?;
            style.fmt_suffix(f)?;
            f.write_str("\x02")
        }
        ShellMode::Zsh => {
            f.write_str("%{")?;
            style.fmt_prefix(f)?;
            f.write_str("%}")?;
            write!(ZshEscapeWriter(f), "{val}")?;
            f.write_str("%{")?;
            style.fmt_suffix(f)?;
            f.write_str("%}")
        }
    }
}

impl<T: Display> Display for Chunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.icon, &self.info) {
            (Some((icon, icon_s)), Some((info, info_s))) => {
                fmt_styled(f, icon, icon_s, self.shell)?;
                f.write_str(" ")?;
                fmt_styled(f, info, info_s, self.shell)
            }
            (Some((icon, icon_s)), None) => fmt_styled(f, icon, icon_s, self.shell),
            (None, Some((info, info_s))) => fmt_styled(f, info, info_s, self.shell),
            (None, None) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chunk;
    use crate::options::ShellMode;
    use owo_colors::Style;

    #[test]
    fn test_shell_escaping_bash_and_zsh() {
        let bold = Style::new().bold();

        let bash = Chunk::new("μ", "60.5%")
            .with_style(bold, bold, ShellMode::Bash)
            .to_string();
        assert_eq!(
            bash,
            "\x01\x1b[1m\x02μ\x01\x1b[0m\x02 \x01\x1b[1m\x0260.5%\x01\x1b[0m\x02"
        );

        let zsh = Chunk::new("μ", "60.5%")
            .with_style(bold, bold, ShellMode::Zsh)
            .to_string();
        assert_eq!(zsh, "%{\x1b[1m%}μ%{\x1b[0m%} %{\x1b[1m%}60.5%%%{\x1b[0m%}");
    }
}
