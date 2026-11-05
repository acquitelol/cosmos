use std::{fs, rc::Rc};

use floem::{
    prelude::*,
    views::{editor::text::SimpleStyling, text_editor::text_editor},
};
use latex_rust::MathFont;
use rfd::FileDialog;

mod markup;
mod render;

fn main() {
    let font = Rc::new(MathFont::stix_two_math().expect("latex-rust could not load STIX Two Math"));
    floem::launch(move || app_view(font.clone()));
}

fn app_view(font: Rc<MathFont>) -> impl IntoView {
    let editor = text_editor("");
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
                .font_size(16)
                .line_height(1.45)
                .build(),
        )
        .editor_style(|s| s.hide_gutter(true))
        .style(|s| {
            s.width(570.0)
                .height(650.0)
                .padding(12.0)
                .background(floem::peniko::Color::WHITE)
                .color(floem::peniko::Color::BLACK)
        });

    let source_pane =
        v_stack((label(|| "Source"), editor)).style(|s| s.width(580.0).height(690.0).gap(8.0));

    let preview_pane = v_stack((
        label(|| "Rendered"),
        scroll(preview).style(|s| s.width(600.0).height(650.0)),
    ))
    .style(|s| s.width(610.0).height(690.0).gap(8.0));

    v_stack((
        label(|| "Cosmos").style(|s| s.font_size(25.0)),
        h_stack((save_button, export_png, export_pdf)).style(|s| s.gap(6.0)),
        h_stack((source_pane, preview_pane)).style(|s| s.gap(14.0)),
        label(move || status.get()).style(|s| s.font_size(12.0)),
    ))
    .style(|s| s.size_full().padding(16.0).gap(10.0))
}

fn document_preview(source_text: &str, font: Rc<MathFont>) -> impl IntoView + use<> {
    let blocks = markup::parse_document(source_text);
    let children = blocks
        .into_iter()
        .map(|block| block_preview(block, font.clone()))
        .collect::<Vec<_>>();

    v_stack_from_iter(children).style(|s| s.width(570.0).padding(22.0).gap(16.0))
}

fn block_preview(block: markup::Block, font: Rc<MathFont>) -> impl IntoView {
    dyn_view(move || match block.clone() {
        markup::Block::Paragraph(parts) => {
            let children = parts
                .into_iter()
                .map(|part| inline_preview(part, font.clone()))
                .collect::<Vec<_>>();

            h_stack_from_iter(children)
                .style(|s| s.width_full().gap(5.0).items_center())
                .into_any()
        }
        markup::Block::MathContainer(expression) => {
            render_math(expression.clone(), font.clone(), true).into_any()
        }
    })
}

fn inline_preview(part: markup::Inline, font: Rc<MathFont>) -> impl IntoView {
    match part {
        markup::Inline::Text(text) => label(move || text.clone()).into_any(),
        markup::Inline::Math(expression) => render_math(expression, font, false).into_any(),
    }
}

fn render_math(expression: String, font: Rc<MathFont>, display: bool) -> impl IntoView {
    let draft = RwSignal::new(expression.clone());

    match render::math_svg(&draft.get(), font.as_ref(), display) {
        Ok(image) => floem::views::svg(image.markup)
            .style(move |s| {
                s.width(image.width)
                    .height(image.height)
                    .gap(4.0)
                    .items_center()
            })
            .into_any(),
        Err(error) => label(move || format!("Parse error: {error}")).into_any(),
    }
}
