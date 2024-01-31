use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent};
use futures::StreamExt;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::{select, spawn};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Render,
    Quit,
}

pub struct EventObserver {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    cancellation_token: CancellationToken,
}

impl EventObserver {
    pub fn init() -> Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();

        let mut observer = Self {
            sender,
            receiver,
            cancellation_token,
        };

        observer.observe_system_events()?;
        observer.observe_terminal_events();

        Ok(observer)
    }

    pub async fn observe(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    pub fn observe_system_events(&mut self) -> Result<JoinHandle<()>> {
        let sender = self.sender.clone();

        let mut stream = Signals::new([SIGINT, SIGTERM, SIGQUIT, SIGHUP])?;

        Ok(spawn(async move {
            while let Some(signal) = stream.next().await {
                match signal {
                    SIGINT | SIGTERM | SIGQUIT | SIGHUP => {
                        if sender.send(Event::Quit).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }))
    }

    pub fn observe_terminal_events(&mut self) -> JoinHandle<()> {
        let mut stream = EventStream::new();
        let sender = self.sender.clone();
        let cancellation = self.cancellation_token.clone();

        spawn(async move {
            loop {
                select! {
                        _ = cancellation.cancelled() => break,
                        Some(Ok(crossterm_event)) = stream.next() => {
                            let event = match crossterm_event {
                                CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => Event::Key(key),
                                _ => continue,
                            };

                            if sender.send(event).is_err() {
                                break;
                            }
                        }
                }
            }
        })
    }

    pub fn resume(&mut self) {
        if self.cancellation_token.is_cancelled() {
            self.cancellation_token = CancellationToken::new();
        }
    }

    pub fn stop(&mut self) {
        if !self.cancellation_token.is_cancelled() {
            self.cancellation_token.cancel()
        }
    }
}
