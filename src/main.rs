mod order;
mod orderbook;
mod feed_simulator;
mod statistics;

use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{self, Event as CEvent, KeyCode},
};
use orderbook::OrderBook;
use feed_simulator::FeedSimulator;
use statistics::Stats;

use crate::order::{Event, Order};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{io, sync::mpsc, time::Duration, };
use std::thread;
use order::Side;

fn main() -> io::Result<()>{    
    // --- terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    
    let (order_tx, order_rx) = mpsc::channel::<Order>();
    spawn_simulator(order_tx);

    let mut app = App::new(order_rx);
    app.run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn spawn_simulator(order_tx: mpsc::Sender<Order>) {
    thread::spawn(move || {
        let mut simulator = FeedSimulator::new(5.0, 2.0);
        loop {
            let (order, gap) = simulator.generate_next();
            thread::sleep(Duration::from_secs_f64(gap.min(1.0)));
            if order_tx.send(order).is_err() {
                break; // receiver was dropped - main thread exited, stop generating
            }
        }
    });
}
struct App {
    book: OrderBook,
    events: Vec<Event>,
    order_rx: mpsc::Receiver<Order>,
    statistics: Stats,
    exit:bool,
}

impl App {
    fn new(order_rx: mpsc::Receiver<Order>) -> Self {
        App {
            book: OrderBook::new(),
            events: Vec::new(),
            order_rx,
            statistics: Stats::new(),
            exit: false,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.exit {
            while let Ok(order) = self.order_rx.try_recv() {
                let outcome = self.book.submit(order);
                for event in &outcome {
                    self.statistics.record_event(event);
                }
                self.events.extend(outcome);
            }

            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(50))? {
                if let CEvent::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        self.exit = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
            // Split view into chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(frame.area());

            let log_items: Vec<ListItem> = self.events.iter()
            .rev()
            .take(30)
            .map(|e| ListItem::new(e.to_log_line()))
            .collect();

            let (best_bid, best_ask) = {
                let bids = self.book.get_outstanding(Side::Buy);
                let asks = self.book.get_outstanding(Side::Sell);
                let best_bid = bids.iter().map(|o| o.price).max();
                let best_ask = asks.iter().map(|o| o.price).min();
                (best_bid, best_ask)
            };

            let spread_text = match (best_bid, best_ask) {
                (Some(b), Some(a)) => format!(
                    "Bid: {}  Ask: {}  Spread: {}  Trades: {}  Vol: {}  VWAP: {:.2}",
                    b, a, a.saturating_sub(b), self.statistics.trade_count, self.statistics.total_volume,
                    self.statistics.vwap().unwrap_or(0.0)
                ),
                _ => "Waiting for liquidity...".to_string(),
            };

            // Top view
            let stats_widget = Paragraph::new(spread_text)
                .block(Block::default().borders(Borders::ALL).title("Market Stats"));
            frame.render_widget(stats_widget, chunks[0]);
        
            // Mid view
            let log_list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title("Trade Log"));
            frame.render_widget(log_list, chunks[1]);
            
            // Bottom view (bids & asks)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);

            let mut bids: Vec<Order> = self.book.get_outstanding(Side::Buy);
            // Sort highest to lowest
            bids.sort_by(|a, b| b.price.cmp(&a.price));

            let bid_lines: Vec<String> = bids.iter()
                .map(|o| format!("#{} {} @ {}", o.id, o.quantity, o.price))
                .collect();

            let buy_widget = List::new(bid_lines)
                .block(Block::default().borders(Borders::ALL).title("Bids"));
            frame.render_widget(buy_widget, bottom_chunks[0]);
            
            let mut asks: Vec<Order> = self.book.get_outstanding(Side::Sell);
            // Sort lowest to highest
            asks.sort_by_key(|o| o.price);

            let ask_lines: Vec<String> = asks.iter()
                .map(|o| format!("#{} {} @ {}", o.id, o.quantity, o.price))
                .collect();

            let sell_widget = List::new(ask_lines)
                .block(Block::default().borders(Borders::ALL).title("Asks"));
            frame.render_widget(sell_widget, bottom_chunks[1]);
    }
}