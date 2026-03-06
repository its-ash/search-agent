#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk_index: usize,
    pub text: String,
    pub token_count: usize,
    pub section: Option<String>,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
}

pub fn chunk_text(text: &str, target_tokens: usize, overlap_tokens: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let mut chunks = Vec::new();
    let mut start_line = 0usize;
    let target_tokens = target_tokens.max(1);
    let overlap_tokens = overlap_tokens.min(target_tokens.saturating_sub(1));

    while start_line < lines.len() {
        let mut end_line = start_line;
        let mut tokens = 0usize;

        while end_line < lines.len() {
            let next_tokens = approx_tokens(lines[end_line]);
            if tokens > 0 && tokens + next_tokens > target_tokens {
                break;
            }
            tokens += next_tokens;
            end_line += 1;
        }

        if end_line == start_line {
            end_line += 1;
            tokens = approx_tokens(lines[start_line]);
        }

        let segment = lines[start_line..end_line].join("\n");
        chunks.push(Chunk {
            chunk_index: chunks.len(),
            token_count: tokens,
            text: segment,
            section: None,
            page_start: None,
            page_end: None,
        });

        if end_line >= lines.len() {
            break;
        }

        if overlap_tokens == 0 {
            start_line = end_line;
            continue;
        }

        // Walk backward to include approximately overlap_tokens worth of lines.
        let mut back = end_line;
        let mut overlap_count = 0usize;
        while back > start_line {
            let prev = back - 1;
            overlap_count += approx_tokens(lines[prev]);
            if overlap_count >= overlap_tokens {
                back = prev;
                break;
            }
            back = prev;
        }

        if back == start_line {
            start_line = end_line;
        } else {
            start_line = back;
        }
    }

    chunks
}

fn approx_tokens(line: &str) -> usize {
    line.split_whitespace().count().max(1)
}

#[cfg(test)]
mod tests {
    use super::chunk_text;

    #[test]
    fn chunking_uses_overlap() {
        let input = (0..120)
            .map(|i| format!("w{i}"))
            .collect::<Vec<_>>()
            .chunks(10)
            .map(|line| line.join(" "))
            .collect::<Vec<_>>()
            .join("\n");
        let chunks = chunk_text(&input, 50, 10);
        assert!(chunks.len() >= 3);
        assert!(chunks[0].token_count >= 40);
    }
}
