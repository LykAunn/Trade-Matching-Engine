#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Accepted {
        order_id: u64,
        timestamp: u64,
    },

    Trade {
        buy_order_id: u64,
        sell_order_id: u64,
        price: u64,
        qty: u64,
        timestamp: u64,
    },

    Rested {
        order_id: u64,
        timestamp: u64,
    },

    Cancelled {
        order_id: u64,
        timestamp: u64,
    },

    Rejected {
        order_id: u64,
        reason: String,
        timestamp: u64,
    },

    RestingFulfilled {
        order_id: u64,
        timestamp: u64,
    }

}