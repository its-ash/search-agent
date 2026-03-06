use regex::Regex;

use crate::retrieval::models::RetrievedChunk;

pub struct ExtractiveAnswer {
    pub answer: String,
    pub source_chunk_index: usize,
}

pub fn answer_from_context(question: &str, chunks: &[RetrievedChunk]) -> Option<ExtractiveAnswer> {
    if let Some(ans) = answer_variable_value(question, chunks) {
        return Some(ans);
    }
    answer_possession_question(question, chunks)
}

fn answer_variable_value(question: &str, chunks: &[RetrievedChunk]) -> Option<ExtractiveAnswer> {
    let var_name = infer_variable_name(question)?;

    let js_decl =
        Regex::new(&format!(r"(?m)\b(?:const|let|var)\s+{}\s*=\s*([^;\n]+)", regex::escape(&var_name))).ok()?;
    let py_decl = Regex::new(&format!(r"(?m)^\s*{}\s*=\s*([^\n#]+)", regex::escape(&var_name))).ok()?;

    for (idx, chunk) in chunks.iter().enumerate() {
        if let Some(caps) = js_decl.captures(&chunk.text) {
            let value = caps.get(1)?.as_str().trim();
            let value = cleanup_js_value(value);
            return Some(ExtractiveAnswer {
                answer: format!("`{var_name}` = {value}"),
                source_chunk_index: idx,
            });
        }
        if let Some(caps) = py_decl.captures(&chunk.text) {
            let value = caps.get(1)?.as_str().trim();
            let value = cleanup_python_value(value);
            return Some(ExtractiveAnswer {
                answer: format!("`{var_name}` = {value}"),
                source_chunk_index: idx,
            });
        }
    }

    None
}

fn answer_possession_question(question: &str, chunks: &[RetrievedChunk]) -> Option<ExtractiveAnswer> {
    let q = normalize(question);
    let q_lower = q.to_lowercase();
    let is_possession = q_lower.starts_with("do i have")
        || q_lower.starts_with("have i")
        || q_lower.starts_with("did i")
        || q_lower.starts_with("do i own");
    if !is_possession {
        return None;
    }

    let wanted_tokens: Vec<String> = q_lower
        .split_whitespace()
        .filter(|t| !STOP_WORDS.contains(t))
        .map(ToString::to_string)
        .collect();

    if wanted_tokens.is_empty() {
        return None;
    }

    for (idx, chunk) in chunks.iter().enumerate() {
        let hay = format!("{} {}", chunk.file.to_lowercase(), normalize(&chunk.text).to_lowercase());
        let match_count = wanted_tokens.iter().filter(|t| hay.contains(t.as_str())).count();

        if match_count >= wanted_tokens.len().min(2) {
            let topic = topic_from_tokens(&wanted_tokens);
            let answer = if q_lower.contains("certificate") || hay.contains("certificate") || hay.contains("statement of achievement") {
                format!("Yes, you have a {} certificate.", topic)
            } else {
                format!("Yes, I found {} in your indexed documents.", topic)
            };
            return Some(ExtractiveAnswer {
                answer,
                source_chunk_index: idx,
            });
        }
    }

    None
}

const STOP_WORDS: &[&str] = &[
    "do", "i", "have", "did", "own", "is", "the", "a", "an", "my", "me", "of", "in", "to", "for",
    "and", "or", "certificate", "certification", "any", "there",
];

fn topic_from_tokens(tokens: &[String]) -> String {
    let words: Vec<&str> = tokens
        .iter()
        .filter(|t| t.as_str() != "certificate" && t.as_str() != "certification")
        .map(String::as_str)
        .take(4)
        .collect();
    if words.is_empty() {
        "that".to_string()
    } else {
        words.join(" ")
    }
}

fn normalize(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c.is_whitespace() || c == '_' { c } else { ' ' })
        .collect()
}

fn cleanup_js_value(raw: &str) -> String {
    let mut s = raw.trim().to_string();

    // If chunk text collapsed newlines, stop before next declaration.
    for marker in [" const ", " let ", " var ", " function ", " class "] {
        if let Some(pos) = s.find(marker) {
            s.truncate(pos);
        }
    }

    // Keep complete array/object/string literals when possible.
    if s.starts_with('[') {
        if let Some(v) = cut_balanced(&s, '[', ']') {
            return v;
        }
    } else if s.starts_with('{') {
        if let Some(v) = cut_balanced(&s, '{', '}') {
            return v;
        }
    } else if s.starts_with('"') || s.starts_with('\'') || s.starts_with('`') {
        if let Some(v) = cut_string_literal(&s) {
            return v;
        }
    }

    s.trim().trim_end_matches(',').to_string()
}

fn cleanup_python_value(raw: &str) -> String {
    raw.trim().trim_end_matches(',').to_string()
}

fn cut_balanced(input: &str, open: char, close: char) -> Option<String> {
    let mut depth = 0usize;
    let mut in_str: Option<char> = None;
    let mut escaped = false;

    for (i, ch) in input.char_indices() {
        if let Some(q) = in_str {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                in_str = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            in_str = Some(ch);
            continue;
        }

        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(input[..=i].trim().to_string());
            }
        }
    }
    None
}

fn cut_string_literal(input: &str) -> Option<String> {
    let quote = input.chars().next()?;
    let mut escaped = false;
    for (i, ch) in input.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(input[..=i].to_string());
        }
    }
    None
}

fn infer_variable_name(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let patterns = [
        Regex::new(r"value of ([a-zA-Z_][a-zA-Z0-9_]*)").ok()?,
        Regex::new(r"value of ([a-zA-Z_][a-zA-Z0-9_]*)\?").ok()?,
        Regex::new(r"value of the ([a-zA-Z_][a-zA-Z0-9_]*)").ok()?,
        Regex::new(r"what is ([a-zA-Z_][a-zA-Z0-9_]*)").ok()?,
        Regex::new(r"what's ([a-zA-Z_][a-zA-Z0-9_]*)").ok()?,
    ];

    for re in patterns {
        if let Some(caps) = re.captures(&q) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    let cleaned: String = q
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c.is_whitespace() { c } else { ' ' })
        .collect();
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.len() >= 2 {
        for i in 0..(tokens.len() - 1) {
            if tokens[i] == "of" {
                let candidate = tokens[i + 1];
                if candidate.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_') == Some(true) {
                    return Some(candidate.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::answer_from_context;
    use crate::retrieval::models::RetrievedChunk;

    #[test]
    fn extracts_js_value() {
        let chunks = vec![RetrievedChunk {
            chunk_id: "c1".into(),
            file: "test.js".into(),
            text: "const input = [\"zebra\", \"dog\"]".into(),
            score: 0.8,
            page_start: None,
            page_end: None,
        }];

        let ans = answer_from_context("what is the value of input?", &chunks).unwrap();
        assert!(ans.answer.contains("zebra"));
        assert!(!ans.answer.contains("const output"));
    }

    #[test]
    fn extracts_only_array_when_chunk_has_multiple_decls() {
        let chunks = vec![RetrievedChunk {
            chunk_id: "c1".into(),
            file: "test.js".into(),
            text: "const input = [\"zebra\", \"dog\", \"duck\", \"dove\"] const output = [\"z\", \"dog\"]".into(),
            score: 0.8,
            page_start: None,
            page_end: None,
        }];
        let ans = answer_from_context("value of input", &chunks).unwrap();
        assert_eq!(ans.answer, "`input` = [\"zebra\", \"dog\", \"duck\", \"dove\"]");
    }

    #[test]
    fn answers_possession_question_concisely() {
        let chunks = vec![RetrievedChunk {
            chunk_id: "c1".into(),
            file: "Ashvini JavaScript.pdf".into(),
            text: "Statement of Achievement JavaScript Essentials 1".into(),
            score: 0.9,
            page_start: None,
            page_end: None,
        }];
        let ans = answer_from_context("Do i have javascript certificate?", &chunks).unwrap();
        assert!(ans.answer.to_lowercase().starts_with("yes"));
        assert!(ans.answer.to_lowercase().contains("javascript"));
    }
}
