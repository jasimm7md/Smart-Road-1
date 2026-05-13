//! Smart intersection controller: sets vehicle velocities to avoid collisions and respect safe distance.

use crate::intersection::{
    conflicting_paths, in_intersection_zone, near_intersection_zone, IntersectionConfig, Route,
};
use crate::stats::StatsCollector;
use crate::vehicle::{Vehicle, VehiclePhysicsConfig, VelocityLevel};

/// Progress is arc length along the path (world units). Tie-break with vehicle id when equal
/// (e.g. all spawn at progress 0): lower id is treated as ahead so platoon order is stable.
const PROGRESS_EPS: f64 = 1e-4;

#[inline]
fn is_ahead_on_same_path(progress_j: f64, id_j: u32, progress_i: f64, id_i: u32) -> bool {
    if progress_j > progress_i + PROGRESS_EPS {
        return true;
    }
    if (progress_j - progress_i).abs() <= PROGRESS_EPS && id_j < id_i {
        return true;
    }
    false
}

pub fn update_velocities(
    vehicles: &mut [Vehicle],
    physics: &VehiclePhysicsConfig,
    inter: &IntersectionConfig,
    stats: &mut StatsCollector,
) {
    let positions: Vec<_> = vehicles
        .iter()
        .map(|v| v.position(inter))
        .collect();
    // Padding so we start yielding exactly at the stop line (car length is 64, so center stops 35px before junction).
    let approach_pad = 35.0;

    for i in 0..vehicles.len() {
        if vehicles[i].exited {
            continue;
        }
        let path = vehicles[i].path;
        let pos_i = positions[i];
        // After crossing the intersection box (entered then left), keep moving on the exit arm:
        // do not apply conflicting-path yields to traffic still in the junction.
        let past_intersection_box =
            vehicles[i].detected && !in_intersection_zone(inter, pos_i);

        let mut level = VelocityLevel::Normal;

        // 1) Same path: vehicle ahead within safe distance?
        let progress_i = vehicles[i].progress;
        let id_i = vehicles[i].id;
        let mut min_progress_ahead = f64::INFINITY;
        for j in 0..vehicles.len() {
            if i == j || vehicles[j].exited {
                continue;
            }
            if vehicles[j].path.0 != path.0 {
                continue;
            }
            let progress_j = vehicles[j].progress;
            if !is_ahead_on_same_path(progress_j, vehicles[j].id, progress_i, id_i) {
                continue;
            }
            if progress_j < min_progress_ahead {
                min_progress_ahead = progress_j;
            }
        }
        if min_progress_ahead.is_finite() {
            let gap = min_progress_ahead - progress_i;
            if gap < physics.safe_distance {
                level = VelocityLevel::Stopped;
            } else if gap < physics.safe_distance * 1.5 {
                level = VelocityLevel::Slow;
            }
        }

        // 2) Conflicting path: another vehicle occupies the intersection box?
        if level != VelocityLevel::Stopped
            && near_intersection_zone(inter, pos_i, approach_pad)
            && !past_intersection_box
        {
            for &conf in conflicting_paths(inter, path) {
                for j in 0..vehicles.len() {
                    if i == j || vehicles[j].exited {
                        continue;
                    }
                    if vehicles[j].path.0 != conf.0 {
                        continue;
                    }
                    let j_inside = in_intersection_zone(inter, positions[j]);
                    let j_near = near_intersection_zone(inter, positions[j], approach_pad);

                    if j_near {
                        // Check if vehicle j actually intersects our remaining path with a buffer
                        if !crate::intersection::is_threat(inter, path, vehicles[i].progress, vehicles[j].path, vehicles[j].progress) {
                            continue;
                        }

                        let dist = pos_i.distance_to(positions[j]);
                        if dist < physics.safe_distance {
                            stats.record_close_call();
                        }

                        let i_inside = in_intersection_zone(inter, pos_i);
                        
                        if j_inside {
                            if i_inside {
                                // Both inside: lower ID has priority
                                if vehicles[i].id > vehicles[j].id {
                                    level = VelocityLevel::Stopped;
                                }
                            } else {
                                // i is outside, j is inside: i MUST yield
                                level = VelocityLevel::Stopped;
                            }
                        } else {
                            // j is near but NOT inside.
                            if i_inside {
                                // i is inside, j is outside: i has priority (no yield)
                            } else {
                                // Both are outside (but near). Resolve right-of-way.
                                let d_i = pos_i.distance_to(crate::intersection::Vec2::new(inter.center_x, inter.center_y));
                                let d_j = positions[j].distance_to(crate::intersection::Vec2::new(inter.center_x, inter.center_y));
                                
                                if d_j < d_i - 2.0 {
                                    // j is closer to center, i yields
                                    level = VelocityLevel::Stopped;
                                } else if (d_j - d_i).abs() <= 2.0 && vehicles[i].id > vehicles[j].id {
                                    // tie, lower ID goes
                                    level = VelocityLevel::Stopped;
                                }
                            }
                        }
                    }
                }
                if level == VelocityLevel::Stopped {
                    break;
                }
            }
        }

        // 3) Do not keep vehicles at 0 speed inside the physical intersection box (policy).
        // Rules may still request a stop before entry; once inside the junction, keep moving.
        if in_intersection_zone(inter, pos_i) && level == VelocityLevel::Stopped {
            level = VelocityLevel::Slow;
        }

        vehicles[i].velocity_level = level;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_tie_progress_lower_id_is_ahead() {
        assert!(is_ahead_on_same_path(0.0, 1, 0.0, 2));
        assert!(!is_ahead_on_same_path(0.0, 2, 0.0, 1));
    }

    #[test]
    fn same_path_higher_progress_is_ahead() {
        assert!(is_ahead_on_same_path(10.0, 99, 5.0, 1));
        assert!(!is_ahead_on_same_path(5.0, 1, 10.0, 0));
    }
}
