use anyhow::{Result, anyhow};

pub fn parse_human_size(s: &str) -> Result<u64> {
    let s_clean = s.trim();

    if let Ok(v) = s_clean.parse::<u64>() {
        return Ok(v);
    }

    let mut num_part = String::new();
    let mut unit_part = String::new();

    for c in s_clean.chars() {
        if c.is_ascii_digit() || c == '.' {
            num_part.push(c);
        } else {
            unit_part.push(c);
        }
    }

    let number: f64 = num_part
        .parse()
        .map_err(|_| anyhow!("Invalid number in '{}'", s))?;

    // multiply by 1000 instead of 1024 to not confuse the user
    // although may be dependend on who the user is :D
    let multiplier = match unit_part.trim().to_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" => 1000.0,
        "m" | "mb" => 1000.0 * 1000.0,
        "g" | "gb" => 1000.0 * 1000.0 * 1000.0,
        "t" | "tb" => 1000.0 * 1000.0 * 1000.0 * 1000.0,
        other => return Err(anyhow!("Unknown size suffix '{}'", other)),
    };

    Ok((number * multiplier) as u64)
}
