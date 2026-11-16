//! `text-overflow`: which lines get a marker, and that painting it changes only those.

use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{BaseDocument, DocumentConfig, ScrollBehavior, node::TextLayout};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_paint::paint_scene;
use blitz_test_harness::Harness;
use blitz_traits::shell::{ColorScheme, Viewport};
use std::sync::Arc;

const LONG: &str = "The quick brown fox jumps over the lazy dog";

fn html(text_overflow: &str, text: &str) -> String {
    format!(
        "<div id='test' style='width:100px; white-space:nowrap; overflow:hidden; \
         text-overflow:{text_overflow}'>{text}</div>"
    )
}

fn inline<'a>(doc: &'a BaseDocument, selector: &str) -> &'a TextLayout {
    let id = doc.query_selector(selector).unwrap().unwrap();
    doc.get_node(id)
        .unwrap()
        .element_data()
        .unwrap()
        .inline_layout_data
        .as_ref()
        .unwrap()
}

/// Renders `html` with `#test` scrolled horizontally by `scroll_x` CSS pixels.
fn render(html: &str, scroll_x: f64) -> Vec<u8> {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            viewport: Some(Viewport::new(120, 40, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    if scroll_x != 0.0 {
        let id = doc.query_selector("#test").unwrap().unwrap();
        doc.scroll_by(id, scroll_x, 0.0, ScrollBehavior::Instant);
        doc.resolve(0.0);
    }
    render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| paint_scene(scene, doc.as_mut(), 1.0, 120, 40, 0, 0),
        120,
        40,
    )
}

#[test]
fn ellipsis_truncates_an_overflowing_line() {
    let harness = Harness::from_html(&html("ellipsis", LONG));
    let doc = harness.base();
    let overflow = inline(&doc, "#test").overflow.as_deref().unwrap();
    assert_eq!(overflow.lines.len(), 1);
    assert_eq!(overflow.lines[0].line_index, 0);
    assert!(overflow.marker.advance > 0.0);
}

#[test]
fn string_marker_is_shaped() {
    let ellipsis = Harness::from_html(&html("ellipsis", LONG));
    let string = Harness::from_html(&html("\"[...]\"", LONG));
    let ellipsis_doc = ellipsis.base();
    let string_doc = string.base();
    let ellipsis_marker = &inline(&ellipsis_doc, "#test")
        .overflow
        .as_deref()
        .unwrap()
        .marker;
    let string_marker = &inline(&string_doc, "#test")
        .overflow
        .as_deref()
        .unwrap()
        .marker;
    assert!(string_marker.advance > ellipsis_marker.advance);
}

#[test]
fn no_marker_when_the_text_fits() {
    let harness = Harness::from_html(&html("ellipsis", "Fits"));
    assert!(inline(&harness.base(), "#test").overflow.is_none());
}

#[test]
fn no_marker_for_clip_or_visible_overflow() {
    let clip = Harness::from_html(&html("clip", LONG));
    assert!(inline(&clip.base(), "#test").overflow.is_none());

    let visible = Harness::from_html(&format!(
        "<div id='test' style='width:100px; white-space:nowrap; text-overflow:ellipsis'>{LONG}</div>"
    ));
    assert!(inline(&visible.base(), "#test").overflow.is_none());
}

#[test]
fn no_marker_on_the_anonymous_block_of_a_flex_container() {
    // `text-overflow` applies to block containers: the text of a flex container is wrapped in
    // an anonymous flex item, which does not take it from the flex container.
    let harness = Harness::from_html(&format!(
        "<div id='test' style='display:flex; width:100px; white-space:nowrap; overflow:hidden; \
         text-overflow:ellipsis'>{LONG}</div>"
    ));
    let doc = harness.base();
    let id = doc.query_selector("#test").unwrap().unwrap();
    let children = doc.get_node(id).unwrap().layout_children.borrow().clone();
    let mut inline_roots = 0;
    for child in children.unwrap_or_default() {
        if let Some(text) = doc
            .get_node(child)
            .unwrap()
            .element_data()
            .and_then(|el| el.inline_layout_data.as_ref())
        {
            inline_roots += 1;
            assert!(text.overflow.is_none());
        }
    }
    assert_eq!(inline_roots, 1);
}

#[test]
fn ellipsis_paints_only_where_text_is_cut() {
    // Overflowing: the marker replaces the clipped glyphs.
    assert_ne!(
        render(&html("ellipsis", LONG), 0.0),
        render(&html("clip", LONG), 0.0)
    );
    // Fitting: nothing is cut, so `ellipsis` paints exactly like `clip`.
    assert_eq!(
        render(&html("ellipsis", "Fits"), 0.0),
        render(&html("clip", "Fits"), 0.0)
    );
}

#[test]
fn no_marker_once_scrolled_to_the_end() {
    // Scrolled part way, the end of the line is still out of view: the marker is drawn.
    assert_ne!(
        render(&html("ellipsis", LONG), 40.0),
        render(&html("clip", LONG), 40.0)
    );
    // Scrolled to the end, nothing is hidden on the end side any more (css-overflow §5.2).
    assert_eq!(
        render(&html("ellipsis", LONG), 1000.0),
        render(&html("clip", LONG), 1000.0)
    );
}
