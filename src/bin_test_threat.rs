use smart_road::intersection::*;
use smart_road::vehicle::*;

fn main() {
    let config = IntersectionConfig::default();
    let p_south_straight = PathId::from_direction_route(Direction::South, Route::Straight);
    let p_west_straight = PathId::from_direction_route(Direction::West, Route::Straight);
    
    // West straight crosses from left to right.
    // Center is 400, 300. West Straight is at y = 300 + 1.5 * 40 = 360.
    // It crosses x = 400.
    
    // South straight crosses from bottom to top.
    // It is at x = 400 + 1.5 * 40 = 460.
    // It crosses y = 300.
    
    // Collision point is x=460, y=360.
    
    let w_len = path_length(&config, p_west_straight);
    let s_len = path_length(&config, p_south_straight);
    
    // Find the progress where West Straight is at x=460.
    // West Straight starts at x = 400 - 350 = 50.
    // x = 460 means progress is 410.
    
    // Find the progress where South Straight is at y=360.
    // South Straight starts at y = 300 + 350 = 650.
    // y = 360 means progress is 650 - 360 = 290.
    
    // Let's test is_threat.
    // South is at stop line. South starts at y=650. Stop line is y=420.
    // So South progress is 650 - 420 = 230.
    
    // West is crossing. West progress is 410 (exactly at collision point).
    let threat1 = is_threat(&config, p_south_straight, 230.0, p_west_straight, 410.0);
    println!("Threat when exactly at collision point: {}", threat1);
    
    // West progress is 450 (rear bumper is at 450 - 64 = 386. Collision is at 410. Hasn't cleared).
    let threat2 = is_threat(&config, p_south_straight, 230.0, p_west_straight, 450.0);
    println!("Threat when rear bumper is at 386 (collision 410): {}", threat2);
    
    // West progress is 490 (rear bumper is at 490 - 64 = 426. Collision is at 410. Cleared by 16).
    let threat3 = is_threat(&config, p_south_straight, 230.0, p_west_straight, 490.0);
    println!("Threat when rear bumper is at 426 (collision 410): {}", threat3);
    
    // West progress is 500 (rear bumper is at 436. Collision is at 410. Cleared by 26).
    let threat4 = is_threat(&config, p_south_straight, 230.0, p_west_straight, 500.0);
    println!("Threat when rear bumper is at 436 (collision 410): {}", threat4);
}
