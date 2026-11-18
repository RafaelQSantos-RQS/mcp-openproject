/// Helper module to parse and normalize human-friendly durations into ISO 8601 Durations (e.g. "PT2H30M").
///
/// Supported formats:
/// - ISO 8601: "PT4H", "PT2H30M", "P1D"
/// - Hours with suffix: "4h", "2.5h", "2,5h"
/// - Composite: "1h30m", "1h 30m"
/// - Minutes: "30m", "45min"
/// - Colon format: "02:30", "1:15"
/// - Plain numbers (assumed hours): "4", "1.5"
pub fn parse_duration(input: &str) -> Result<String, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Duration string cannot be empty".into());
    }
    let upper = s.to_ascii_uppercase().replace(',', ".");
    if upper.starts_with('P') {
        return Ok(upper);
    }

    // HH:MM format
    if let Some((h_str, m_str)) = upper.split_once(':') {
        let h: u32 = h_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid hours in '{input}'"))?;
        let m: u32 = m_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid minutes in '{input}'"))?;
        return Ok(match (h, m) {
            (h, 0) => format!("PT{h}H"),
            (0, m) => format!("PT{m}M"),
            (h, m) => format!("PT{h}H{m}M"),
        });
    }

    // Composite format like "1h30m" or "1h 30m"
    if upper.contains('M')
        && let Some((h_part, m_part)) = upper.split_once('H')
    {
        let h: f64 = h_part
            .trim()
            .parse()
            .map_err(|_| format!("Invalid hours in '{input}'"))?;
        let m_str = m_part
            .trim()
            .trim_end_matches("IN")
            .trim_end_matches('M')
            .trim();
        let m: f64 = m_str
            .parse()
            .map_err(|_| format!("Invalid minutes in '{input}'"))?;
        let total_mins = ((h * 60.0) + m).round() as u64;
        let final_h = total_mins / 60;
        let final_m = total_mins % 60;
        return Ok(match (final_h, final_m) {
            (h, 0) => format!("PT{h}H"),
            (0, m) => format!("PT{m}M"),
            (h, m) => format!("PT{h}H{m}M"),
        });
    }

    // Minutes format like "30m", "45min"
    if upper.ends_with('M') || upper.ends_with("MIN") {
        let m_str = upper.trim_end_matches("MIN").trim_end_matches('M').trim();
        let m: f64 = m_str
            .parse()
            .map_err(|_| format!("Invalid minutes in '{input}'"))?;
        let total_mins = m.round() as u64;
        let final_h = total_mins / 60;
        let final_m = total_mins % 60;
        return Ok(match (final_h, final_m) {
            (h, 0) => format!("PT{h}H"),
            (0, m) => format!("PT{m}M"),
            (h, m) => format!("PT{h}H{m}M"),
        });
    }

    // Hours format like "4h", "2.5h", or plain number "4", "1.5"
    let h_str = upper.trim_end_matches('H').trim();
    let hours: f64 = h_str.parse().map_err(|_| {
        format!("Cannot parse duration '{input}'. Use e.g. '4h', '2.5h', '30m' or 'PT4H'")
    })?;
    if hours < 0.0 {
        return Err("Duration cannot be negative".into());
    }
    let total_mins = (hours * 60.0).round() as u64;
    let final_h = total_mins / 60;
    let final_m = total_mins % 60;
    Ok(match (final_h, final_m) {
        (h, 0) => format!("PT{h}H"),
        (0, m) => format!("PT{m}M"),
        (h, m) => format!("PT{h}H{m}M"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_various_formats() {
        assert_eq!(parse_duration("4h").unwrap(), "PT4H");
        assert_eq!(parse_duration("2.5h").unwrap(), "PT2H30M");
        assert_eq!(parse_duration("2,5h").unwrap(), "PT2H30M");
        assert_eq!(parse_duration("1h30m").unwrap(), "PT1H30M");
        assert_eq!(parse_duration("45m").unwrap(), "PT45M");
        assert_eq!(parse_duration("45min").unwrap(), "PT45M");
        assert_eq!(parse_duration("01:15").unwrap(), "PT1H15M");
        assert_eq!(parse_duration("PT8H").unwrap(), "PT8H");
        assert_eq!(parse_duration("4").unwrap(), "PT4H");
        assert_eq!(parse_duration("0").unwrap(), "PT0H");
        assert_eq!(parse_duration("1.5").unwrap(), "PT1H30M");
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("-2h").is_err());
        assert!(parse_duration("invalid").is_err());
    }
}
