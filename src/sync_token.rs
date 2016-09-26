use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TokenSource {
    token: Arc<Token>,
}

pub struct Token {
    is_triggered: AtomicBool
}

impl TokenSource {
    pub fn new() -> TokenSource {
        TokenSource {
            token: Arc::new(Token { is_triggered: AtomicBool::new(false) } )
        }
    }

    pub fn get_token(&self) -> Arc<Token> {
        self.token.clone()
    }

    pub fn trigger(&self) {
        self.token.is_triggered.store(true, Ordering::Relaxed)
    }
}

impl Token {
    pub fn is_triggered(&self) -> bool {
        self.is_triggered.load(Ordering::Relaxed)
    }
}
