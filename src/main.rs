mod order;
mod orderbook;
mod feed_simulator;

use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{self, Event as CEvent, KeyCode},
};
use orderbook::OrderBook;
use feed_simulator::FeedSimulator;

use crate::order::Event;
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, Paragraph},
};
use core::time;
use std::io;
use order::Side;

fn main() -> io::Result<()>{
    let mut book = OrderBook::new();
    let mut events: Vec<Event> = Vec::new();
    let mut simulator = FeedSimulator::new(1.0, 1.0);

    // --- terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen);
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // --- app loop ---
    loop {
        // advance simulation by one order per tick
        let (order, gap) = simulator.generate_next();
        let outcome = book.submit(order);
        events.extend(outcome);

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(frame.area());

            let log_items: Vec<ListItem> = events.iter()
                .rev()
                .take(20)
                .map(|e| ListItem::new(e.to_log_line()))
                .collect();

            let log_list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title("Trade Log"));
            frame.render_widget(log_list, chunks[0]);

            let book_text = format!(
                "Bids: {:?}\n\nAsks: {:?}",
                book.get_outstanding(Side::Buy),
                book.get_outstanding(Side::Sell)
            );
            let book_widget = Paragraph::new(book_text)
                .block(Block::default().borders(Borders::ALL).title("Order Book"));
            frame.render_widget(book_widget, chunks[1]);
        })?;

        let wait = std::time::Duration::from_secs_f64(gap.min(1.0));
        if event::poll(wait)? {
            if let CEvent::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}