use std::{fs, rc::Rc};

use floem::{
    prelude::*,
    text::FamilyOwned,
    views::{editor::text::SimpleStyling, text_editor::text_editor},
};
use latex_rust::MathFont;
use rfd::FileDialog;

mod markup;
mod render;

const SOURCE_FONT_SIZE: usize = 18;
const PREVIEW_FONT_SIZE: usize = 24;
const APP_FONT: &str = "STIX Two Text";

fn main() {
    let font = Rc::new(MathFont::stix_two_math().expect("latex-rust could not load STIX Two Math"));
    floem::launch(move || app_view(font.clone()));
}

fn app_view(font: Rc<MathFont>) -> impl IntoView {
    let editor = text_editor(
        r#"Let \${A} be a set of positive integers. Consider the following relation \${R} on the elements of \${A}. If \${a} and \${b} are positive integers then \${(a,b)} is in \${R} if and only if \${a \mid b}. Then \${R} is a partial order. 

\${\mathbf{Proof.}} We must prove that \${R} is reflexive, anti-symmetric, and transitive.

\${\mathbf{•\ Reflexive:}} We must prove, \${\forall a\in A, (a,a)\in R}. We let \${a} be an arbitrary member of \${A}. This is because \${a\mid a}, \${(a,a)\in R}. //

\${\mathbf{•\ Anti-symmetric:}} Assume \${((a,b),(b,a)\in R)\implies a=b}. Therefore \${b=ma} and \${a=nb} for some \${m,n\in Z}. Thus \${b=ma=m(nb)=(mn)b}. Now \${mn=1}, implying that \${m=n=1}, or \${m=n=-1}. We can eliminate \${m=n=-1} since \${b=ma}, and \${a,b>0}. So \${m=n=1}, so \${b=a}. //

\${\mathbf{•\ Transitive:}} Assume \${(a,b)\in R} and \${(b,c)\in R}. Therefore \${b=ma} and \${c=nb} for some \${m,n\in Z}. Thus \${c=nb=n(ma)=(nm)a}. Since \${nm\in Z}, this implies that \${a\mid c}, so \${(a,c)\in R}. //"#,
    );
    let document = editor.doc();

    let source = RwSignal::new(document.text().to_string());
    let status = RwSignal::new(String::from("Ready"));

    let editor = editor.update(move |_| {
        source.set(document.clone().text().to_string());
    });

    let preview_source = source;
    let preview_font = font.clone();
    let preview = dyn_view(move || {
        let current = preview_source.get();
        document_preview(&current, preview_font.clone())
    });

    let save_button = button("Save .mathdoc").action(move || {
        if let Some(path) = FileDialog::new()
            .add_filter("MathPad document", &["mathdoc", "txt"])
            .save_file()
        {
            match fs::write(&path, source.clone().get()) {
                Ok(()) => status.clone().set(format!("Saved {}", path.display())),
                Err(error) => status
                    .clone()
                    .set(format!("Could not save source: {error}")),
            }
        }
    });

    let export_png = button("Export PNG");
    let export_pdf = button("Export PDF");

    let editor = editor
        .styling(
            SimpleStyling::builder()
                .font_size(SOURCE_FONT_SIZE)
                .font_family(vec![FamilyOwned::Name(APP_FONT.to_string())])
                .line_height(1.45)
                .build(),
        )
        .editor_style(|s| s.hide_gutter(true))
        .style(|s| {
            s.width_full()
                .flex_grow(1.0f32)
                .min_width(0.0)
                .min_height(0.0)
                .padding_pct(2.0)
                .background(Color::WHITE)
                .color(Color::BLACK)
        });

    let source_pane = v_stack((label(|| "Source"), editor)).style(|s| {
        s.width_pct(48.0)
            .flex_grow(1.0f32)
            .min_width(0.0)
            .min_height(0.0)
            .gap(8.0)
    });

    let preview_pane = v_stack((
        label(|| "Rendered"),
        scroll(preview).style(|s| s.flex_grow(1.0f32).min_width(0.0).min_height(0.0)),
    ))
    .style(|s| {
        s.width_pct(48.0)
            .flex_grow(1.0f32)
            .min_width(0.0)
            .min_height(0.0)
            .gap(8.0)
    });

    let panes = h_stack((source_pane, preview_pane)).style(|s| {
        s.width_full()
            .flex_grow(1.0f32)
            .min_width(0.0)
            .min_height(0.0)
            .gap(14.0)
    });

    let toolbar = h_stack((save_button, export_png, export_pdf)).style(|s| {
        s.width_full()
            .flex_wrap(floem::style::FlexWrap::Wrap)
            .gap(6.0)
    });

    v_stack((
        label(|| "Cosmos").style(|s| s.font_size(25.0)),
        toolbar,
        panes,
        label(move || status.get()).style(|s| s.font_size(12.0)),
    ))
    .style(|s| {
        s.size_full()
            .font_family(APP_FONT.to_string())
            .padding_pct(2.0)
            .gap(10.0)
    })
}

fn document_preview(source_text: &str, font: Rc<MathFont>) -> impl IntoView + use<> {
    let blocks = markup::parse_document(source_text);
    let children = blocks
        .into_iter()
        .map(|block| block_preview(block, font.clone()))
        .collect::<Vec<_>>();

    v_stack_from_iter(children).style(|s| s.width_full().padding_pct(3.0).gap(16.0))
}

fn block_preview(block: markup::Block, font: Rc<MathFont>) -> impl IntoView {
    dyn_view(move || match block.clone() {
        markup::Block::Paragraph(parts) => {
            let mut children = Vec::new();

            for part in parts {
                match part {
                    markup::Inline::Text(text) => {
                        for token in split_text_for_wrap(&text) {
                            children.push(
                                label(move || token.clone())
                                    .style(|s| {
                                        s.font_family(APP_FONT.to_string())
                                            .font_size(PREVIEW_FONT_SIZE as i32)
                                    })
                                    .into_any(),
                            );
                        }
                    }

                    markup::Inline::Math(expression) => {
                        children.push(render_math(expression, font.clone(), false).into_any());
                    }
                }
            }

            h_stack_from_iter(children)
                .style(|s| {
                    s.width_full()
                        .flex_wrap(floem::style::FlexWrap::Wrap)
                        .column_gap(0.0)
                        .row_gap(6.0)
                        .items_center()
                })
                .into_any()
        }

        markup::Block::MathContainer(expression) => {
            render_math(expression.clone(), font.clone(), true).into_any()
        }
    })
}

fn split_text_for_wrap(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while cursor < text.len() {
        let start = cursor;

        while cursor < text.len() {
            let ch = text[cursor..].chars().next().unwrap();

            if !ch.is_whitespace() {
                break;
            }

            cursor += ch.len_utf8();
        }

        while cursor < text.len() {
            let ch = text[cursor..].chars().next().unwrap();

            if ch.is_whitespace() {
                break;
            }

            cursor += ch.len_utf8();
        }

        while cursor < text.len() {
            let ch = text[cursor..].chars().next().unwrap();

            if !ch.is_whitespace() {
                break;
            }

            cursor += ch.len_utf8();
        }

        tokens.push(text[start..cursor].to_string());
    }

    tokens
}

fn render_math(expression: String, font: Rc<MathFont>, display: bool) -> impl IntoView {
    let draft = RwSignal::new(expression.clone());

    match render::math_svg(&draft.get(), font.as_ref(), display) {
        Ok(image) => floem::views::svg(image.markup)
            .style(move |s| {
                s.width(image.width)
                    .height(image.height)
                    .flex_shrink(0.0f32)
                    .row_gap(6.0)
                    .items_center()
            })
            .into_any(),

        Err(error) => label(move || format!("Parse error: {error}"))
            .style(|s| s.font_size(PREVIEW_FONT_SIZE as i32))
            .into_any(),
    }
}
