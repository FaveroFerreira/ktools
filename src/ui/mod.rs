use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::ui::theme::Theme;

mod component;
mod theme;
mod utils;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Editing,
}

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
    mode: Mode,
    input: String,
    cursor_position: usize,
    show_help: bool,
}

impl Ui {
    pub fn init() -> Result<Self> {
        Ok(Self {
            tab: Tab::default(),
            theme: Theme::load()?,
            mode: Mode::Normal,
            input: String::new(),
            cursor_position: 0,
            show_help: false,
        })
    }

    pub fn render(&self, frame: &mut Frame) {
        if self.show_help {
            frame.render_widget(Clear, frame.size());
            frame.render_widget(self.help(), frame.size());
            return;
        }

        let constraints = match self.mode {
            Mode::Editing => vec![
                Constraint::Percentage(10), // Header
                Constraint::Percentage(10), // Input
                Constraint::Percentage(80), // Content
            ],
            Mode::Normal => vec![
                Constraint::Percentage(10), // Header
                Constraint::Percentage(90), // Content
            ],
        };

        let chunks = Layout::default()
            .constraints(constraints)
            .direction(Direction::Vertical)
            .split(frame.size());

        frame.render_widget(self.header(), chunks[0]);

        match self.mode {
            Mode::Normal => frame.render_widget(self.content(), chunks[1]),
            Mode::Editing => {
                frame.render_widget(self.input(), chunks[1]);
                frame.set_cursor(
                    chunks[1].x + self.cursor_position as u16 + 1,
                    chunks[1].y + 1,
                );
                frame.render_widget(self.content(), chunks[2]);
            }
        }
    }

    fn header(&self) -> Paragraph {
        Paragraph::new(Text::raw("KTools"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD))
    }

    fn input(&self) -> Paragraph {
        let block = Block::new()
            .title("COMMAND")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_style.into());

        Paragraph::new(self.input.as_str()).block(block)
    }

    fn content(&self) -> Paragraph {
        let block = Block::new()
            .title("CONTENT")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_style.into());

        Paragraph::new(self.tab.to_string()).block(block)
    }

    fn help(&self) -> Paragraph {
        let block = Block::new()
            .title("HELP")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_style.into());

        let lines = vec![
            Line::from("Press "),
            Line::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
            Line::from(" to switch tabs"),
        ];

        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left)
    }

    fn clamp_cursor(&self, position: usize) -> usize {
        position.clamp(0, self.input.len())
    }

    fn move_cursor_right(&mut self) {
        let new_cursor_pos = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(new_cursor_pos);
    }

    fn move_cursor_left(&mut self) {
        let new_cursor_pos = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(new_cursor_pos);
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    pub fn input_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);

        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let current_index = self.cursor_position;
        let from_left_to_current_index = current_index - 1;

        let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
        let after_char_to_delete = self.input.chars().skip(current_index);

        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Editing
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn enter_edit_mode(&mut self) {
        self.mode = Mode::Editing;
    }

    pub fn show_help(&mut self) {
        self.show_help = true;
    }

    pub fn hide_help(&mut self) {
        self.show_help = false;
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
