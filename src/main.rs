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
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, Paragraph},
};
use ratatui::style::Color;
use ratatui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
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
        let mut simulator = FeedSimulator::new(10.0, 2.0);
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

            // Get bid and ask BTree
            let bids: std::collections::BTreeMap<u64, Vec<Order>> = self.book.get_bids();
            let asks: std::collections::BTreeMap<u64, Vec<Order>> = self.book.get_asks();

            let (best_bid, best_ask) = {
                let best_bid = bids.last_key_value().map(|(price, _)| *price);
                let best_ask = asks.first_key_value().map(|(price, _)| *price);
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
        
            // let log_items: Vec<ListItem> = self.events.iter()
            // .rev()
            // .take(30)
            // .map(|e| ListItem::new(e.to_log_line()))
            // .collect();

            let visible_candles = &self.statistics.candles;
            let min_price = visible_candles.iter().map(|o| o.low).min().unwrap_or(0) as f64;
            let max_price = visible_candles.iter().map(|o| o.high).max().unwrap_or(0) as f64;
            let x_max = 3.0 * 30.0 + 1.0;

            let canvas_widget = Canvas::default()
            .block(Block::bordered().title("Canvas"))
            .x_bounds([0.0, x_max])
            .y_bounds([min_price, max_price])
            .paint(|ctx| {
                
                let next_index = self.statistics.candles.len();
                for (index, candle) in self.statistics.candles.iter().enumerate().rev().take(30) {
                    let color = if candle.close >= candle.open { Color::Green} else { Color::Red };

                    ctx.draw(&Line {
                        x1: index as f64 * 3.0 + 1.0,
                        y1: candle.low as f64 ,
                        x2: index as f64 * 3.0 + 1.0,
                        y2: candle.high as f64,
                        color,
                    });
                    ctx.draw(&Rectangle {
                        x: index as f64 * 3.0,
                        y: candle.open.min(candle.close) as f64,
                        width: 2.0,
                        height: (candle.open as f64 - candle.close as f64).abs().max(0.5),
                        color,
                    });
                }

                if let Some(current_candle) = &self.statistics.current_candle {
                    let color = if current_candle.close >= current_candle.open { Color::Green} else { Color::Red };
                    ctx.draw(&Line {
                        x1: next_index as f64 * 3.0 + 1.0,
                        y1: current_candle.low as f64 ,
                        x2: next_index as f64 * 3.0 + 1.0,
                        y2: current_candle.high as f64,
                        color,
                    });
                    ctx.draw(&Rectangle {
                        x: next_index as f64 * 3.0,
                        y: current_candle.open.min(current_candle.close) as f64,
                        width: 2.0, 
                        height: (current_candle.open as f64 - current_candle.close as f64).abs().max(0.5),
                        color,
                    });
                };
            });


            frame.render_widget(canvas_widget, chunks[1]);


            // Mid view
            // let log_list = List::new(log_items)
            //     .block(Block::default().borders(Borders::ALL).title("Trade Log"));
            // frame.render_widget(log_list, chunks[1]);
            
            // Bottom view (bids & asks)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);
            
            let max_bid_quantity = bids.values()
            .map(|orders| orders.iter().map(|o| o.quantity).sum::<u64>())
            .max()
            .unwrap_or(0);
        
            let max_ask_quantity = asks.values()
                .map(|orders| orders.iter().map(|o| o.quantity).sum::<u64>())
                .max()
                .unwrap_or(0);
            
            let mut bid_levels: Vec<(u64, u64)> = bids.iter()
            .map(|(price, orders)| (*price, orders.iter().map(|o| o.quantity).sum::<u64>()))
            .collect();
            bid_levels.sort_by(|a, b| b.0.cmp(&a.0)); // Descending
        
            let mut ask_levels: Vec<(u64, u64)> = asks.iter()
            .map(|(price, orders)| (*price, orders.iter().map(|o| o.quantity).sum::<u64>()))
            .collect();
            ask_levels.sort_by(|a, b| a.0.cmp(&b.0)); // Ascending
            
            let bar_width = 50;

            // Bid
            let bid_lines: Vec<String> = bid_levels.iter()
            .map(|(price, quantity)| {
                let filled = ((*quantity as f64 / max_bid_quantity as f64) * bar_width as f64).round() as usize;
                let bar = "█".repeat(filled);
                format!("{:>5} orders @ {:>5} {}", quantity, price, bar)
            })
            .collect();
        
        
            let buy_widget = List::new(bid_lines)
                .block(Block::default().borders(Borders::ALL).title("Bids"));
            frame.render_widget(buy_widget, bottom_chunks[0]);

            // Ask
            let ask_lines: Vec<String> = ask_levels.iter()
            .map(|(price, quantity)| {
                let filled = ((*quantity as f64 / max_ask_quantity as f64) * bar_width as f64).round() as usize;
                let bar = "█".repeat(filled);
                format!("{:>5} orders @ {:>5} {}", quantity, price, bar)
            })
            .collect();

            let ask_widget = List::new(ask_lines)
                .block(Block::default().borders(Borders::ALL).title("Asks"));
            frame.render_widget(ask_widget, bottom_chunks[1]);
    }
}