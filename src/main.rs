mod order;
mod orderbook;
mod feed_simulator;

use std::println;

use order::{Side};
use orderbook::OrderBook;
use feed_simulator::FeedSimulator;

use crate::order::Event;

fn main() {
    let mut book = OrderBook::new();
    let mut events: Vec<Event> = Vec::new();
    let mut simulator = FeedSimulator::new(1.0, 1.0);

    for _ in 0..10 {
        let order = simulator.generate_next();
        let outcome = book.submit(order.0);
        for event in &outcome {
            println!("{}", event.to_log_line());
        }
        events.extend(outcome);
        
    }

    // events.extend(book.submit(Order {id: 134, side: Side::Buy, price: 100,  quantity: 10, trade_type: order::TradeType::Limit}));
    // events.extend(book.submit(Order {id:123, side: Side::Sell, price:0, quantity: 999, trade_type: order::TradeType::Market}));
    // events.extend(book.submit(Order {id: 223, side: Side::Sell, price: 99, quantity: 6, trade_type: order::TradeType::Limit}));
    
    // for event in events {
    //     println!("{}", event.to_log_line());
    // }
    println!("{:?}", book.cancel_trade(134));
    println!("{:?}", book.get_outstanding(Side::Buy));
    println!("{:?}", book.get_outstanding(Side::Sell));
}