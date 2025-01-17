pub fn slug(input: String) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .replace(' ', "-")
        .to_lowercase()
}
