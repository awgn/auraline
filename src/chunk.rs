use std::fmt::Display;

use owo_colors::style;
use owo_colors::Style;
use owo_colors::Styled;

pub struct Chunk<T> {
    icon: Option<Styled<&'static str>>,
    info: Option<Styled<T>>,
}

#[derive(Default, Debug)]
pub struct Unit;
impl Display for Unit {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub struct Adjoin<T, V>(pub (T, V));
impl<T, V> Display for Adjoin<T, V>
where
    T: Display,
    V: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl<T: Default> Chunk<T> {
    pub fn new(icon: &'static str, info: T) -> Self {
        Self {
            icon: Some(style().style(icon)),
            info: Some(style().style(info)),
        }
    }

    pub fn icon(icon: &'static str) -> Self {
        Self {
            icon: Some(style().style(icon)),
            info: None,
        }
    }

    pub fn info(info: T) -> Self {
        Self {
            icon: None,
            info: Some(style().style(info)),
        }
    }

    pub fn with_style(mut self, icon_s: Style, info_s: Style) -> Self {
        let icon = self.icon.as_mut().map(|i| std::mem::take(i.inner_mut()));
        let info = self.info.as_mut().map(|i| std::mem::take(i.inner_mut()));
        Self {
            icon: icon.map(|i| icon_s.style(i)),
            info: info.map(|i| info_s.style(i)),
        }
    }
}

impl<T: Display> Display for Chunk<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.icon, &self.info) {
            (Some(icon), Some(info)) => write!(f, "{icon} {info}"),
            (Some(icon), None) => write!(f, "{icon}"),
            (None, Some(info)) => write!(f, "{info}"),
            (None, None) => Ok(()),
        }
    }
}
