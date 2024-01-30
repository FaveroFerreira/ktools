use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::widgets::Widget;
use ui::Ui;

use crate::term::Term;

mod error;
mod handler;
mod term;
mod ui;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum State {
    Running,
    Refresh,
    Quit,
}

pub struct KTools {
    term: Term,
    state: State,
    ui: Ui,
}

impl KTools {
    pub fn new() -> Result<Self> {
        let term = Term::init()?;

        Ok(Self {
            term,
            ui: Ui::init()?,
            state: State::Running,
        })
    }

    pub fn run(mut self) -> Result<()> {
        while self.state != State::Quit {
            self.update_ui()?;
            self.handle_input()?;
        }

        self.term.cleanup()?;
        Ok(())
    }

    fn update_ui(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();

            self.ui.render(area, buf);
        })?;

        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        if let Some(event) = self.term.poll_event()? {
            match event {
                Event::Key(key) => self.handle_key_press(key.code)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Left => self.ui.previous_tab(),
            KeyCode::Right => self.ui.next_tab(),
            KeyCode::Char('q') => self.state = State::Quit,
            _ => {}
        }

        Ok(())
    }
}
