# Smart Road – Next steps and progress

This file tracks steps and optional improvements relative to the mandatory objectives.

## Mandatory requirements (objectives/README.md)

- [x] Cross intersection with lanes r, s, l
- [x] AVs with at least 3 velocities; controller sets velocity
- [x] Safe distance (strictly positive); no collisions
- [x] Physics: velocity, distance, time per vehicle; time from detection to exit
- [x] Animation: vehicles move and are drawn (simple shapes)
- [x] Arrow keys: spawn from S/N/E/W; R for continuous random spawn
- [x] Esc: end simulation and show statistics window
- [x] No spawn spam: cooldown per direction
- [x] Statistics: max vehicles passed, max/min velocity, max/min time through intersection, close calls
- [ ] Animation with assets: Use sprite assets for vehicles and intersection and animate rotation on turns.

## Done

1. Project created (Rust + SDL2, **`bundled`** so SDL2 builds with the project).
1b. **Six lanes per road (objectives ASCII)**: three **inbound** lanes (r, s, l) and three **outbound** (unlabeled); single yellow dashed median at the road center (cx / cy); `PathId` remains 4×3 routes; solid white dividers only between lanes, not on the median.
1c. **Conflict deadlock fix**: When two vehicles on conflicting paths were both inside `in_intersection_zone`, the controller set both to Stopped forever. Now approaching vehicles yield to traffic already in the junction; if both are inside, lower vehicle `id` has priority so one can clear.
1d. **Exit arm**: Vehicles that have already left the intersection box (`detected` && `!in_intersection_zone`) no longer get conflicting-path stops so they keep moving on the outbound leg (same-path safe distance still applies).
1e. **Conflict matrix**: East/West same-arm **right vs left** (paths 6↔8, 9↔11) were incorrectly mutual conflicts; those turns diverge (N vs S) and do not cross, so those pairs were removed from `conflict_list`.
1f. **Right-turn routes**: controller skips conflict rule (2) for `Route::Right` so `r` lanes do not wait for cross traffic. The **full** `conflict_list` still includes `r` path ids where paths cross so **straight/left** vehicles yield when an `r` vehicle is in the junction (removing `r` from the matrix caused overlaps under heavy random spawn).
1g. **Opposing straight** (N↔S `s`, E↔W `s`): removed mutual conflicts—two straight vehicles from opposite directions use separate lanes (median); they should not yield to each other (fixes unnecessary waits and reduces bogus stacking).
1h. **Right-turn same-lane spacing**: controller also skips rule (1) (same-path safe distance) for `Route::Right`, so `r` lanes do not stack-stop behind each other at the approach while other movements are active (matches “no conflict” treatment for `r`).
1i. **Same-path following**: spacing uses **arc length** (`progress` gap to the nearest ahead vehicle), not Euclidean distance; **tie-break** when `progress` is equal (e.g. spawn) — lower `id` is ahead so the leader is never ambiguous.
1j. **Intersection policy**: vehicles inside `in_intersection_zone` are never held at **Stopped** (0 speed) — clamp to **Slow** if rules would stop. **`Route::Right`** is forced to **Normal** after all rules so `r` lanes never show as stopped at the line or in the box.
2. Intersection geometry and path conflict matrix.
3. Vehicle physics and movement along paths.
4. Smart controller: same-lane safe distance + conflicting-path stop.
5. Main loop, keyboard input, spawn cooldown, R-held random spawn.
6. Simple rendering (intersection + vehicles).
7. Stats collection and stats window on Esc (**bitmap text** via `font8x8`, no SDL2_ttf).
8. **`.cargo/config.toml`**: `CMAKE_POLICY_VERSION_MINIMUM=3.5` for CMake 4+ when building bundled SDL2.
9. **Route l/r vs turns**: Fixed `exit_direction` for South/East/West (left/right turns were swapped vs right-hand traffic). Fixed North inbound `lateral_offset_approach` so lane labels r/s/l match left vs right turn paths (objectives diagram).
10. **E/W exit lanes**: Swapped `lateral_offset_exit` for East and West so right-turn / left-turn routes use the correct outbound lane (e.g. South `r` → East stays in the right lane).
11. **North r/l exit alignment**: `lateral_offset_exit_for_path` — North `r` (left) exits westbound lane 0 (north); North `l` (right) exits eastbound **left** lane (north / inner), not the right lane.

## Optional next steps (bonus / polish)

- [ ] Add more statistics (e.g. average time, throughput).
- [ ] Add acceleration/deceleration (smooth speed changes).
- [ ] Tune conflict matrix and safe distance under heavy load.

## Verification

- **`cargo build`**: Uses **bundled SDL2** (first build compiles SDL2; needs CMake + C toolchain). No manual `SDL2.lib` / `LIB` setup on Windows unless you disable `bundled`.
- **Stats UI**: Window + console; no system font / SDL2_ttf required.

## Git / GitHub

- **Remote**: `https://github.com/jasimm7md/smart-road.git`.

## Notes

- **Windows**: Install MSVC + CMake. If CMake/SDL configure fails, see README and rust-sdl2 “bundled” docs.
- **Linux**: Install build deps (README); add any packages CMake lists as missing.
