use ratatui::style::{Color, Modifier, Style};

/// Theme colors inspired by GitHub's diff UI
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub added_bg: Color,
    pub added_fg: Color,
    pub removed_bg: Color,
    pub removed_fg: Color,
    pub context_fg: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            header_bg: Color::DarkGray,
            header_fg: Color::White,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
            added_bg: Color::Rgb(22, 77, 37),      // GitHub green
            added_fg: Color::Rgb(167, 255, 164),   // Light green text
            removed_bg: Color::Rgb(136, 23, 27),   // GitHub red
            removed_fg: Color::Rgb(255, 153, 164), // Light red text
            context_fg: Color::Gray,
            border: Color::DarkGray,
        }
    }
}

impl Theme {
    pub fn header_style(&self) -> Style {
        Style::default()
            .bg(self.header_bg)
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.selected_bg)
            .fg(self.selected_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn added_style(&self) -> Style {
        Style::default().bg(self.added_bg).fg(self.added_fg)
    }

    pub fn removed_style(&self) -> Style {
        Style::default().bg(self.removed_bg).fg(self.removed_fg)
    }

    pub fn context_style(&self) -> Style {
        Style::default().fg(self.context_fg)
    }

    pub fn normal_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.fg)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
}
