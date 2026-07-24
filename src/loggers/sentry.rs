use std::collections::BTreeMap;

use sentry::{
    ClientInitGuard,
    Level,
    Scope,
    protocol::Event
};

use crate::{
    error::logger::{Logger, PanicErrorInfo},
    request::Request,
    response::Response
};

pub struct Sentry {
    environment: String,
    guard: ClientInitGuard,
}

impl Sentry {
    pub fn new(api_key: impl Into<String>, environment: impl Into<String>) -> Self {
        return Self {
            environment: environment.into(),
            guard: sentry::init((api_key.into(), sentry::ClientOptions {
                release: sentry::release_name!(),
                send_default_pii: true,
                ..Default::default()
            }))
        };
    }
}

impl Logger for Sentry {
    async fn call(&self, _info: PanicErrorInfo, _req: Request, _res: Response) -> () {
        let mut scope = Scope::default();
        scope.set_extra("scope_level_extra", "This comes from the scope object".into());
        scope.set_tag("module", "billing_worker");
        scope.set_level(Some(Level::Error));

        let mut event_extras = BTreeMap::new();
        event_extras.insert("event_level_extra".to_string(), "This comes directly from the Event struct".into());
        event_extras.insert("retry_attempts".to_string(), 3.into());

        let event = Event {
            message: Some("Database handshake timeout".to_string()),
            extra: event_extras,
            environment: Some(self.environment.clone().into()),
            ..Default::default()
        };

        self.guard.capture_event(event, Some(&scope));
    }
}
