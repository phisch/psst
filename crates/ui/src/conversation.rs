use std::io::{BufRead, Write};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{
    card_style, edges, field_input, icon_style, lock_icon, namespace, pill_button, text_button,
    MAX_LIFETIME,
};
use iced::widget::{button, column, container, row, svg, text, text_input, Space};
use iced::{Background, Border, Element, Length, Subscription, Task, Theme};
use iced_layershell::build_pattern::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use theme::theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Update {
    Start { title: String, message: String },
    Info { text: String },
    Error { text: String },
    Prompt { echo_on: bool, label: String },
    Done { success: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Reply {
    Response { secret: String },
    Cancel,
}

static FIELD_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(|| iced::widget::Id::new("psst-conversation-field"));

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Command(Update),
    Input(String),
    Submit,
    Cancel,
    Eof,
}

#[derive(Clone)]
struct Prompt {
    echo_on: bool,
    label: String,
}

#[derive(Default)]
struct State {
    title: String,
    body: String,
    info: Vec<String>,
    error: Option<String>,
    prompt: Option<Prompt>,
    input: Zeroizing<String>,
    done: bool,
}

pub fn run_conversation() {
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = done_rx.recv_timeout(MAX_LIFETIME)
        {
            eprintln!("psst: polkit dialog timed out; releasing keyboard grab");
            std::process::exit(2);
        }
    });

    let _ = application(boot, namespace, update, view)
        .style(style)
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                layer: Layer::Overlay,
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                exclusive_zone: -1,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                size: None,
                ..Default::default()
            },
            ..Default::default()
        })
        .run();

    let _ = done_tx.send(());
}

fn boot() -> (State, Task<Message>) {
    (State::default(), Task::none())
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    if state.done {
        return Task::none();
    }
    match message {
        Message::Command(command) => apply(state, command),
        Message::Input(value) => {
            state.input = Zeroizing::new(value);
            Task::none()
        }
        Message::Submit => {
            if state.prompt.take().is_some() {
                let secret = std::mem::take(&mut state.input);
                emit(&Reply::Response {
                    secret: secret.to_string(),
                });
                state.error = None;
            }
            Task::none()
        }
        Message::Cancel => {
            emit(&Reply::Cancel);
            finish(state)
        }
        Message::Eof => finish(state),
        _ => Task::none(),
    }
}

fn apply(state: &mut State, command: Update) -> Task<Message> {
    match command {
        Update::Start { title, message } => {
            state.title = title;
            state.body = message;
            Task::none()
        }
        Update::Info { text } => {
            state.info.push(text);
            Task::none()
        }
        Update::Error { text } => {
            state.error = Some(text);
            Task::none()
        }
        Update::Prompt { echo_on, label } => {
            state.prompt = Some(Prompt { echo_on, label });
            state.input = Zeroizing::new(String::new());
            iced::widget::operation::focus(FIELD_ID.clone())
        }
        Update::Done { .. } => finish(state),
    }
}

fn finish(state: &mut State) -> Task<Message> {
    state.done = true;
    iced::exit()
}

fn emit(message: &Reply) {
    if let Ok(line) = serde_json::to_string(message) {
        let line = Zeroizing::new(line);
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{}", &*line);
        let _ = out.flush();
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let spacing = theme().window.spacing;
    let mut card = column![header(&state.title)]
        .spacing(spacing)
        .width(Length::Fill);

    if !state.body.is_empty() || !state.info.is_empty() {
        let mut body = column![].spacing(6);
        if !state.body.is_empty() {
            body = body.push(line(&state.body));
        }
        for info in &state.info {
            body = body.push(line(info));
        }
        card = card.push(body);
    }

    if let Some(error) = &state.error {
        card = card.push(banner(error));
    }
    if let Some(prompt) = &state.prompt {
        card = card.push(field(prompt, &state.input));
    }
    card = card.push(footer(state.prompt.is_some()));

    let window = &theme().window;
    let card = container(card)
        .padding(edges(window.padding))
        .max_width(window.width)
        .width(Length::Fill)
        .style(card_style);

    container(card).center(Length::Fill).padding(64).into()
}

fn header(title: &str) -> Element<'_, Message> {
    let icon = &theme().title_icon;
    let tile = icon.size + icon.padding * 2.0;
    let glyph = container(
        svg(lock_icon())
            .width(Length::Fixed(icon.size))
            .height(Length::Fixed(icon.size))
            .style(|_theme, _status| svg::Style {
                color: Some(theme().title_icon.color),
            }),
    )
    .center(Length::Fixed(tile));

    row![
        container(glyph).style(icon_style),
        text(title.to_string())
            .size(theme().title.font.size)
            .font(theme().title.font.family)
            .color(theme().title.color)
    ]
    .spacing(14)
    .align_y(iced::Alignment::Center)
    .into()
}

fn line(value: &str) -> Element<'static, Message> {
    let style = &theme().description.value;
    text(value.to_string())
        .size(style.font.size)
        .font(style.font.family)
        .color(style.color)
        .width(Length::Fill)
        .into()
}

fn banner(message: &str) -> Element<'static, Message> {
    container(
        text(message.to_string())
            .size(theme().error.font.size)
            .color(theme().error.color),
    )
    .padding(edges(theme().error.padding))
    .width(Length::Fill)
    .style(|_theme| {
        let error = &theme().error;
        container::Style {
            background: Some(Background::Color(error.background)),
            border: Border {
                color: error.border.color,
                width: error.border.size,
                radius: error.radius.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn field<'a>(prompt: &Prompt, input: &'a str) -> Element<'a, Message> {
    let theme = theme();
    text_input(&prompt.label, input)
        .secure(!prompt.echo_on)
        .on_input(Message::Input)
        .on_submit(Message::Submit)
        .id(FIELD_ID.clone())
        .font(theme.field.font.family)
        .size(theme.field.font.size)
        .padding(edges(theme.field.padding))
        .width(Length::Fill)
        .style(field_input)
        .into()
}

fn footer(prompting: bool) -> Element<'static, Message> {
    let mut row = row![hints(prompting)]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);
    row = row.push(Space::new().width(Length::Fill));
    row = row.push(
        button(text("Cancel").size(theme().cancel.font.size))
            .on_press(Message::Cancel)
            .padding(edges(theme().cancel.padding))
            .style(text_button),
    );
    if prompting {
        row = row.push(
            button(text("Authenticate").size(theme().confirm.font.size))
                .on_press(Message::Submit)
                .padding(edges(theme().confirm.padding))
                .style(pill_button),
        );
    }
    row.into()
}

fn hints(prompting: bool) -> Element<'static, Message> {
    let items: &[(&str, &str)] = if prompting {
        &[("\u{21B5}", "authenticate"), ("Esc", "cancel")]
    } else {
        &[("Esc", "cancel")]
    };

    let key = &theme().hint.key;
    let word = &theme().hint.word;
    let mut line = row![]
        .spacing(theme().hint.spacing)
        .align_y(iced::Alignment::Center);
    for (i, (glyph, label)) in items.iter().enumerate() {
        if i > 0 {
            line = line.push(text("\u{00B7}").size(word.font.size).color(word.color));
        }
        line = line.push(
            text(glyph.to_string())
                .font(key.font.family)
                .size(key.font.size)
                .color(key.color),
        );
        line = line.push(
            text(label.to_string())
                .font(word.font.family)
                .size(word.font.size)
                .color(word.color),
        );
    }
    line.into()
}

fn style(_state: &State, _theme: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: theme().backdrop.color,
        text_color: theme().title.color,
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run(stdin_worker),
        iced::event::listen_with(handle_event),
    ])
}

fn handle_event(
    event: iced::Event,
    _status: iced::event::Status,
    _id: iced::window::Id,
) -> Option<Message> {
    use iced::keyboard::{key::Named, Event::KeyPressed, Key};
    match event {
        iced::Event::Keyboard(KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        }) => Some(Message::Cancel),
        iced::Event::Keyboard(KeyPressed {
            key: Key::Named(Named::Enter),
            ..
        }) => Some(Message::Submit),
        _ => None,
    }
}

fn stdin_worker() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(
        16,
        |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            use iced::futures::{SinkExt, StreamExt};

            let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<Option<String>>();
            std::thread::spawn(move || {
                for line in std::io::stdin().lock().lines() {
                    match line {
                        Ok(line) => {
                            if tx.unbounded_send(Some(line)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let _ = tx.unbounded_send(None);
            });

            while let Some(item) = rx.next().await {
                let message = match item {
                    Some(line) => match serde_json::from_str::<Update>(line.trim()) {
                        Ok(command) => Message::Command(command),
                        Err(_) => continue,
                    },
                    None => Message::Eof,
                };
                if output.send(message).await.is_err() {
                    break;
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_messages_round_trip_one_line() {
        let prompt = Update::Prompt {
            echo_on: false,
            label: "Password: ".into(),
        };
        let line = serde_json::to_string(&prompt).unwrap();
        assert!(!line.contains('\n'));
        assert!(matches!(
            serde_json::from_str::<Update>(&line).unwrap(),
            Update::Prompt { echo_on: false, .. }
        ));
    }

    #[test]
    fn dialog_response_serializes_secret_safely() {
        let line = serde_json::to_string(&Reply::Response {
            secret: "with \"quotes\"\nand newline".into(),
        })
        .unwrap();
        assert!(!line.contains('\n'));
        match serde_json::from_str::<Reply>(&line).unwrap() {
            Reply::Response { secret } => {
                assert_eq!(secret, "with \"quotes\"\nand newline")
            }
            _ => panic!("expected response"),
        }
    }
}
