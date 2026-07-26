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

use crate::order::{Event, Order};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, macros::constraint, widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{fmt::format, io};
use order::Side;

fn main() -> io::Result<()>{
    let mut book = OrderBook::new();
    let mut events: Vec<Event> = Vec::new();
    let mut simulator = FeedSimulator::new(5.0, 2.0);

    // --- terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // --- app loop ---
    loop {
        // advance simulation by one order per tick
        let (order, gap) = simulator.generate_next();
        let outcome = book.submit(order);
        events.extend(outcome);

        terminal.draw(|frame| {
            // Top view
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(frame.area());

            // Bottom view (bids & asks)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let log_items: Vec<ListItem> = events.iter()
                .rev()
                .take(30)
                .map(|e| ListItem::new(e.to_log_line()))
                .collect();

            let log_list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title("Trade Log"));
            frame.render_widget(log_list, chunks[0]);

            let mut bids: Vec<Order> = book.get_outstanding(Side::Buy);
            // Sort highest to lowest
            bids.sort_by(|a, b| b.price.cmp(&a.price));

            let bid_lines: Vec<String> = bids.iter()
                .map(|o| format!("#{} {} @ {}", o.id, o.quantity, o.price))
                .collect();

            let buy_widget = List::new(bid_lines)
                .block(Block::default().borders(Borders::ALL).title("Bids"));
            frame.render_widget(buy_widget, bottom_chunks[0]);
            
            let mut asks: Vec<Order> = book.get_outstanding(Side::Sell);
            // Sort lowest to highest
            asks.sort_by_key(|o| o.price);

            let ask_lines: Vec<String> = asks.iter()
                .map(|o| format!("#{} {} @ {}", o.id, o.quantity, o.price))
                .collect();

            let sell_widget = List::new(ask_lines)
                .block(Block::default().borders(Borders::ALL).title("Asks"));
            frame.render_widget(sell_widget, bottom_chunks[1]);
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