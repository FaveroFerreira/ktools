use ratatui::widgets::TableState;

#[derive(Clone, Debug)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}
