use std::sync::OnceLock;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use lettre::message::header::ContentType;
use lettre::message::{Mailbox as LettreMailBox, MessageBuilder};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};

use crate::view::{View, ViewData};

static GLOBAL_MAILER: OnceLock<SmtpTransport> = OnceLock::new();

pub struct SMTP;

impl SMTP {
    pub fn init(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
        tls: bool
    ) -> Result<()> {
        let mut builder = SmtpTransport::relay(&host.into())?
            .port(port)
            .credentials(Credentials::new(username.into(), password.into()));

        if !tls {
            builder = builder.tls(Tls::None);
        }

        GLOBAL_MAILER
            .set(builder.build())
            .map_err(|_| anyhow!("Global mailer already initialized"))
    }

    #[inline]
    pub fn global() -> Result<&'static SmtpTransport> {
        GLOBAL_MAILER
            .get()
            .ok_or_else(|| anyhow!("Mailer has not been initialized. Call SMTP::init first."))
    }
}

#[derive(Clone, Debug)]
pub struct Mailbox {
    pub email: String,
    pub name: Option<String>,
}

impl Mailbox {
    pub fn new(email: impl Into<String>, name: Option<&str>) -> Self {
        Self {
            email: email.into(),
            name: name.map(Into::into),
        }
    }

    fn to_lettre(&self) -> Result<LettreMailBox> {
        let address = self.email.parse()?;
        Ok(LettreMailBox::new(self.name.clone(), address))
    }
}

pub struct Mail {
    builder: MessageBuilder,
    body: Vec<u8>,
}

impl Default for Mail {
    fn default() -> Self {
        Self::new()
    }
}

impl Mail {
    pub fn new() -> Self {
        Self {
            builder: Message::builder(),
            body: Vec::new(),
        }
    }

    pub fn from(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        let mb = LettreMailBox::new(name.map(Into::into), email.into().parse().unwrap());
        self.builder = self.builder.from(mb);
        self
    }

    pub fn reply_to(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        let mb = LettreMailBox::new(name.map(Into::into), email.into().parse().unwrap());
        self.builder = self.builder.reply_to(mb);
        self
    }

    pub fn sender(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        let mb = LettreMailBox::new(name.map(Into::into), email.into().parse().unwrap());
        self.builder = self.builder.sender(mb);
        self
    }

    pub fn cc(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        let mb = LettreMailBox::new(name.map(Into::into), email.into().parse().unwrap());
        self.builder = self.builder.cc(mb);
        self
    }

    pub fn bcc(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        let mb = LettreMailBox::new(name.map(Into::into), email.into().parse().unwrap());
        self.builder = self.builder.bcc(mb);
        self
    }

    pub fn date(mut self) -> Self {
        self.builder = self.builder.date_now();
        self
    }

    pub fn date_at(mut self, time: SystemTime) -> Self {
        self.builder = self.builder.date(time);
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.builder = self.builder.subject(subject);
        self
    }

    pub fn text(mut self, text: impl Into<Vec<u8>>) -> Self {
        self.builder = self.builder.header(ContentType::TEXT_PLAIN);
        self.body = text.into();
        self
    }

    pub fn html(mut self, html: impl Into<Vec<u8>>) -> Self {
        self.builder = self.builder.header(ContentType::TEXT_HTML);
        self.body = html.into();
        self
    }

    pub fn view(self, path: impl Into<String>, template: impl Into<String>, data: Option<ViewData>) -> Self {
        let view_bytes = View::render(path, template, data).unwrap();
        self.html(view_bytes)
    }

    pub fn send_to_many(&self, recipients: &[Mailbox]) -> Result<()> {
        let transport = SMTP::global()?;

        for recipient in recipients {
            let lettre_mb = recipient.to_lettre()?;
            let message = self
                .builder
                .clone()
                .to(lettre_mb)
                .body(self.body.clone())?;

            transport.send(&message)?;
        }

        Ok(())
    }

    pub fn send(&self, email: impl Into<String>, name: Option<&str>) -> Result<()> {
        self.send_to_many(&[Mailbox::new(email, name)])
    }
}