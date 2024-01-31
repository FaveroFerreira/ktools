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

        let mut events = Vec::with_capacity(50);

        loop {
            self.observer.observe(&mut events).await;

            for event in events.drain(..) {
                self.handle(event)?;
            }

            events.clear();
            self.update_ui()?;
        }

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
        match key.code {
            KeyCode::Left => self.ui.previous_tab(),
            KeyCode::Right => self.ui.next_tab(),
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('?') => self.ui.toggle_help(),
            KeyCode::Esc => self.ui.toggle_help(),
            _ => {}
        }

        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        todo!()
    }

    fn quit(&mut self) {
        if self.term.cleanup().is_err() {
            std::process::exit(1);
        }

        std::process::exit(0);
    }
}
