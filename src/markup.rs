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

const INLINE_MACROS: [&str; 2] = [r"\math{", r"\${"];
const BLOCK_MACROS: [&str; 2] = [r"\container{", r"\#{"];

pub fn parse_document(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();
    let mut text = String::new();
    let mut cursor = 0;

    'parse: while cursor < source.len() {
        let rest = &source[cursor..];

        if let Some(block_macro) = matching_macro(rest, &BLOCK_MACROS) {
            let brace_at = cursor + block_macro.len() - 1;

            if let Some((expression, after)) = read_braced(source, brace_at) {
                flush_text(&mut paragraph, &mut text);
                flush_paragraph(&mut blocks, &mut paragraph);

                blocks.push(Block::MathContainer(expression));
                cursor = after;
            } else {
                text.push_str(block_macro);
                cursor += block_macro.len();
            }

            continue 'parse;
        }

        if let Some(inline_macro) = matching_macro(rest, &INLINE_MACROS) {
            let brace_at = cursor + inline_macro.len() - 1;

            if let Some((expression, after)) = read_braced(source, brace_at) {
                flush_text(&mut paragraph, &mut text);

                paragraph.push(Inline::Math(expression));
                cursor = after;
            } else {
                text.push_str(inline_macro);
                cursor += inline_macro.len();
            }

            continue 'parse;
        }

        if let Some(after) = blank_line_end(source, cursor) {
            flush_text(&mut paragraph, &mut text);
            flush_paragraph(&mut blocks, &mut paragraph);

            cursor = after;
            continue 'parse;
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

fn matching_macro<'a>(rest: &str, macros: &'a [&'a str]) -> Option<&'a str> {
    for &macro_text in macros {
        if rest.starts_with(macro_text) {
            return Some(macro_text);
        }
    }

    None
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

fn line_ending_end(source: &str, at: usize) -> Option<usize> {
    let bytes = source.as_bytes();

    if bytes.get(at) == Some(&b'\n') {
        Some(at + 1)
    } else if bytes.get(at) == Some(&b'\r') && bytes.get(at + 1) == Some(&b'\n') {
        Some(at + 2)
    } else {
        None
    }
}

fn blank_line_end(source: &str, start: usize) -> Option<usize> {
    let mut probe = line_ending_end(source, start)?;

    while source
        .as_bytes()
        .get(probe)
        .is_some_and(|b| *b == b' ' || *b == b'\t')
    {
        probe += 1;
    }

    line_ending_end(source, probe)
}
