use anyhow::Result;
use lettre::message::MessageBuilder;
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use once_cell::sync::OnceCell;

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

pub struct Mail {
    builder: MessageBuilder,
    body: Option<String>,
}

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
            .from(Mailbox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn reply_to(&mut self, email: String, name: Option<String>) -> &mut Self {
        self.builder = self
            .builder
            .clone()
            .reply_to(Mailbox::new(name, email.parse().unwrap()));

        return self;
    }

    pub fn subject(&mut self, subject: String) -> &mut Self {
        self.builder = self.builder.clone().subject(subject);

        return self;
    }

    pub fn html(&mut self, html: String) -> &mut Self {
        self.builder = self.builder
            .clone()
            .header(ContentType::TEXT_HTML);

        self.body = Some(html);

        return self;
    }

    #[allow(static_mut_refs)]
    pub async fn send(&mut self, to: Vec<String>) -> Result<()> {
        unsafe {
            let transport = &GLOBAL_MAILER
                    .get_mut()
                    .unwrap()
                    .transport;

            for email in to {
                let message =  self.builder
                    .clone()
                    .to(Mailbox::new(None, email.parse().unwrap()))
                    .body(self.body.clone().unwrap_or(String::new()))
                    .unwrap();

                transport.send(&message).unwrap();
            }

            return Ok(());
        }
    }
}
