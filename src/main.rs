mod order;
mod orderbook;

use order::{Order, Side};
use orderbook::OrderBook;

fn main() {
    let mut book = OrderBook::new();

    book.submit(Order {id: 1, side: Side::Sell, price: 100,  quantity: 10});
    let fills = book.submit(Order {id: 2, side: Side::Buy, price: 100, quantity: 6});

    println!("{:?}", fills)
}