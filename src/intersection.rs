//! Cross intersection geometry: directions, routes, paths, and conflict matrix.

use std::f64::consts::FRAC_PI_2;

/// Cardinal direction from which a vehicle approaches the intersection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North, // top
    South, // bottom
    East,  // right
    West,  // left
}

/// Route through the intersection: right, straight, or left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Straight,
    Left,
}

impl Route {
    #[allow(dead_code)]
    pub fn as_char(self) -> char {
        match self {
            Route::Right => 'r',
            Route::Straight => 's',
            Route::Left => 'l',
        }
    }
}

/// Unique path id: 0..=11 (4 directions × 3 routes). Road has 6 lanes per axis: 3 inbound (r/s/l) + 3 outbound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathId(pub u8);

impl PathId {
    pub fn from_direction_route(d: Direction, r: Route) -> Self {
        let dir = match d {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3,
        };
        let route = match r {
            Route::Right => 0,
            Route::Straight => 1,
            Route::Left => 2,
        };
        PathId(dir * 3 + route)
    }

    pub fn direction(self) -> Direction {
        match self.0 / 3 {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        }
    }

    pub fn route(self) -> Route {
        match self.0 % 3 {
            0 => Route::Right,
            1 => Route::Straight,
            _ => Route::Left,
        }
    }
}

/// Intersection layout and path geometry (world units).
pub struct IntersectionConfig {
    pub center_x: f64,
    pub center_y: f64,
    /// Half-width of the intersection box (road width * 2 or so).
    pub half_size: f64,
    /// Distance from spawn to center (approach length).
    pub approach_len: f64,
    /// Lane width (for lateral offset of r/s/l).
    pub lane_width: f64,
    pub conflict_matrix: Vec<Vec<PathId>>,
}

impl Default for IntersectionConfig {
    fn default() -> Self {
        let mut config = Self {
            center_x: 400.0,
            center_y: 300.0,
            // Half of a 6-lane road (3 * lane_width from center line to curb).
            half_size: 120.0,
            approach_len: 350.0,
            lane_width: 40.0,
            conflict_matrix: vec![],
        };
        config.conflict_matrix = generate_conflict_matrix(&config);
        config
    }
}

/// World position (pixels / world units).
#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Vec2) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Path endpoints: start of approach (off-screen arm) and end of exit arm.
fn path_endpoints(config: &IntersectionConfig, path: PathId) -> (Vec2, Vec2) {
    let (dx, dy) = direction_vector(path.direction());
    let lateral = lateral_offset_approach(path.direction(), path.route()) * config.lane_width;
    let (ox, oy) = lateral_offset_xy(path.direction(), lateral);
    // Start is *before* the intersection along the incoming direction (into = +direction).
    let start = Vec2::new(
        config.center_x - dx * config.approach_len + ox,
        config.center_y - dy * config.approach_len + oy,
    );
    let end = exit_position(config, path);
    (start, end)
}

/// Get spawn position for a path (at the start of the approach).
#[allow(dead_code)]
pub fn spawn_position(config: &IntersectionConfig, path: PathId) -> Vec2 {
    path_endpoints(config, path).0
}

/// Unit vector pointing *into* the intersection (from approach).
fn direction_vector(d: Direction) -> (f64, f64) {
    match d {
        Direction::North => (0.0, 1.0),   // from top, moving down
        Direction::South => (0.0, -1.0),  // from bottom, moving up
        Direction::East => (-1.0, 0.0),   // from right, moving left
        Direction::West => (1.0, 0.0),    // from left, moving right
    }
}

/// Lane center offset in [`IntersectionConfig::lane_width`] units: 3 inbound lanes per approach half.
/// E–W road: horizontal median at `cy`. West inbound = half below median (l,s,r toward center); East inbound = half above (r,s,l).
/// West outbound / East outbound use the opposite halves (see `lateral_offset_exit`).
/// North: objectives diagram has r,s,l left→right with r=left turn, l=right turn — map Left to lane index 0, Right to 2.
fn lateral_offset_approach(d: Direction, r: Route) -> f64 {
    let idx = match r {
        Route::Right => 0,
        Route::Straight => 1,
        Route::Left => 2,
    };
    match d {
        Direction::North => {
            // Left lane = left turn, right lane = right turn (matches render labels r←, l→).
            let idx_n = match r {
                Route::Right => 0,
                Route::Straight => 1,
                Route::Left => 2,
            };
            -2.5 + idx_n as f64
        }
        Direction::South => 0.5 + (2 - idx) as f64,
        // Inbound below median (y increasing): l, s, r toward intersection.
        Direction::West => match r {
            Route::Left => 0.5,
            Route::Straight => 1.5,
            Route::Right => 2.5,
        },
        // Inbound above median: r, s, l top → bottom.
        Direction::East => match r {
            Route::Right => -2.5,
            Route::Straight => -1.5,
            Route::Left => -0.5,
        },
    }
}

fn lateral_offset_exit(exit: Direction, r: Route) -> f64 {
    let idx = match r {
        Route::Right => 0,
        Route::Straight => 1,
        Route::Left => 2,
    };
    match exit {
        Direction::North => 0.5 + (2 - idx) as f64,
        Direction::South => -2.5 + idx as f64,
        // Westbound exit: half above median. Heading -x, driver's right is -y (north) -> smaller offset.
        Direction::West => match r {
            Route::Right => -2.5,
            Route::Straight => -1.5,
            Route::Left => -0.5,
        },
        // Eastbound exit: half below median. Heading +x, driver's right is +y (south) -> larger offset.
        Direction::East => match r {
            Route::Right => 2.5,
            Route::Straight => 1.5,
            Route::Left => 0.5,
        },
    }
}



fn lateral_offset_xy(d: Direction, offset: f64) -> (f64, f64) {
    match d {
        Direction::North | Direction::South => (offset, 0.0),
        Direction::East | Direction::West => (0.0, offset),
    }
}

/// Unit direction *along the exit arm* away from the intersection center.
fn outgoing_unit(exit: Direction) -> (f64, f64) {
    match exit {
        Direction::North => (0.0, -1.0),
        Direction::South => (0.0, 1.0),
        Direction::East => (1.0, 0.0),
        Direction::West => (-1.0, 0.0),
    }
}

fn nearly_parallel(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 * b.1 - a.1 * b.0).abs() < 1e-6
}

/// Path as one segment (straight through) or two orthogonal segments (right / left turn).
enum PathPolyline {
    Straight {
        start: Vec2,
        end: Vec2,
    },
    Turn {
        start: Vec2,
        bend: Vec2,
        len1: f64,
        len2: f64,
        d_in: (f64, f64),
        d_out: (f64, f64),
    },
}

impl PathPolyline {
    fn build(config: &IntersectionConfig, path: PathId) -> Self {
        let (start, end) = path_endpoints(config, path);
        let approach = path.direction();
        let route = path.route();

        if route == Route::Straight {
            return PathPolyline::Straight { start, end };
        }

        let d_in = direction_vector(approach);
        let ex = exit_direction(approach, route);
        let d_out = outgoing_unit(ex);

        if nearly_parallel(d_in, d_out) {
            return PathPolyline::Straight { start, end };
        }

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let det = d_in.0 * d_out.1 - d_in.1 * d_out.0;
        if det.abs() < 1e-9 {
            return PathPolyline::Straight { start, end };
        }
        let t1 = (dx * d_out.1 - dy * d_out.0) / det;
        let t2 = (d_in.0 * dy - d_in.1 * dx) / det;

        if t1 < 0.0 || t2 < 0.0 {
            return PathPolyline::Straight { start, end };
        }

        let bend = Vec2::new(start.x + t1 * d_in.0, start.y + t1 * d_in.1);
        PathPolyline::Turn {
            start,
            bend,
            len1: t1,
            len2: t2,
            d_in,
            d_out,
        }
    }

    fn length(&self) -> f64 {
        match self {
            PathPolyline::Straight { start, end } => start.distance_to(*end),
            PathPolyline::Turn { len1, len2, .. } => len1 + len2,
        }
    }

    fn position_at(&self, progress: f64) -> Vec2 {
        match self {
            PathPolyline::Straight { start, end } => {
                let pl = start.distance_to(*end).max(1e-6);
                let t = (progress / pl).clamp(0.0, 1.0);
                Vec2::new(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                )
            }
            PathPolyline::Turn {
                start,
                bend,
                len1,
                d_in,
                d_out,
                ..
            } => {
                if progress <= *len1 {
                    Vec2::new(
                        start.x + progress * d_in.0,
                        start.y + progress * d_in.1,
                    )
                } else {
                    let q = progress - len1;
                    Vec2::new(bend.x + q * d_out.0, bend.y + q * d_out.1)
                }
            }
        }
    }
}

/// True if world position is inside the intersection box (same footprint as drawn center).
pub fn in_intersection_zone(config: &IntersectionConfig, pos: Vec2) -> bool {
    let h = config.half_size;
    (pos.x - config.center_x).abs() <= h && (pos.y - config.center_y).abs() <= h
}

/// Expanded box: vehicles “approaching” the junction (for conflict lookahead).
pub fn near_intersection_zone(config: &IntersectionConfig, pos: Vec2, extra: f64) -> bool {
    let h = config.half_size + extra;
    (pos.x - config.center_x).abs() <= h && (pos.y - config.center_y).abs() <= h
}

/// Exit direction after taking a route from an approach direction.
/// Right/left turns follow right-hand traffic: e.g. from South (heading north) right=E, left=W;
/// from East (heading west) right=N, left=S; from West (heading east) right=S, left=N.
fn exit_direction(approach: Direction, route: Route) -> Direction {
    use Direction::{East, North, South, West};
    use Route::{Left, Right, Straight};
    match (approach, route) {
        (North, Right) => West,
        (North, Straight) => South,
        (North, Left) => East,
        (South, Right) => East,
        (South, Straight) => North,
        (South, Left) => West,
        (East, Right) => North,
        (East, Straight) => West,
        (East, Left) => South,
        (West, Right) => South,
        (West, Straight) => East,
        (West, Left) => North,
    }
}

/// Angle in radians for rendering (0 = right/east, increasing counter-clockwise).
pub fn path_angle_at_progress(config: &IntersectionConfig, path: PathId, progress: f64) -> f64 {
    let d = path.direction();
    let r = path.route();
    let mut entry = entry_angle(d);
    let mut exit = exit_angle(exit_direction(d, r));

    let poly = PathPolyline::build(config, path);
    match poly {
        PathPolyline::Straight { .. } => normalize_angle(entry),
        PathPolyline::Turn { len1, .. } => {
            let pi = std::f64::consts::PI;
            if exit - entry > pi {
                entry += 2.0 * pi;
            } else if entry - exit > pi {
                exit += 2.0 * pi;
            }

            let turn_radius = config.lane_width * 1.5;
            let turn_start = (len1 - turn_radius).max(0.0);
            let turn_end = len1 + turn_radius;

            if progress <= turn_start {
                normalize_angle(entry)
            } else if progress >= turn_end {
                normalize_angle(exit)
            } else {
                let t = (progress - turn_start) / (turn_end - turn_start);
                let angle = entry + (exit - entry) * t;
                normalize_angle(angle)
            }
        }
    }
}

#[allow(dead_code)]
fn entry_angle(d: Direction) -> f64 {
    match d {
        Direction::North => FRAC_PI_2,           // down
        Direction::South => -FRAC_PI_2,          // up
        Direction::East => std::f64::consts::PI, // left
        Direction::West => 0.0,                  // right
    }
}

#[allow(dead_code)]
fn exit_angle(d: Direction) -> f64 {
    match d {
        Direction::North => -FRAC_PI_2,
        Direction::South => FRAC_PI_2,
        Direction::East => 0.0,
        Direction::West => std::f64::consts::PI,
    }
}

#[allow(dead_code)]
fn normalize_angle(a: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let mut a = a;
    while a > pi {
        a -= 2.0 * pi;
    }
    while a < -pi {
        a += 2.0 * pi;
    }
    a
}

/// World path length (pixels) along approach + bend + exit (speed = px/s along polyline).
pub fn path_length(config: &IntersectionConfig, path: PathId) -> f64 {
    PathPolyline::build(config, path).length()
}

/// Position along path from progress (0..=path_length).
pub fn position_at_progress(config: &IntersectionConfig, path: PathId, progress: f64) -> Vec2 {
    let poly = PathPolyline::build(config, path);
    let pl = poly.length().max(1e-6);
    poly.position_at(progress.min(pl))
}

fn exit_position(config: &IntersectionConfig, path: PathId) -> Vec2 {
    let d = path.direction();
    let r = path.route();
    let exit = exit_direction(d, r);
    let (dx, dy) = match exit {
        Direction::North => (0.0, -1.0),
        Direction::South => (0.0, 1.0),
        Direction::East => (1.0, 0.0),
        Direction::West => (-1.0, 0.0),
    };
    let lateral = lateral_offset_exit(exit, r) * config.lane_width;
    let (ox, oy) = lateral_offset_xy(exit, lateral);
    Vec2::new(
        config.center_x + dx * config.approach_len + ox,
        config.center_y + dy * config.approach_len + oy,
    )
}

/// Precomputed conflict set: paths that physically cross in the intersection.
///
/// **Controller note:** vehicles on [`Route::Right`] do not *apply* this list to stop themselves
/// (they proceed without cross-traffic yield), but **other** routes still list right-turn paths
/// so straight/left traffic yields when an `r` vehicle occupies the junction—otherwise overlaps occur.
///
/// **Opposing** straight-through paths (N↔S, E↔W) are **not** paired: they use separate lanes across
/// the median and do not cross each other in this layout.
pub fn conflicting_paths(config: &IntersectionConfig, path: PathId) -> &[PathId] {
    if (path.0 as usize) < config.conflict_matrix.len() {
        &config.conflict_matrix[path.0 as usize]
    } else {
        &[]
    }
}

fn generate_conflict_matrix(config: &IntersectionConfig) -> Vec<Vec<PathId>> {
    let mut matrix = vec![vec![]; 12];
    let step = 4.0;
    
    for i in 0..12 {
        let path_i = PathId(i as u8);
        let len_i = path_length(config, path_i);
        let mut pts_i = Vec::new();
        let mut p = 0.0;
        while p <= len_i {
            let pos = position_at_progress(config, path_i, p);
            if in_intersection_zone(config, pos) {
                pts_i.push(pos);
            }
            p += step;
        }
        
        for j in 0..12 {
            if i == j { continue; }
            let path_j = PathId(j as u8);
            
            // Opposing straight check: N(1) vs S(4), E(7) vs W(10)
            if (i == 1 && j == 4) || (i == 4 && j == 1) { continue; }
            if (i == 7 && j == 10) || (i == 10 && j == 7) { continue; }

            let len_j = path_length(config, path_j);
            let mut conflict = false;
            let mut p2 = 0.0;
            while p2 <= len_j {
                let pos = position_at_progress(config, path_j, p2);
                if in_intersection_zone(config, pos) {
                    for pt in &pts_i {
                        if pt.distance_to(pos) < config.lane_width * 0.85 {
                            conflict = true;
                            break;
                        }
                    }
                }
                if conflict { break; }
                p2 += step;
            }
            
            if conflict {
                matrix[i].push(path_j);
            }
        }
    }
    matrix
}

pub fn is_threat(config: &IntersectionConfig, path_i: PathId, prog_i: f64, path_j: PathId, prog_j: f64) -> bool {
    let len_i = path_length(config, path_i);
    let len_j = path_length(config, path_j);
    let step = 8.0;

    let rear_j = prog_j - 64.0; // Vehicle length

    let mut p1 = prog_i;
    while p1 <= len_i {
        let pos_i = position_at_progress(config, path_i, p1);
        if in_intersection_zone(config, pos_i) {
            
            // Find the closest point on j's path to this point on i's path
            let mut min_dist = f64::MAX;
            let mut min_p2 = 0.0;
            
            let mut p2 = 0.0;
            while p2 <= len_j {
                let pos_j = position_at_progress(config, path_j, p2);
                let d = pos_i.distance_to(pos_j);
                if d < min_dist {
                    min_dist = d;
                    min_p2 = p2;
                }
                p2 += step;
            }
            
            // If they intersect here, check if j has cleared this intersection point
            if min_dist < config.lane_width * 0.85 {
                // rear_j is the back of the car. We add 20.0 as a safety margin.
                if rear_j < min_p2 + 20.0 {
                    return true;
                }
            }
        }
        p1 += step;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_id_roundtrip() {
        for d in [Direction::North, Direction::South, Direction::East, Direction::West] {
            for r in [Route::Right, Route::Straight, Route::Left] {
                let id = PathId::from_direction_route(d, r);
                assert_eq!(id.direction(), d);
                assert_eq!(id.route(), r);
            }
        }
    }



    #[test]
    fn opposing_straight_do_not_conflict() {
        let config = IntersectionConfig::default();
        let n_s = PathId::from_direction_route(Direction::North, Route::Straight);
        let s_s = PathId::from_direction_route(Direction::South, Route::Straight);
        assert!(!conflicting_paths(&config, n_s).contains(&s_s));
        assert!(!conflicting_paths(&config, s_s).contains(&n_s));
        let e_s = PathId::from_direction_route(Direction::East, Route::Straight);
        let w_s = PathId::from_direction_route(Direction::West, Route::Straight);
        assert!(!conflicting_paths(&config, e_s).contains(&w_s));
        assert!(!conflicting_paths(&config, w_s).contains(&e_s));
    }

    #[test]
    fn print_matrix() {
        let config = IntersectionConfig::default();
        let matrix = super::generate_conflict_matrix(&config);
        for (i, row) in matrix.iter().enumerate() {
            println!("{:2} conflicts with: {:?}", i, row.iter().map(|p| p.0).collect::<Vec<_>>());
        }
    }
}
