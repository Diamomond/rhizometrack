//! XP and leveling logic.
/// Convert seconds of study into XP. Policy: 1 XP per full minute.
pub fn seconds_to_xp(seconds: i64) -> i64 {
    seconds / 60
}

/// Given total XP, compute current level and XP progress towards next level.
/// Simple leveling curve: XP required for next level = 100 * level
/// Level 1 starts at 0 XP. Level increases when total_xp >= sum_{i=1..level} (100*i)
/// We'll compute level such that total_xp >= xp_needed_for_level(level) but < next.
pub fn level_for_xp(total_xp: i64) -> (i64, i64, i64) {
    if total_xp <= 0 {
        return (1, 0, 100);
    }

    let mut level = 1i64;
    let mut consumed = 0i64;
    loop {
        let need = 100 * level;
        if total_xp < consumed + need {
            let progress = total_xp - consumed;
            return (level, progress, need);
        }
        consumed += need;
        level += 1;
        // safety cap
        if level > 10000 {
            return (level, total_xp - consumed, 100 * level);
        }
    }
}
