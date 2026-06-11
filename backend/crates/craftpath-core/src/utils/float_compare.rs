use std::cmp::Ordering;

pub fn cmp_f64(a: f64, b: f64) -> Ordering {
    let eps = 1e-15; // f64 should have a precision of 15 (to 17). cap at 15
    if (a - b).abs() < eps {
        Ordering::Equal
    } else if a < b {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
