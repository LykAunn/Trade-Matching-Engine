use crate::order::{Event, Order, Side};
use std::{cmp::min, collections::{BTreeMap, HashMap}, time::UNIX_EPOCH};
use std::time::{SystemTime};

pub struct OrderBook {
    bids: BTreeMap<u64, Vec<Order>>, // Buy
    asks: BTreeMap<u64, Vec<Order>>, // Sell
    order_index: HashMap<u64, (Side, u64)> // order_id -> (side, price)
}

pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos() as u64
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {bids: BTreeMap::new(), asks: BTreeMap::new(), order_index: HashMap::new()}
    }


    pub fn submit(&mut self, mut order: Order) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        let time = now_ts();

        events.push(Event::Accepted { order_id: order.id, timestamp: time});

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

                    // Note down fill to events
                    events.push(Event:: Trade { buy_order_id: order.id, sell_order_id: resting_order.id, price: resting_order.price,
                        qty: resting_order.quantity, timestamp: time});

                    order.quantity -= fill_qty;
                    resting_order.quantity -= fill_qty;

                    if resting_order.quantity == 0 {
                        let id: u64 = resting_order.id;
                        resting.remove(i);
                        self.order_index.remove(&id);

                        // If resting order fulfilled, note down in events
                        events.push(Event::RestingFulfilled { order_id: id, timestamp: time });
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
            events.push(Event::Rested { order_id: order.id, timestamp: time });
            resting_book.entry(order.price).or_insert_with(Vec::new).push(order);

        }

        events
    }

    pub fn get_outstanding(&self, side: Side) -> Vec<Order> {
        let book = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks
        };

        book.values().flatten().cloned().collect()
    }

    pub fn cancel_trade(&mut self, id: u64) -> Vec<Event> {
        let mut event: Vec<Event> = Vec::new();
        let time = now_ts();
        event.push(Event::Accepted { order_id: id, timestamp: time });

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

                        event.push(Event::Cancelled { order_id: id, timestamp: time });
                        return event
                    }
                }
            }
            event.push(Event::Rejected { order_id: id, reason: String::from("Matching trade not found"), timestamp: time });
            event
        } else {
            // not found in order_index
            event.push(Event::Rejected { order_id: id, reason: String::from("Order not found"), timestamp: time });
            event
        }
    }
}