use crate::intersection::{IntersectionConfig, PathId, path_length, position_at_progress, in_intersection_zone};

pub fn generate_conflict_matrix(config: &IntersectionConfig) -> Vec<Vec<PathId>> {
    let mut matrix = vec![vec![]; 12];
    let step = 5.0;
    
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
            
            // Except opposing straights! 
            // In our layout, they don't share lanes, so we can let the distance check handle it.
            // If the distance check finds them > lane_width, they won't conflict.
            
            if conflict {
                matrix[i].push(path_j);
            }
        }
    }
    matrix
}
