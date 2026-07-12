use crate::order::{Order, Side};
use std::{cmp::min, collections::{BTreeMap, HashMap}};

pub struct OrderBook {
    bids: BTreeMap<u64, Vec<Order>>, // Buy
    asks: BTreeMap<u64, Vec<Order>>, // Sell
    order_index: HashMap<u64, (Side, u64)> // order_id -> (side, price)
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {bids: BTreeMap::new(), asks: BTreeMap::new(), order_index: HashMap::new()}
    }

    pub fn submit(&mut self, mut order: Order) -> Vec<(u64, u64, u64)> {
        let mut fills = Vec::new();
        let book = match order.side { // Pick the opposite book
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let matching_prices: Vec<u64> = match order.side {
            Side::Buy => book.range(..=order.price).map(|(p, _)| * p).collect(),
            Side::Sell => book.range(order.price..).rev().map(|(p, _)| * p).collect(),
        };

        for price in matching_prices {
            if order.quantity == 0 {break}
            if let Some(resting) = book.get_mut(&price) {
                let mut i = 0;
                while i < resting.len() {
                    if order.quantity == 0 {break} // If there is no more to be filled, end

                    let resting_order = &mut resting[i]; // Iterates through all resting orders of that price
                    let fill_qty = min(order.quantity, resting_order.quantity);
                    fills.push((order.id, resting_order.id, fill_qty));
                    order.quantity -= fill_qty;
                    resting_order.quantity -= fill_qty;

                    if resting_order.quantity == 0 {
                        let id: u64 = resting_order.id;
                        resting.remove(i);
                        self.order_index.remove(&id);
                    } else {
                        i += 1;
                    }
                }
            }
        }

        if order.quantity > 0 {
            let resting_book = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };


            self.order_index.insert(order.id, (order.side, order.price));
            resting_book.entry(order.price).or_insert_with(Vec::new).push(order);
        }

        fills
    }

    pub fn get_outstanding(&self, side: Side) -> Vec<Order> {
        let book = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks
        };

        book.values().flatten().cloned().collect()
    }

    pub fn cancel_trade(&mut self, id: u64) -> bool {
        if let Some(value) = self.order_index.get(&id) {
            let order_side = value.0;
            let order_price = value.1;

            let book = match order_side {
                Side::Buy => &mut self.bids,
                Side:: Sell => &mut self.asks,
            };

            if let Some(matching) = book.get_mut(&order_price) {
                for i in 0..matching.len() {
                    if matching[i].id == id {
                        matching.remove(i);
                        self.order_index.remove(&id);
                        return true;
                    }
                }
            }
            false
        } else {
            // not found in order_index
            false
        }
    }
}