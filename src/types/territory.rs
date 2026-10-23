//! Pure territory ownership calculation shared by influence consumers.

/// Near ties (within one percent of the strongest influence) remain contested.
pub fn controller(values: [f32; 5]) -> Option<usize> {
    let mut best = 0;
    for i in 1..5 {
        if values[i] > values[best] {
            best = i;
        }
    }
    let strongest = values[best];
    if strongest <= 0.00001 {
        return None;
    }
    if values
        .iter()
        .enumerate()
        .any(|(i, value)| i != best && strongest - value <= strongest * 0.01)
    {
        None
    } else {
        Some(best)
    }
}
