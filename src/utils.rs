//! Small helpers shared across modules.

/// Turns a status name into the lowercase, dash-separated slug used on the command line.
pub fn slug(input: String) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .replace(' ', "-")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn slug_lowercases_dashes_and_drops_punctuation() {
        assert_eq!(slug("Done".to_string()), "done");
        assert_eq!(slug("In Progress".to_string()), "in-progress");
        assert_eq!(slug("Ready for test!".to_string()), "ready-for-test");
    }
}
