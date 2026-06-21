use std::path::PathBuf;
use std::sync::OnceLock;

use gpui::{Hsla, Rgba};
use kdl::{KdlDocument, KdlNode, KdlValue};

const LOCK_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px" fill="#e3e3e3"><path d="M226.67-80q-27.5 0-47.09-19.58Q160-119.17 160-146.67v-422.66q0-27.5 19.58-47.09Q199.17-636 226.67-636h60v-90.67q0-80.23 56.57-136.78T480.07-920q80.26 0 136.76 56.55 56.5 56.55 56.5 136.78V-636h60q27.5 0 47.09 19.58Q800-596.83 800-569.33v422.66q0 27.5-19.58 47.09Q760.83-80 733.33-80H226.67Zm0-66.67h506.66v-422.66H226.67v422.66Zm308.5-155.85Q558-325.04 558-356.67q0-31-22.95-55.16Q512.11-436 479.89-436t-55.06 24.17Q402-387.67 402-356.33q0 31.33 22.95 53.83 22.94 22.5 55.16 22.5t55.06-22.52ZM353.33-636h253.34v-90.67q0-52.77-36.92-89.72-36.93-36.94-89.67-36.94-52.75 0-89.75 36.94-37 36.95-37 89.72V-636ZM226.67-146.67v-422.66 422.66Z"/></svg>"##;

const EYE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" height="20px" viewBox="0 -960 960 960" width="20px" fill="#e3e3e3"><path d="M599-361q49-49 49-119t-49-119q-49-49-119-49t-119 49q-49 49-49 119t49 119q49 49 119 49t119-49Zm-187-51q-28-28-28-68t28-68q28-28 68-28t68 28q28 28 28 68t-28 68q-28 28-68 28t-68-28ZM220-270.5Q103-349 48-480q55-131 172-209.5T480-768q143 0 260 78.5T912-480q-55 131-172 209.5T480-192q-143 0-260-78.5ZM480-480Zm207 158q95-58 146-158-51-100-146-158t-207-58q-112 0-207 58T127-480q51 100 146 158t207 58q112 0 207-58Z"/></svg>"##;

const EYE_OFF_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" height="20px" viewBox="0 -960 960 960" width="20px" fill="#e3e3e3"><path d="m637-425-62-62q4-38-23-65.5T487-576l-62-62q13-5 27-7.5t28-2.5q70 0 119 49t49 119q0 14-2.5 28t-8.5 27Zm133 133-52-52q36-28 65.5-61.5T833-480q-49-101-144.5-158.5T480-696q-26 0-51 3t-49 10l-58-58q38-15 77.5-21t80.5-6q143 0 261.5 77.5T912-480q-22 57-58.5 103.5T770-292Zm-2 202L638-220q-38 14-77.5 21t-80.5 7q-143 0-261.5-77.5T48-480q22-57 58-104t84-85L90-769l51-51 678 679-51 51ZM241-617q-35 28-65 61.5T127-480q49 101 144.5 158.5T480-264q26 0 51-3.5t50-9.5l-45-45q-14 5-28 7.5t-28 2.5q-70 0-119-49t-49-119q0-14 3.5-28t6.5-28l-81-81Zm287 89Zm-96 96Z"/></svg>"##;

#[derive(Clone, Debug, PartialEq)]
pub enum FontFamily {
    Default,
    Monospace,
    Named(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Padding {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: Hsla,
    pub blur: f32,
    pub spread: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub background: Hsla,
    pub text: Hsla,
    pub placeholder: Hsla,
    pub selection: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub radius: f32,
    pub padding: Padding,
    pub gap: f32,
    pub font: FontFamily,
    pub size: f32,
    pub shadow: Option<Shadow>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background: rgb(0x00000000),
            text: rgb(0xf0f0f2ff),
            placeholder: rgb(0x00000000),
            selection: rgb(0x00000000),
            border: rgb(0x00000000),
            border_width: 0.0,
            radius: 0.0,
            padding: Padding { x: 0.0, y: 0.0 },
            gap: 0.0,
            font: FontFamily::Default,
            size: 13.0,
            shadow: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub base: Style,
    pub hover: Style,
    pub active: Style,
    pub focus: Style,
    pub checked: Style,
}

impl Element {
    fn new(base: Style) -> Self {
        Element {
            hover: base.clone(),
            active: base.clone(),
            focus: base.clone(),
            checked: base.clone(),
            base,
        }
    }
}

pub struct Icons {
    pub lock: String,
    pub eye: String,
    pub eye_off: String,
}

pub struct Theme {
    pub backdrop: Element,
    pub window: Element,
    pub title: Element,
    pub icon: Element,
    pub description_label: Element,
    pub description_value: Element,
    pub error: Element,
    pub field: Element,
    pub reveal: Element,
    pub strength: Element,
    pub strength_weak: Element,
    pub strength_medium: Element,
    pub strength_strong: Element,
    pub checkbox: Element,
    pub confirm: Element,
    pub cancel: Element,
    pub hint_key: Element,
    pub hint_word: Element,
    pub icons: Icons,
}

impl Default for Theme {
    fn default() -> Self {
        let blank = Element::new(Style::default());
        let mut theme = Theme {
            backdrop: blank.clone(),
            window: blank.clone(),
            title: blank.clone(),
            icon: blank.clone(),
            description_label: blank.clone(),
            description_value: blank.clone(),
            error: blank.clone(),
            field: blank.clone(),
            reveal: blank.clone(),
            strength: blank.clone(),
            strength_weak: blank.clone(),
            strength_medium: blank.clone(),
            strength_strong: blank.clone(),
            checkbox: blank.clone(),
            confirm: blank.clone(),
            cancel: blank.clone(),
            hint_key: blank.clone(),
            hint_word: blank,
            icons: Icons {
                lock: LOCK_SVG.into(),
                eye: EYE_SVG.into(),
                eye_off: EYE_OFF_SVG.into(),
            },
        };
        let doc = KdlDocument::parse(include_str!("default-theme.kdl"))
            .expect("bundled default theme is valid");
        read_theme(&doc, &mut theme, true);
        theme
    }
}

static THEME: OnceLock<Theme> = OnceLock::new();

pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::load)
}

impl Theme {
    fn load() -> Theme {
        let mut theme = Theme::default();
        let Some(text) = config_path().and_then(|path| std::fs::read_to_string(path).ok()) else {
            return theme;
        };
        match KdlDocument::parse(&text) {
            Ok(doc) => read_theme(&doc, &mut theme, false),
            Err(error) => eprintln!("psst: ignoring invalid theme: {error}"),
        }
        theme
    }
}

#[derive(Clone, Default)]
struct Inherited {
    font: Option<FontFamily>,
    size: Option<f32>,
    text: Option<Hsla>,
}

fn read_theme(doc: &KdlDocument, theme: &mut Theme, establish: bool) {
    for node in doc.nodes() {
        overlay(node, &Inherited::default(), theme, establish);
    }
}

fn overlay(node: &KdlNode, inherited: &Inherited, theme: &mut Theme, establish: bool) {
    let mut here = inherited.clone();
    if let Some(name) = string(child(node, "font")) {
        here.font = Some(parse_font(name));
    }
    if let Some(size) = num(child(node, "size")) {
        here.size = Some(size);
    }
    if let Some(text) = string(child(node, "text")).and_then(parse_hex) {
        here.text = Some(text);
    }

    if let Some(element) = element_mut(theme, node.name().value()) {
        if let Some(font) = &here.font {
            element.base.font = font.clone();
        }
        if let Some(size) = here.size {
            element.base.size = size;
        }
        if let Some(text) = here.text {
            element.base.text = text;
        }
        read_style(node, &mut element.base);
        if establish {
            element.hover = element.base.clone();
            element.active = element.base.clone();
            element.focus = element.base.clone();
            element.checked = element.base.clone();
        }
        for (name, state) in [
            ("hover", &mut element.hover),
            ("active", &mut element.active),
            ("focus", &mut element.focus),
            ("checked", &mut element.checked),
        ] {
            if let Some(block) = child(node, name) {
                read_style(block, state);
            }
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            overlay(child, &here, theme, establish);
        }
    }
}

fn read_style(node: &KdlNode, style: &mut Style) {
    set_color(node, "background", &mut style.background);
    set_color(node, "text", &mut style.text);
    set_color(node, "placeholder", &mut style.placeholder);
    set_color(node, "selection", &mut style.selection);
    set_color(node, "border", &mut style.border);
    set_num(node, "border-width", &mut style.border_width);
    set_num(node, "radius", &mut style.radius);
    set_num(node, "gap", &mut style.gap);
    set_num(node, "size", &mut style.size);
    if let Some(name) = string(child(node, "font")) {
        style.font = parse_font(name);
    }
    if let Some(p) = child(node, "padding") {
        set_num(p, "x", &mut style.padding.x);
        set_num(p, "y", &mut style.padding.y);
    }
    if let Some(s) = child(node, "shadow") {
        let mut shadow = style.shadow.unwrap_or(Shadow {
            color: rgb(0x00000000),
            blur: 0.0,
            spread: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        });
        set_color(s, "color", &mut shadow.color);
        set_num(s, "blur", &mut shadow.blur);
        set_num(s, "spread", &mut shadow.spread);
        set_num(s, "offset-x", &mut shadow.offset_x);
        set_num(s, "offset-y", &mut shadow.offset_y);
        style.shadow = Some(shadow);
    }
}

fn element_mut<'a>(theme: &'a mut Theme, name: &str) -> Option<&'a mut Element> {
    Some(match name {
        "backdrop" => &mut theme.backdrop,
        "window" => &mut theme.window,
        "title" => &mut theme.title,
        "icon" => &mut theme.icon,
        "description-label" => &mut theme.description_label,
        "description-value" => &mut theme.description_value,
        "error" => &mut theme.error,
        "field" => &mut theme.field,
        "reveal" => &mut theme.reveal,
        "strength" => &mut theme.strength,
        "strength-weak" => &mut theme.strength_weak,
        "strength-medium" => &mut theme.strength_medium,
        "strength-strong" => &mut theme.strength_strong,
        "checkbox" => &mut theme.checkbox,
        "confirm" => &mut theme.confirm,
        "cancel" => &mut theme.cancel,
        "hint-key" => &mut theme.hint_key,
        "hint-word" => &mut theme.hint_word,
        _ => return None,
    })
}

fn rgb(v: u32) -> Hsla {
    Rgba {
        r: ((v >> 24) & 0xff) as f32 / 255.0,
        g: ((v >> 16) & 0xff) as f32 / 255.0,
        b: ((v >> 8) & 0xff) as f32 / 255.0,
        a: (v & 0xff) as f32 / 255.0,
    }
    .into()
}

fn child<'a>(parent: &'a KdlNode, name: &str) -> Option<&'a KdlNode> {
    parent
        .children()?
        .nodes()
        .iter()
        .find(|node| node.name().value() == name)
}

fn arg(node: &KdlNode) -> Option<&KdlValue> {
    node.entries()
        .iter()
        .find(|entry| entry.name().is_none())
        .map(|entry| entry.value())
}

fn string(node: Option<&KdlNode>) -> Option<&str> {
    arg(node?).and_then(KdlValue::as_string)
}

fn num(node: Option<&KdlNode>) -> Option<f32> {
    let value = arg(node?)?;
    value
        .as_float()
        .map(|n| n as f32)
        .or_else(|| value.as_integer().map(|n| n as f32))
}

fn set_color(parent: &KdlNode, name: &str, target: &mut Hsla) {
    if let Some(color) = string(child(parent, name)).and_then(parse_hex) {
        *target = color;
    }
}

fn set_num(parent: &KdlNode, name: &str, target: &mut f32) {
    if let Some(number) = num(child(parent, name)) {
        *target = number;
    }
}

fn parse_font(name: &str) -> FontFamily {
    match name.trim().to_ascii_lowercase().as_str() {
        "monospace" | "mono" => FontFamily::Monospace,
        "" | "default" | "sans" | "sans-serif" => FontFamily::Default,
        _ => FontFamily::Named(name.trim().to_string()),
    }
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("psst").join("theme.kdl"))
}

fn parse_hex(text: &str) -> Option<Hsla> {
    let text = text.strip_prefix('#').unwrap_or(text).trim();
    let bytes = match text.len() {
        3 | 4 => hex_bytes(&text.chars().flat_map(|c| [c, c]).collect::<String>())?,
        6 | 8 => hex_bytes(text)?,
        _ => return None,
    };
    let a = bytes.get(3).copied().unwrap_or(255);
    Some(
        Rgba {
            r: bytes[0] as f32 / 255.0,
            g: bytes[1] as f32 / 255.0,
            b: bytes[2] as f32 / 255.0,
            a: a as f32 / 255.0,
        }
        .into(),
    )
}

fn hex_bytes(text: &str) -> Option<Vec<u8>> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Theme {
        let mut theme = Theme::default();
        read_theme(
            &KdlDocument::parse(text).expect("parses"),
            &mut theme,
            false,
        );
        theme
    }

    #[test]
    fn overrides_only_named_properties() {
        let theme = read(r##"field { background "#111111" }"##);
        assert_eq!(theme.field.base.background, parse_hex("#111111").unwrap());
        assert_eq!(theme.field.base.radius, Theme::default().field.base.radius);
    }

    #[test]
    fn nesting_cascades_font_text_size_but_not_background() {
        let theme = read(
            r##"window { background "#111"; font "serif"; text "#abcdef"; title { size 30 } }"##,
        );
        assert_eq!(theme.title.base.font, FontFamily::Named("serif".into()));
        assert_eq!(theme.title.base.text, parse_hex("#abcdef").unwrap());
        assert_eq!(theme.title.base.size, 30.0);
        assert_eq!(
            theme.title.base.background,
            Theme::default().title.base.background
        );
    }

    #[test]
    fn states_and_shadow_parse() {
        let theme = read(
            r##"confirm { hover { background "#abcdef" }; shadow { blur 12; spread 2; offset-x 1; offset-y 4 } }"##,
        );
        assert_eq!(
            theme.confirm.hover.background,
            parse_hex("#abcdef").unwrap()
        );
        let shadow = theme.confirm.base.shadow.unwrap();
        assert_eq!((shadow.blur, shadow.spread), (12.0, 2.0));
        assert_eq!((shadow.offset_x, shadow.offset_y), (1.0, 4.0));
    }
}
