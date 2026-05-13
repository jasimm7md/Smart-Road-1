//! Autonomous vehicle: physics (velocity, distance, time) and movement along path.

use crate::intersection::{position_at_progress, path_angle_at_progress, IntersectionConfig, PathId, Vec2};
use crate::stats::StatsCollector;

/// At least 3 velocities as per spec. We use 3 levels: stopped, slow, normal, (optional fast).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VelocityLevel {
    Stopped = 0,
    Slow = 1,
    Normal = 2,
}

impl VelocityLevel {
    pub fn to_speed(self, config: &VehiclePhysicsConfig) -> f64 {
        match self {
            VelocityLevel::Stopped => 0.0,
            VelocityLevel::Slow => config.speed_slow,
            VelocityLevel::Normal => config.speed_normal,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VehiclePhysicsConfig {
    pub speed_slow: f64,
    pub speed_normal: f64,
    pub safe_distance: f64,
}

impl Default for VehiclePhysicsConfig {
    fn default() -> Self {
        Self {
            speed_slow: 80.0,
            speed_normal: 200.0,
            safe_distance: 90.0,
        }
    }
}

/// One autonomous vehicle.
#[derive(Clone, Debug)]
pub struct Vehicle {
    #[allow(dead_code)]
    pub id: u32,
    pub path: PathId,
    /// Current progress along path (0..=path_length).
    pub progress: f64,
    pub path_length: f64,
    /// Current velocity level (controller sets this).
    pub velocity_level: VelocityLevel,
    /// Time in seconds since spawn.
    pub time_total: f64,
    /// Time inside intersection (from detection until exit). Set when crossing entry threshold.
    pub time_in_intersection: f64,
    /// Whether we have crossed the "detection" (entry) threshold (for stats).
    pub detected: bool,
    /// Whether we have fully exited (to be removed).
    pub exited: bool,
}

impl Vehicle {
    pub fn new(id: u32, path: PathId, path_length: f64) -> Self {
        Self {
            id,
            path,
            progress: 0.0,
            path_length,
            velocity_level: VelocityLevel::Normal,
            time_total: 0.0,
            time_in_intersection: 0.0,
            detected: false,
            exited: false,
        }
    }

    /// Current speed in world units per second.
    pub fn speed(&self, physics: &VehiclePhysicsConfig) -> f64 {
        self.velocity_level.to_speed(physics)
    }

    /// Velocity = distance / time: here we use progress (distance along path) and time.
    pub fn velocity_magnitude(&self, physics: &VehiclePhysicsConfig) -> f64 {
        self.speed(physics)
    }

    pub fn position(&self, inter: &IntersectionConfig) -> Vec2 {
        position_at_progress(inter, self.path, self.progress)
    }

    #[allow(dead_code)]
    pub fn angle_rad(&self, config: &IntersectionConfig) -> f64 {
        path_angle_at_progress(config, self.path, self.progress)
    }

    /// Is this vehicle inside the intersection zone (between entry and exit progress)?
    #[allow(dead_code)]
    pub fn in_intersection(&self, entry: f64, exit: f64) -> bool {
        self.progress >= entry && self.progress < exit
    }

    /// Advance by dt. Returns true if vehicle just exited (for stats and removal).
    pub fn update(
        &mut self,
        dt: f64,
        physics: &VehiclePhysicsConfig,
        inter: &IntersectionConfig,
        stats: &mut StatsCollector,
    ) -> bool {
        let speed = self.speed(physics);
        let pos = self.position(inter);

        if !self.detected && crate::intersection::in_intersection_zone(inter, pos) {
            self.detected = true;
        }
        if self.detected {
            self.time_in_intersection += dt;
        }
        self.time_total += dt;

        self.progress += speed * dt;
        if self.progress >= self.path_length {
            self.progress = self.path_length;
            self.exited = true;
            stats.record_vehicle_exit(
                self.velocity_magnitude(physics),
                self.time_in_intersection,
            );
            return true;
        }
        false
    }
}
