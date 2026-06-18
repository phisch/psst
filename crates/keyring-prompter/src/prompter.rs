use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::message::Header;
use zbus::names::BusName;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::Connection;

use crate::prompt::{Cancel, PromptKind, PromptRequest, PromptResponse, Prompter as Ui};
use crate::secret_exchange::SecretExchange;

const CALLBACK_INTERFACE: &str = "org.gnome.keyring.internal.Prompter.Callback";

type SessionKey = (String, String);
type ActiveMap = Arc<Mutex<HashMap<SessionKey, Cancel>>>;

pub struct Service {
    ui: Arc<dyn Ui>,
    sessions: Mutex<HashMap<SessionKey, SecretExchange>>,
    active: ActiveMap,
}

impl Service {
    pub fn new(ui: Arc<dyn Ui>) -> Self {
        Service {
            ui,
            sessions: Mutex::new(HashMap::new()),
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[zbus::interface(name = "org.gnome.keyring.internal.Prompter")]
impl Service {
    #[zbus(name = "BeginPrompting")]
    async fn begin_prompting(
        &self,
        callback: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) {
        let Some(key) = session_key(&header, &callback) else {
            return;
        };
        if !authorized(connection, &key.0).await {
            eprintln!(
                "hush-keyring: refusing prompt from unauthorized caller {}",
                key.0
            );
            return;
        }

        let exchange = SecretExchange::generate();
        let message = exchange.public_message();
        self.sessions.lock().unwrap().insert(key.clone(), exchange);

        let connection = connection.clone();
        tokio::spawn(async move {
            prompt_ready(&connection, &key, "", &message, HashMap::new()).await;
        });
    }

    #[zbus(name = "PerformPrompt")]
    async fn perform_prompt(
        &self,
        callback: OwnedObjectPath,
        r#type: String,
        properties: HashMap<String, OwnedValue>,
        exchange: String,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) {
        let Some(key) = session_key(&header, &callback) else {
            return;
        };
        if !authorized(connection, &key.0).await {
            return;
        }

        let Some(session) = self.sessions.lock().unwrap().get(&key).cloned() else {
            return;
        };
        let Some(transport_key) = session.transport_key(&exchange) else {
            return;
        };

        let want_password = r#type == "password";
        let request = build_request(&properties, want_password);

        let cancel = Cancel::default();
        self.active.lock().unwrap().insert(key.clone(), cancel.clone());

        let ui = self.ui.clone();
        let active = self.active.clone();
        let connection = connection.clone();
        tokio::spawn(async move {
            let response = tokio::task::spawn_blocking(move || ui.prompt(request, &cancel))
                .await
                .unwrap_or(PromptResponse::Dismissed);
            active.lock().unwrap().remove(&key);

            let (reply, message, properties) = match response {
                PromptResponse::Password(secret) => {
                    // gnome-keyring shows a "store unencrypted?" confirmation
                    // unless the prompter reports a non-zero password strength.
                    let strength = if secret.is_empty() { 0 } else { 1 };
                    (
                        "yes",
                        session.encrypted_message(&transport_key, secret.as_bytes()),
                        password_strength(strength),
                    )
                }
                PromptResponse::Confirmed => ("yes", session.public_message(), HashMap::new()),
                PromptResponse::Dismissed => ("no", session.public_message(), HashMap::new()),
            };

            prompt_ready(&connection, &key, reply, &message, properties).await;
        });
    }

    #[zbus(name = "StopPrompting")]
    async fn stop_prompting(
        &self,
        callback: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) {
        let Some(key) = session_key(&header, &callback) else {
            return;
        };
        self.sessions.lock().unwrap().remove(&key);
        if let Some(cancel) = self.active.lock().unwrap().remove(&key) {
            cancel.trigger();
        }

        let connection = connection.clone();
        tokio::spawn(async move {
            prompt_done(&connection, &key).await;
        });
    }
}

async fn authorized(connection: &Connection, sender: &str) -> bool {
    if std::env::var_os("HUSH_ALLOW_ANY_CALLER").is_some() {
        return true;
    }
    let Ok(bus_name) = BusName::try_from(sender) else {
        return false;
    };
    let Ok(proxy) = zbus::fdo::DBusProxy::new(connection).await else {
        return false;
    };
    let Ok(pid) = proxy.get_connection_unix_process_id(bus_name).await else {
        return false;
    };
    match std::fs::read_link(format!("/proc/{pid}/exe")) {
        Ok(path) => path.file_name().is_some_and(|name| name == "gnome-keyring-daemon"),
        Err(_) => false,
    }
}

fn session_key(header: &Header<'_>, callback: &OwnedObjectPath) -> Option<SessionKey> {
    let sender = header.sender()?.to_string();
    Some((sender, callback.as_str().to_string()))
}

fn build_request(properties: &HashMap<String, OwnedValue>, want_password: bool) -> PromptRequest {
    let string = |key: &str| {
        properties
            .get(key)
            .and_then(|value| String::try_from(value.clone()).ok())
            .filter(|value| !value.is_empty())
    };
    let flag = |key: &str| {
        properties
            .get(key)
            .and_then(|value| bool::try_from(value.clone()).ok())
            .unwrap_or(false)
    };

    PromptRequest {
        kind: if want_password {
            PromptKind::Password {
                confirm: flag("password-new"),
            }
        } else {
            PromptKind::Confirm
        },
        title: string("message").or_else(|| string("title")),
        description: string("description"),
        warning: string("warning"),
        continue_label: string("continue-label"),
        cancel_label: string("cancel-label"),
    }
}

fn password_strength(strength: i32) -> HashMap<String, OwnedValue> {
    // gcr's prompter wire nests each a{sv} value in a second variant: its client
    // unboxes twice, so a plain `variant(i)` reads back as 0. We must send
    // `variant(variant(i))` to match.
    let nested = Value::Value(Box::new(Value::I32(strength)));
    let value = OwnedValue::try_from(nested).expect("nested variant is valid");
    HashMap::from([("password-strength".to_string(), value)])
}

async fn prompt_ready(
    connection: &Connection,
    key: &SessionKey,
    reply: &str,
    exchange: &str,
    properties: HashMap<String, OwnedValue>,
) {
    let (sender, path) = key;
    if let Err(error) = connection
        .call_method(
            Some(sender.as_str()),
            path.as_str(),
            Some(CALLBACK_INTERFACE),
            "PromptReady",
            &(reply, properties, exchange),
        )
        .await
    {
        eprintln!("hush-keyring: PromptReady call failed: {error}");
    }
}

async fn prompt_done(connection: &Connection, key: &SessionKey) {
    let (sender, path) = key;
    if let Err(error) = connection
        .call_method(
            Some(sender.as_str()),
            path.as_str(),
            Some(CALLBACK_INTERFACE),
            "PromptDone",
            &(),
        )
        .await
    {
        eprintln!("hush-keyring: PromptDone call failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_strength_is_a_nested_int32_variant() {
        let properties = password_strength(1);
        let value: Value = properties.get("password-strength").unwrap().clone().into();
        assert_eq!(value, Value::Value(Box::new(Value::I32(1))));
    }
}
