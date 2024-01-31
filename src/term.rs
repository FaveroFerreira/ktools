use std::io::{self, Stdout};
use std::ops::{Deref, DerefMut};

use anyhow::Result;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, EnableBracketedPaste, EnableFocusChange,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub struct Term(Terminal<CrosstermBackend<Stdout>>);

impl Term {
    pub fn init() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?;
        queue!(
            io::stdout(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableFocusChange
        )?;

        if supports_keyboard_enhancement()? {
            queue!(
                io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                )
            )?;
        }

        terminal.hide_cursor()?;
        terminal.clear()?;
        terminal.flush()?;

        Ok(Self(terminal))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if supports_keyboard_enhancement()? {
            execute!(io::stdout(), PopKeyboardEnhancementFlags)?;
        }

        execute!(
            io::stdout(),
            DisableFocusChange,
            DisableBracketedPaste,
            LeaveAlternateScreen,
            SetCursorStyle::DefaultUserShape
        )?;

        self.show_cursor()?;
        disable_raw_mode()?;

        Ok(())
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
