use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

use crate::event::{Event, EventObserver};
use crate::term::Term;
use crate::ui::Ui;

mod error;
mod event;
mod handler;
mod term;
mod ui;

pub struct KTools {
    term: Term,
    ui: Ui,
    observer: EventObserver,
}

impl KTools {
    pub fn new() -> Result<Self> {
        let term = Term::init()?;
        let observer = EventObserver::init()?;
        let ui = Ui::init()?;

        Ok(Self { term, ui, observer })
    }

    pub async fn run(mut self) -> Result<()> {
        self.update_ui()?;

        while let Some(event) = self.observer.observe().await {
            self.handle(event)?;
            self.update_ui()?;
        }

        Ok(())
    }

    fn update_ui(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            self.ui.render(frame);
        })?;

        Ok(())
    }

    fn handle(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key(key_event)?,
            Event::Mouse(mouse_event) => self.handle_mouse(mouse_event)?,
            Event::Render => todo!(),
            Event::Quit => self.quit(),
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.ui.is_editing() {
            self.handle_edit(key);
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('?') => self.ui.show_help(),
            KeyCode::Left => self.ui.previous_tab(),
            KeyCode::Right => self.ui.next_tab(),
            KeyCode::Char(':') => self.ui.enter_edit_mode(),
            _ => {}
        }

        Ok(())
    }

    fn handle_edit(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.ui.enter_normal_mode(),
            KeyCode::Enter => self.ui.enter_normal_mode(),
            KeyCode::Backspace => self.ui.delete_char(),
            KeyCode::Char(c) => self.ui.input_char(c),
            _ => {}
        }
    }

    fn handle_mouse(&mut self, _mouse: MouseEvent) -> Result<()> {
        todo!()
    }

    fn quit(&mut self) {
        if self.term.cleanup().is_err() {
            std::process::exit(1);
        }

        std::process::exit(0);
    }
}
