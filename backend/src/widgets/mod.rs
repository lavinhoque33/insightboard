pub mod crypto;
pub mod github;
pub mod http;
pub mod net_policy;
pub mod news;
pub mod status;
pub mod weather;

/// Trim and collapse whitespace runs in free-text query input (city, topic).
pub fn normalize_text(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize_text;

    #[test]
    fn free_text_is_trimmed_and_collapsed() {
        assert_eq!(normalize_text("  New \t York "), "New York");
        assert_eq!(normalize_text("   "), "");
    }
}
