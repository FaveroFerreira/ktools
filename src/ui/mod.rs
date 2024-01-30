use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Tabs, Widget};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::ui::theme::Theme;

mod theme;
mod utils;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, FromRepr, EnumIter, Display)]
pub enum Tab {
    #[default]
    #[strum(to_string = "Context")]
    Context,
    #[strum(to_string = "Kafka")]
    Kafka,
    #[strum(to_string = "Schema Registry")]
    SchemaRegistry,
}

pub struct Ui {
    tab: Tab,
    theme: Theme,
}

impl Ui {
    pub fn init() -> Result<Self> {
        Ok(Self {
            tab: Tab::default(),
            theme: Theme::load()?,
        })
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(3), // Tabs
                Constraint::Length(3), // Content
            ])
            .direction(Direction::Vertical)
            .split(area);

        self.render_title(chunks[0], buf);
        self.render_tabs(chunks[1], buf);
        self.render_content(chunks[2], buf);
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::raw("KTools"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .render(area, buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = Tab::iter().map(|tab| tab.to_string()).collect::<Vec<_>>();
        let selected_tab_index = self.tab as usize;

        let block = Block::new()
            .title("TABS")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_style.into());

        Tabs::new(titles)
            .block(block)
            .select(selected_tab_index)
            .highlight_style(self.theme.tab_active.into())
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title("CONTENT")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_style.into());

        Paragraph::new(self.tab.to_string())
            .block(block)
            .render(area, buf);
    }

    pub fn next_tab(&mut self) {
        let current_index = self.tab as usize;
        let next_index = current_index.saturating_add(1);

        self.tab = Tab::iter().cycle().nth(next_index).unwrap_or(self.tab);
    }

    pub fn previous_tab(&mut self) {
        let current_index = self.tab as usize;
        let previous_index = current_index.saturating_sub(1);

        self.tab = Tab::iter().cycle().nth(previous_index).unwrap_or(self.tab);
    }
}
