mod order;
mod orderbook;

use order::{Order, Side};
use orderbook::OrderBook;

fn main() {
    let mut book = OrderBook::new();
    let mut events = Vec::new();

    events.extend(book.submit(Order {id: 134, side: Side::Buy, price: 100,  quantity: 10}));
    events.extend(book.submit(Order {id:123, side: Side::Buy,price:101, quantity: 999}));
    events.extend(book.submit(Order {id: 223, side: Side::Sell, price: 99, quantity: 6}));
    
    // println!("{:?}", event);
    for event in events {
        println!("{}", event.to_log_line());
    }
    println!("{:?}", book.get_outstanding(Side::Buy));
    println!("{:?}", book.cancel_trade(134));
    println!("{:?}", book.get_outstanding(Side::Buy));
}