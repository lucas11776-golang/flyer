use std::time::Duration;

use sentry::{
    protocol::Event,
    ClientInitGuard,
    Level,
};

use crate::{
    error::Error, loggers::Logger, request::Request, response::Response,
};

pub struct Sentry {
    guard: ClientInitGuard,
}

impl Sentry {
    pub fn new(dsn: impl Into<String>, environment: impl Into<String>) -> Self {
        let guard = sentry::init((
            dsn.into(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(environment.into().into()),
                send_default_pii: true,
                ..Default::default()
            },
        ));

        Self { guard }
    }
}

impl Logger for Sentry {
    async fn call(&self, error: Error, req: Request, res: Response) {
        sentry::configure_scope(|scope| {
            scope.clear();

            scope.set_extra("request", req.into());
            scope.set_extra("response", res.into());
        });

        sentry::capture_event(Event {
            level: Level::Error,
            message: Some(format!("{}", error.error)),
            ..Default::default()
        });

        self.guard.flush(Some(Duration::from_secs(2)));
    }
}