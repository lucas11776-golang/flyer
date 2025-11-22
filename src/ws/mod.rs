use futures::{executor::block_on};

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) type OnEvent = dyn Fn(Event, &mut Box<dyn Writer + Send + Sync>) + Send + Sync + 'static;

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: String,
}

pub enum Event {
    Ready(),
    Text(Vec<u8>),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<Reason>),
}

pub trait Writer {
    fn write(&mut self, data: Vec<u8>);
    fn write_binary(&mut self, data: Vec<u8>);
    fn ping(&mut self, data: Vec<u8>);
    fn pong(&mut self, data: Vec<u8>);
    fn close(&mut self);
}

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
}

impl Ws {
    pub fn new() -> Self {
        return Ws {
            event: None
        } 
    }

    pub fn on<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn<(Event, &'a mut Box<dyn Writer + Send + Sync>), Output = ()> + Send + Sync + 'static
    {
        self.event = Some(Box::new(move |event, writer| block_on(callback(event, writer))));
    }
}
