//! A scroll container's own inline text moves by its scroll offset, once.

use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{DocumentConfig, ScrollBehavior};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use std::sync::Arc;

const STYLE: &str = "width:100px; white-space:nowrap; overflow:hidden; font:16px monospace";
const TEXT: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

fn render(extra_style: &str, scroll_x: f64) -> Vec<u8> {
    let html = format!(
        "<body style='margin:0'><div id='test' style='{STYLE}; {extra_style}'>{TEXT}</div></body>"
    );
    let mut doc = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            viewport: Some(Viewport::new(120, 40, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    let id = doc.query_selector("#test").unwrap().unwrap();
    doc.scroll_by(id, scroll_x, 0.0, ScrollBehavior::Instant);
    doc.resolve(0.0);
    assert_eq!(doc.get_node(id).unwrap().scroll_offset().x, scroll_x);
    render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| paint_scene(scene, doc.as_mut(), 1.0, 120, 40, 0, 0),
        120,
        40,
    )
}

#[test]
fn scrolled_text_moves_by_the_scroll_offset() {
    assert_eq!(render("", 20.0), render("text-indent:-20px", 0.0));
}

#[test]
fn scrolled_background_clip_text_moves_by_the_scroll_offset() {
    let clip =
        "background:linear-gradient(90deg, red, blue); background-clip:text; color:transparent";
    assert_eq!(
        render(clip, 20.0),
        render(&format!("{clip}; text-indent:-20px"), 0.0)
    );
}
