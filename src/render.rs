use latex_rust::{Dim, MathFont, MathStyle, SvgOptions, latex_to_svg, layout, parse};

#[derive(Clone, Debug)]
pub struct SvgImage {
    pub markup: String,
    pub width: f64,
    pub height: f64,
    pub view_box: String,
    pub inner: String,
}

pub fn math_svg(expression: &str, font: &MathFont, display: bool) -> Result<SvgImage, String> {
    let prepared = fill_empty_slots(expression);

    let mut options = SvgOptions::new();
    options.font_size_pt = Dim::from_i64(if display { 18 } else { 16 });
    options.display = display;

    let markup = latex_to_svg(&prepared, font, &options).map_err(|error| error.to_string())?;

    let width_pt = svg_dimension(&markup, "width").unwrap_or(42.0);
    let height_pt = svg_dimension(&markup, "height").unwrap_or(18.0);
    let view_box =
        svg_attribute(&markup, "viewBox").unwrap_or_else(|| format!("0 0 {width_pt} {height_pt}"));
    let inner = svg_inner(&markup)
        .ok_or_else(|| "latex-rust returned an SVG without a root element".to_string())?;

    let scale = 96.0 / 72.0;
    Ok(SvgImage {
        markup,
        width: width_pt * scale + 2.0,
        height: height_pt * scale + 2.0,
        view_box,
        inner,
    })
}

pub fn fill_empty_slots(expression: &str) -> String {
    if expression.trim().is_empty() {
        return r"\boxed{\phantom{M}}".to_string();
    }

    let mut output = String::with_capacity(expression.len() + 24);
    let mut cursor = 0;

    while cursor < expression.len() {
        let rest = &expression[cursor..];
        if rest.starts_with("{}") {
            output.push_str(r"{\boxed{\phantom{M}}}");
            cursor += 2;
            continue;
        }

        let ch = rest
            .chars()
            .next()
            .expect("cursor is at a character boundary");
        output.push(ch);
        cursor += ch.len_utf8();
    }

    output
}

fn svg_dimension(svg: &str, name: &str) -> Option<f64> {
    let value = svg_attribute(svg, name)?;
    let numeric = value
        .trim()
        .strip_suffix("pt")
        .or_else(|| value.trim().strip_suffix("px"))
        .unwrap_or(value.trim());
    numeric.parse::<f64>().ok()
}

fn svg_attribute(svg: &str, name: &str) -> Option<String> {
    let double = format!("{name}=\"");
    let single = format!("{name}='");

    if let Some(start) = svg.find(&double) {
        let value_start = start + double.len();
        let value_end = svg[value_start..].find('"')? + value_start;
        return Some(svg[value_start..value_end].to_string());
    }

    if let Some(start) = svg.find(&single) {
        let value_start = start + single.len();
        let value_end = svg[value_start..].find('\'')? + value_start;
        return Some(svg[value_start..value_end].to_string());
    }

    None
}

fn svg_inner(svg: &str) -> Option<String> {
    let root_end = svg.find('>')? + 1;
    let close_start = svg.rfind("</svg>")?;
    if close_start < root_end {
        return None;
    }
    Some(svg[root_end..close_start].to_string())
}
