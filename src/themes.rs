use serde::Deserialize;
use std::collections::HashMap;

const TMUX_DEFAULT_STYLE: &str = "#[default]";

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    Normal,

    GitBranch,
    GitStaged,
    GitUnstaged,
    GitUntracked,
    GitAhead,
    GitBehind,

    UserDetails,

    BatteryHigh,
    BatteryMid,
    BatteryLow,
}

pub struct Theme {
    styles: HashMap<Style, String>,
}

impl Theme {
    pub fn from(styles: HashMap<Style, String>) -> Self {
        Self { styles }
    }

    pub fn get_style(&self, link: Style) -> &str {
        match self.styles.get(&link) {
            Some(s) => s,
            None => TMUX_DEFAULT_STYLE,
        }
    }

    pub fn get_style_char(&self, style: Style, ch: char) -> String {
        format!("{}{}{}", self.get_style(style), ch, TMUX_DEFAULT_STYLE)
    }

    pub fn get_style_str(&self, style: Style, text: &str) -> String {
        format!("{}{}{}", self.get_style(style), text, TMUX_DEFAULT_STYLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_char() {
        let test_theme = Theme::from(HashMap::from([(
            Style::Normal,
            String::from("#[fg=red,bg=black]"),
        )]));

        for ch in '\0'..='\u{d7ff}' {
            let actual = test_theme.get_style_char(Style::Normal, ch);
            assert_eq!(actual, format!("#[fg=red,bg=black]{}#[default]", ch));
        }
    }

    #[test]
    fn style_srt() {
        let test_theme = Theme::from(HashMap::from([
            (Style::Normal, String::from("#[fg=red,bg=black]")),
            (Style::BatteryHigh, String::from("#[fg=green,bold]")),
        ]));

        assert_eq!(
            test_theme.get_style_str(Style::Normal, "hello!"),
            "#[fg=red,bg=black]hello!#[default]"
        );
        assert_eq!(
            test_theme.get_style_str(Style::BatteryHigh, "93% 󰂂"),
            "#[fg=green,bold]93% 󰂂#[default]"
        );
        assert_eq!(
            test_theme.get_style_str(Style::GitBranch, " main"),
            "#[default] main#[default]"
        );
    }
}
