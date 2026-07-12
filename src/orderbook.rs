use crate::order::{Order, Side};
use std::collections::BTreeMap;

pub struct OrderBook {
    bids: BTreeMap<u64, Vec<Order>>,
    asks: BTreeMap<u64, Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {bids: BTreeMap::new(), asks: BTreeMap::new()}
    }

    pub fn submit(&mut self, mut order: Order) -> Vec<(u64, u64, u64)> {
        let mut fills = Vec::new();
        let book = match order.side { // Pick the opposite book
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let matching_prices: Vec<u64> = match order.side {
            Side::Buy => book.range(..=order.price).map(|(p, _)| *p).collect(),
            Side::Sell => book.range(..=order.price).map(|(p, _)| * p).collect(),
        };

        for price in matching_prices{
            if order.quantity == 0 {break}
            if let Some(resting) = book.get_mut(&price) {
                resting.retain_mut(|resting_order| {
                    if order.quantity == 0 { return true; }
                    let fill_qty = order.quantity.min(resting_order.quantity);
                    fills.push((order.id, resting_order.id, fill_qty));
                    order.quantity -= fill_qty;
                    resting_order.quantity -= fill_qty;
                    resting_order.quantity > 0
                });
            }
        }

        if order.quantity > 0 {
            let resting_book = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            resting_book.entry(order.price).or_insert_with(Vec::new).push(order);
        }

        fills
    }
}