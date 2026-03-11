pub mod tcp;
pub mod udp;
pub mod handler;

#[allow(unused)]
pub enum Protocol {
    HTTP1,
    HTTP2,
    HTTP3
}