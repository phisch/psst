use gpui::{font, point, prelude::*, px, BoxShadow, Div, Font, FontFallbacks, StyleRefinement};
use theme::Style;

pub(crate) fn font_family(name: &str) -> Font {
    let mut names = name.split(',').map(str::trim).filter(|n| !n.is_empty());
    let mut font = font(names.next().unwrap_or(name).to_owned());
    font.fallbacks = Some(FontFallbacks::from_fonts(
        names.map(str::to_owned).collect(),
    ));
    font
}

pub(crate) fn apply(d: Div, s: &Style) -> Div {
    let d = match s.border_width.round() as i32 {
        0 => d,
        1 => d.border_1(),
        2 => d.border_2(),
        _ => d.border_3(),
    };
    let mut d = d
        .bg(s.background)
        .border_color(s.border)
        .rounded(px(s.radius))
        .px(px(s.padding.x))
        .py(px(s.padding.y))
        .gap(px(s.gap))
        .text_size(px(s.size))
        .text_color(s.text);
    d = match &s.font {
        theme::FontFamily::Default => d,
        theme::FontFamily::Named(name) => d.font(font_family(name)),
    };
    if let Some(shadow) = &s.shadow {
        d = d.shadow(vec![BoxShadow {
            color: shadow.color,
            offset: point(px(shadow.offset_x), px(shadow.offset_y)),
            blur_radius: px(shadow.blur),
            spread_radius: px(shadow.spread),
        }]);
    }
    d
}

pub(crate) fn colors(s: &Style) -> impl Fn(StyleRefinement) -> StyleRefinement + 'static {
    let (bg, text, border) = (s.background, s.text, s.border);
    move |r| r.bg(bg).text_color(text).border_color(border)
}
