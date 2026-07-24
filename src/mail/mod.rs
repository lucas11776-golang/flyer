use std::time::SystemTime;

use anyhow::Result;
use bytes::Bytes;
use lettre::message::MessageBuilder;
use lettre::message::{Mailbox as LettreMailBox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use once_cell::sync::OnceCell;

use crate::view::{ViewData, View};

pub(crate) static mut GLOBAL_MAILER: OnceCell<Box<SMTP>> = OnceCell::new();

pub(crate) struct SMTP {
    transport: SmtpTransport,
}

impl SMTP {
    pub fn new(host: String, port: u16, username: String, password: String, tls: bool) -> Result<SMTP> {
        let mut builder = SmtpTransport::relay(&host)
                .unwrap()
                .port(port)
                .credentials(Credentials::new(username, password));

        if !tls {
            builder = builder.tls(Tls::None);
        }

        return Ok(Self {
            transport: builder.build(),
        });
    }

    #[allow(static_mut_refs)]
    pub fn add(host: String, port: u16, username: String, password: String, tls: bool) -> Result<()> {
        unsafe { 
            return GLOBAL_MAILER
                .set(Box::new(Self::new(host, port, username, password, tls).unwrap()))
                .map_err(|_| anyhow::anyhow!("Global mailer already initialized"));
        }
    }
}

pub struct Mailbox {
    email: String,
    name: Option<String>
}

impl Mailbox {
    pub fn new(email: String, name: Option<String>) -> Self {
        return Self {
            email: email,
            name: name
        };
    }
}

pub struct Mail {
    builder: MessageBuilder,
    body: Option<Bytes>,
}

// TODO: Need to implement (attachment, attachments)
impl Mail {
    pub fn new() -> Self {
        return Self {
            builder: Message::builder(),
            body: None,
        };
    }

    pub fn from(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self.builder
            .clone()
            .from(LettreMailBox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn reply_to(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .reply_to(LettreMailBox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn sender(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .sender(LettreMailBox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn date(&mut self) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .date_now();

        return self;
    }

    pub fn date_now(&mut self, time: SystemTime) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .date(time);

        return self;
    }

    pub fn cc(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .cc(LettreMailBox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn bcc(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .bcc(LettreMailBox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn subject(&mut self, subject: String) -> &mut Self {
        self.builder = self.builder.clone().subject(subject);

        return self;
    }

    pub fn text(&mut self, text: Bytes) -> &mut Self {
        self.builder = self.builder
            .clone()
            .header(ContentType::TEXT_PLAIN);

        self.body = Some(text);

        return self;
    }

    pub fn html(&mut self, html: Bytes) -> &mut Self {
        self.builder = self.builder
            .clone()
            .header(ContentType::TEXT_HTML);

        self.body = Some(html);

        return self;
    }

    pub fn view(&mut self, path: impl Into<String>, template: impl Into<String>, data: Option<ViewData>) -> &mut Self {
        let view = View::render(path, template, data)
            .unwrap();

        return self.html(view);
    }

    #[allow(static_mut_refs)]
    pub async fn send_to_many(&mut self, to: Vec<Mailbox>) -> Result<()> {
        unsafe {
            let transport = &GLOBAL_MAILER
                    .get_mut()
                    .unwrap()
                    .transport;
            let body: Vec<u8> = Vec::from(self.body.clone().unwrap_or(Bytes::new()));

            for mailbox in to {
                let message =  self.builder
                    .clone()
                    .to(LettreMailBox::new(mailbox.name, mailbox.email.parse().unwrap()))
                    .body(body.clone())
                    .unwrap();

                transport.send(&message).unwrap();
            }

            return Ok(());
        }
    }

    pub async fn send(&mut self, email: String, name: Option<String>) -> Result<()> {
        return self.send_to_many(vec![Mailbox::new(email, name)]).await;
    }
}