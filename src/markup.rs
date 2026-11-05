#[derive(Clone, Debug)]
pub enum Inline {
    Text(String),
    Math(String),
}

#[derive(Clone, Debug)]
pub enum Block {
    Paragraph(Vec<Inline>),
    MathContainer(String),
}

const INLINE_MACRO: &str = r"\inline-math{";
const DISPLAY_OPEN: &str = r"\begin{math-container}";
const DISPLAY_CLOSE: &str = r"\end{math-container}";

pub fn parse_document(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();
    let mut text = String::new();
    let mut cursor = 0;

    while cursor < source.len() {
        let rest = &source[cursor..];

        if rest.starts_with(DISPLAY_OPEN) {
            flush_text(&mut paragraph, &mut text);
            flush_paragraph(&mut blocks, &mut paragraph);

            let inner_start = cursor + DISPLAY_OPEN.len();
            if let Some(relative_end) = source[inner_start..].find(DISPLAY_CLOSE) {
                let inner_end = inner_start + relative_end;
                blocks.push(Block::MathContainer(
                    source[inner_start..inner_end].trim().to_string(),
                ));
                cursor = inner_end + DISPLAY_CLOSE.len();
            } else {
                text.push_str(DISPLAY_OPEN);
                cursor = inner_start;
            }
            continue;
        }

        if rest.starts_with(INLINE_MACRO) {
            flush_text(&mut paragraph, &mut text);
            let brace_at = cursor + INLINE_MACRO.len() - 1;
            if let Some((expression, after)) = read_braced(source, brace_at) {
                paragraph.push(Inline::Math(expression));
                cursor = after;
            } else {
                text.push_str(INLINE_MACRO);
                cursor += INLINE_MACRO.len();
            }
            continue;
        }

        if rest.starts_with("\n\n") {
            flush_text(&mut paragraph, &mut text);
            flush_paragraph(&mut blocks, &mut paragraph);
            cursor += 2;
            continue;
        }

        let next = rest
            .chars()
            .next()
            .expect("cursor is at a character boundary");
        text.push(next);
        cursor += next.len_utf8();
    }

    flush_text(&mut paragraph, &mut text);
    flush_paragraph(&mut blocks, &mut paragraph);
    blocks
}

pub(super) fn read_braced(source: &str, open_at: usize) -> Option<(String, usize)> {
    if source.as_bytes().get(open_at).copied()? != b'{' {
        return None;
    }

    let mut depth = 0usize;
    let mut cursor = open_at;
    let mut escaped = false;

    while cursor < source.len() {
        let ch = source[cursor..].chars().next()?;
        if ch == '{' && !escaped {
            depth += 1;
        } else if ch == '}' && !escaped {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some((source[open_at + 1..cursor].to_string(), cursor + 1));
            }
        }

        if ch == '\\' && !escaped {
            escaped = true;
        } else {
            escaped = false;
        }
        cursor += ch.len_utf8();
    }

    None
}

fn flush_text(paragraph: &mut Vec<Inline>, text: &mut String) {
    if !text.is_empty() {
        paragraph.push(Inline::Text(std::mem::take(text)));
    }
}

fn flush_paragraph(blocks: &mut Vec<Block>, paragraph: &mut Vec<Inline>) {
    if !paragraph.is_empty() {
        blocks.push(Block::Paragraph(std::mem::take(paragraph)));
    }
}
