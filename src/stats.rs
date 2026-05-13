//! Statistics: max vehicles passed, max/min velocity, max/min time, close calls.

use std::f64;

pub struct StatsCollector {
    pub vehicles_passed: u32,
    pub max_velocity: f64,
    pub min_velocity: f64,
    pub max_time_through_intersection: f64,
    pub min_time_through_intersection: f64,
    pub close_calls: u32,
    /// For min we need the first valid value.
    min_velocity_set: bool,
    min_time_set: bool,
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self {
            vehicles_passed: 0,
            max_velocity: 0.0,
            min_velocity: f64::MAX,
            max_time_through_intersection: 0.0,
            min_time_through_intersection: f64::MAX,
            close_calls: 0,
            min_velocity_set: false,
            min_time_set: false,
        }
    }
}

impl StatsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_vehicle_exit(&mut self, velocity: f64, time_through_intersection: f64) {
        self.vehicles_passed += 1;
        if velocity > self.max_velocity {
            self.max_velocity = velocity;
        }
        if !self.min_velocity_set || velocity < self.min_velocity {
            self.min_velocity = velocity;
            self.min_velocity_set = true;
        }
        if time_through_intersection > self.max_time_through_intersection {
            self.max_time_through_intersection = time_through_intersection;
        }
        if !self.min_time_set || time_through_intersection < self.min_time_through_intersection {
            self.min_time_through_intersection = time_through_intersection;
            self.min_time_set = true;
        }
    }

    pub fn record_close_call(&mut self) {
        self.close_calls += 1;
    }

    /// For display: min velocity might never have been set.
    pub fn min_velocity_display(&self) -> f64 {
        if self.min_velocity_set {
            self.min_velocity
        } else {
            0.0
        }
    }

    pub fn min_time_display(&self) -> f64 {
        if self.min_time_set {
            self.min_time_through_intersection
        } else {
            0.0
        }
    }
}
