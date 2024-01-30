use std::io::{self, Stdout};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::queue;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub struct Term(Terminal<CrosstermBackend<Stdout>>);

impl Term {
    pub fn init() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?;
        queue!(io::stdout(), LeaveAlternateScreen)?;
        terminal.clear()?;

        Ok(Self(terminal))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        queue!(io::stdout(), LeaveAlternateScreen)?;
        self.flush()?;

        Ok(())
    }

    pub fn poll_event(&mut self) -> Result<Option<Event>> {
        let has_event = event::poll(Duration::from_millis(16))?;

        if has_event {
            return Ok(Some(event::read()?));
        }

        Ok(None)
    }
}

impl Deref for Term {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Term {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
