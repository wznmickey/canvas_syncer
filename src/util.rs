use scraper::{Html, Selector};
use serde_json::Value;
pub fn get_file_link_from_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[data-api-endpoint]").unwrap();
    let values: Vec<String> = document
        .select(&selector)
        .filter_map(|element| element.value().attr("data-api-endpoint").map(String::from))
        .collect();
    values
}

pub trait GetFromJson<T, A, B> {
    fn get_from_json(x: &Value, a: A, b: B) -> Option<T>;
}

pub fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num_str, unit_str) = if s.ends_with("GB") || s.ends_with("gb") {
        (s.trim_end_matches("GB").trim_end_matches("gb"), "GB")
    } else if s.ends_with("MB") || s.ends_with("mb") {
        (s.trim_end_matches("MB").trim_end_matches("mb"), "MB")
    } else if s.ends_with("KB") || s.ends_with("kb") {
        (s.trim_end_matches("KB").trim_end_matches("kb"), "KB")
    } else if s.ends_with("B") || s.ends_with("b") {
        (s.trim_end_matches("B").trim_end_matches("b"), "B")
    } else {
        // Assume bytes if no unit is specified
        (s, "B")
    };

    let num = num_str.trim().parse::<f64>().ok()?;

    let bytes = match unit_str {
        "GB" => num * 1024.0 * 1024.0 * 1024.0,
        "MB" => num * 1024.0 * 1024.0,
        "KB" => num * 1024.0,
        _ => num, // Default to bytes
    };

    Some(bytes as u64)
}

pub enum SizeAction {
    Allow,
    Deny,
}

pub enum SizeOperator {
    GreaterThanOrEqual,
    LessThanOrEqual,
    GreaterThan,
    LessThan,
}

pub struct ParsedSizeRule {
    pub action: SizeAction,
    pub operator: SizeOperator,
    pub size_bytes: u64,
}

pub fn parse_size_rule(rule_str: &str) -> Option<ParsedSizeRule> {
    let parts: Vec<&str> = rule_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None; // Invalid format
    }

    let action_str = parts[0].trim();
    let rule_part = parts[1].trim();

    let action = match action_str.to_lowercase().as_str() {
        "allow" => SizeAction::Allow,
        "deny" => SizeAction::Deny,
        _ => return None, // Invalid action
    };

    let (operator, size_str) = if rule_part.starts_with(">=") {
        (SizeOperator::GreaterThanOrEqual, rule_part.trim_start_matches(">="))
    } else if rule_part.starts_with("<=") {
        (SizeOperator::LessThanOrEqual, rule_part.trim_start_matches("<="))
    } else if rule_part.starts_with(">") {
        (SizeOperator::GreaterThan, rule_part.trim_start_matches(">"))
    } else if rule_part.starts_with("<") {
        (SizeOperator::LessThan, rule_part.trim_start_matches("<"))
    } else {
        return None; // Invalid operator
    };

    let size_bytes = parse_size_string(size_str)?;

    Some(ParsedSizeRule {
        action,
        operator,
        size_bytes,
    })
}
