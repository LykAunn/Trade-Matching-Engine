use crate::order::Event;

pub struct Stats {
    pub total_volume: u64,
    pub trade_count: u64,
    pub total_notional: u64,
}

impl Stats {
    pub fn new() -> Self {
        Stats { total_volume: 0, trade_count: 0, total_notional: 0}
    }

    pub fn record_event(&mut self, event: &Event) {
        if let Event::Trade {price, qty, ..} = event {
            self.total_volume += qty;
            self.trade_count += 1;
            self.total_notional += (price * qty);
        }
    }

    pub fn vwap(&self) -> Option<f64> {
        if self.total_volume ==0 {
            None
        } else {
            Some(self.total_notional as f64 / self.total_volume as f64)
        }
    }
}