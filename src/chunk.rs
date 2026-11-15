use std::fmt::{self, Display, Write};

use owo_colors::Style;
use smallvec::{smallvec, SmallVec};
use smol_str::{format_smolstr, SmolStr};

use crate::options::ShellMode;

pub const LINK_GLYPH: &str = "˖";

pub struct Chunk<T> {
    icon: Option<(SmolStr, Style)>,
    space_after_icon: bool,
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
            space_after_icon: true,
            info: Some((info, Style::new())),
            shell: ShellMode::None,
        }
    }

    pub fn adjoin(icon: impl Into<SmolStr>, info: T) -> Self {
        Self {
            icon: Some((icon.into(), Style::new())),
            space_after_icon: false,
            info: Some((info, Style::new())),
            shell: ShellMode::None,
        }
    }

    pub fn icon(icon: impl Into<SmolStr>) -> Self {
        Self {
            icon: Some((icon.into(), Style::new())),
            space_after_icon: false,
            info: None,
            shell: ShellMode::None,
        }
    }

    pub fn info(info: T) -> Self {
        Self {
            icon: None,
            space_after_icon: false,
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

impl<T: Display> Chunk<T> {
    pub fn into_flat(self) -> FlatChunk {
        let icons = match self.icon {
            Some(pair) => smallvec![pair],
            None => SmallVec::new(),
        };
        let info = self
            .info
            .map(|(val, style)| (format_smolstr!("{val}"), style));
        FlatChunk {
            icons,
            space_after_icon: self.space_after_icon,
            info,
            shell: self.shell,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlatChunk {
    icons: SmallVec<[(SmolStr, Style); 2]>,
    space_after_icon: bool,
    info: Option<(SmolStr, Style)>,
    shell: ShellMode,
}

impl FlatChunk {
    #[inline]
    pub fn can_merge(&self, next: &Self) -> bool {
        match (&self.info, &next.info) {
            (Some((a, _)), Some((b, _))) => !a.is_empty() && a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }

    pub fn merge(&mut self, next: Self) {
        if let (Some((ref mut a, ref mut sa)), Some((b, sb))) = (&mut self.info, next.info) {
            if *a != b && b.chars().any(char::is_uppercase) && !a.chars().any(char::is_uppercase) {
                *a = b;
                *sa = sb;
            }
        }

        for (ic, st) in next.icons {
            if !self.icons.iter().any(|(existing, _)| existing == &ic) {
                self.icons.push((ic, st));
            }
        }

        self.space_after_icon = if self.icons.len() > 1 {
            true
        } else {
            self.space_after_icon || next.space_after_icon
        };
    }
}

pub fn push_coalesced(acc: &mut SmallVec<[FlatChunk; 16]>, chunk: FlatChunk) {
    if let Some(last) = acc.last_mut() {
        if last.can_merge(&chunk) {
            last.merge(chunk);
            return;
        }
    }
    acc.push(chunk);
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
                if !self.space_after_icon && icon_s == info_s {
                    fmt_styled(f, &Adjoin((icon, info)), info_s, self.shell)
                } else {
                    fmt_styled(f, icon, icon_s, self.shell)?;
                    if self.space_after_icon {
                        f.write_str(" ")?;
                    }
                    fmt_styled(f, info, info_s, self.shell)
                }
            }
            (Some((icon, icon_s)), None) => fmt_styled(f, icon, icon_s, self.shell),
            (None, Some((info, info_s))) => fmt_styled(f, info, info_s, self.shell),
            (None, None) => Ok(()),
        }
    }
}

impl Display for FlatChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.icons.as_slice(), &self.info) {
            ([], Some((info, info_s))) => fmt_styled(f, info, info_s, self.shell),
            ([(icon, icon_s)], Some((info, info_s))) => {
                if !self.space_after_icon && icon_s == info_s {
                    fmt_styled(f, &Adjoin((icon, info)), info_s, self.shell)
                } else {
                    fmt_styled(f, icon, icon_s, self.shell)?;
                    if self.space_after_icon {
                        f.write_str(" ")?;
                    }
                    fmt_styled(f, info, info_s, self.shell)
                }
            }
            ([(icon, icon_s)], None) => fmt_styled(f, icon, icon_s, self.shell),
            (icons, info_opt) if icons.len() > 1 => {
                let link_style = Style::new().dimmed();
                for (idx, (icon, icon_s)) in icons.iter().enumerate() {
                    if idx > 0 {
                        fmt_styled(f, &LINK_GLYPH, &link_style, self.shell)?;
                    }
                    fmt_styled(f, icon, icon_s, self.shell)?;
                }
                if let Some((info, info_s)) = info_opt {
                    if self.space_after_icon {
                        f.write_str(" ")?;
                    }
                    fmt_styled(f, info, info_s, self.shell)?;
                }
                Ok(())
            }
            ([], None) => Ok(()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{push_coalesced, Chunk, FlatChunk, Unit};
    use crate::options::ShellMode;
    use owo_colors::Style;
    use smallvec::SmallVec;
    use std::fmt::Write as _;

    fn render_seq(chunks: impl IntoIterator<Item = FlatChunk>) -> String {
        let mut acc = SmallVec::<[FlatChunk; 16]>::new();
        for c in chunks {
            push_coalesced(&mut acc, c);
        }
        let mut out = String::new();
        for c in &acc {
            let _ = write!(out, "{c} ");
        }
        out
    }

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

    #[test]
    fn test_coalesce_chunks_with_link_glyph() {
        // 1. Both without icon, case-insensitive upgrade ("nicola" + "Nicola" -> "Nicola ")
        let s1 = render_seq([
            Chunk::info("nicola").into_flat(),
            Chunk::info("Nicola").into_flat(),
        ]);
        assert_eq!(s1, "Nicola ");

        // 2. Two icons with identical info ("❄ dev" + "⏣ dev" -> "❄˖⏣ dev ")
        let plain = Style::new();
        let s2 = render_seq([
            Chunk::new("❄", "dev")
                .with_style(plain, plain, ShellMode::None)
                .into_flat(),
            Chunk::new("⏣", "dev")
                .with_style(plain, plain, ShellMode::None)
                .into_flat(),
        ]);
        assert_eq!(s2, "❄\x1b[2m˖\x1b[0m⏣ dev ");

        // 3. Adjoin `@` + normal `🖥` with identical host -> `@˖🖥 box `
        let s3 = render_seq([
            Chunk::adjoin("@", "box").into_flat(),
            Chunk::new("🖥", "box").into_flat(),
        ]);
        assert_eq!(s3, "@\x1b[2m˖\x1b[0m🖥 box ");

        // 4. Icon-only chunks (`info: None`) must NEVER merge with each other
        let s4 = render_seq([
            Chunk::<Unit>::icon("⚡").into_flat(),
            Chunk::<Unit>::icon("").into_flat(),
        ]);
        assert_eq!(s4, "⚡  ");

        // 5. Zsh non-printing escape wrapping around `˖`
        let s5 = render_seq([
            Chunk::new("", "app")
                .with_style(Style::new().bold(), plain, ShellMode::Zsh)
                .into_flat(),
            Chunk::new("", "app")
                .with_style(Style::new().bold(), plain, ShellMode::Zsh)
                .into_flat(),
        ]);
        assert_eq!(
            s5,
            "%{\x1b[1m%}%{\x1b[0m%}%{\x1b[2m%}˖%{\x1b[0m%}%{\x1b[1m%}%{\x1b[0m%} app "
        );
    }
}
