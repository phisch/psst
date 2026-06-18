use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zeroize::Zeroizing;

#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub(crate) fn trigger(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub enum PromptKind {
    Password { confirm: bool },
    Confirm,
}

pub struct PromptRequest {
    pub kind: PromptKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub warning: Option<String>,
    pub continue_label: Option<String>,
    pub cancel_label: Option<String>,
}

pub enum PromptResponse {
    Password(Zeroizing<String>),
    Confirmed,
    Dismissed,
}

pub trait Prompter: Send + Sync + 'static {
    fn prompt(&self, request: PromptRequest, cancel: &Cancel) -> PromptResponse;
}
