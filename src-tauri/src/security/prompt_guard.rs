pub fn sanitize_context_text(input: &str) -> String {
    let lowered = input.to_lowercase();
    if lowered.contains("ignore previous instructions") || lowered.contains("system prompt") {
        return "[Filtered potentially malicious prompt-injection content]".to_string();
    }
    input.to_string()
}
