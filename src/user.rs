use crate::order::{Event, Order};

pub struct User {
    pub id: u32,
    matched_orders: Vec<Order>,
    unmatched_orders: Vec<Order>,
    position: f32
}

impl User {
    pub fn new(id:u32) -> Self {
        User {id, matched_orders:Vec::new(), unmatched_orders: Vec::new(), position: 0.0}
    }

    pub fn record_order(&mut self, order: Order) {
        self.unmatched_orders.push(order)
    }

    pub fn check_event(&mut self, event: Event) {
        if let Event::Trade {buy_order_id, sell_order_id, ..} = event {
            if let Some(index) = self.unmatched_orders.iter().position( | o| {
                o.id == buy_order_id || o.id == sell_order_id
            }) {
                self.match_order(index)
            }
        }
    }

    fn match_order(&mut self, index: usize) {
        let order = self.unmatched_orders.swap_remove(index);
        self.matched_orders.push(order)
    }
}