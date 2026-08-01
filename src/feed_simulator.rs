use rand::{RngExt, rngs::ThreadRng};
use rand_distr::{Exp, Distribution, Normal};
use crate::order::{Order, Side, TradeType};

pub struct FeedSimulator {
    mid_price: f64,       // Current simulated mid price
    rng: ThreadRng,       
    next_id: u64,         
    rate_per_second: f64, // orders per second
    volatility: f64,      // size of random walk step
    sim_time: f64,        // total simulated time
    exp: Exp<f64>,
    normal: Normal<f64>
}

impl FeedSimulator {
    pub fn new(rate_per_second: f64, volatility: f64) -> Self {
        FeedSimulator {
            mid_price: 100.0,
            rng: rand::rng(),
            next_id: 1,
            rate_per_second,
            volatility,
            sim_time: 0.0,
            exp: Exp::new(rate_per_second).unwrap(),
            normal : Normal::new(0.0, volatility).unwrap(),
        }
    }

    pub fn generate_next(&mut self) -> (Order, f64) { // Return amount of time to sleep

        // Time
        let gap = self.exp.sample(&mut self.rng);
        self.sim_time += gap;

        // Price
        let delta = self.normal.sample(&mut self.rng);
        self.mid_price += delta;

        // Side
        let side = if self.rng.random_bool(0.5) {Side::Buy} else {Side::Sell};

        // Buy/sell price
        let price = match side{
            Side::Buy => (self.mid_price - self.rng.random_range(2..5) as f64).round() as u64,
            Side::Sell => (self.mid_price + self.rng.random_range(2..5) as f64).round() as u64,
        };

        let quantity = self.rng.random_range(1..=100);

        let id = self.next_id;
        self.next_id += 1;

        let order_type = match self.rng.random_range(1..100) {
            0..=79 => TradeType::Limit,
            80..=94 => TradeType::Market,
            _ => TradeType::Ioc,
        };

        (Order {
            id,
            side,
            price,
            quantity,
            trade_type: order_type
        },
        gap)
    }
}