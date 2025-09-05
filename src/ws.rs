use std::{io::Result};

use serde::Serialize;

pub trait RW {
    fn read(&mut self) -> Result<Vec<u8>>;
    fn write(&mut self, payload: Vec<u8>) -> Result<()>;
}

pub struct Ws {
    rw: Box<dyn RW>
}

impl Ws {
    pub fn new(rw: Box<dyn RW>) -> Self {
        return Ws {
            rw: rw
        }
    }

    pub fn on_ready(&mut self, callback: fn (ws: &mut Ws)) {

    }

    pub fn on_message(&mut self, callback: fn (ws: &mut Ws)) {
        
    }

    pub fn on_ping(&mut self, callback: fn (ws: &mut Ws)) {
        
    }

    pub fn on_pong(&mut self, callback: fn (ws: &mut Ws)) {
        
    }


    pub fn on_close(&mut self, callback: fn (ws: &mut Ws)) {
        
    }


    pub fn on_error(&mut self, callback: fn (ws: &mut Ws)) {
        
    }

    pub fn write(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub fn write_json<J>(&mut self, object: &J) -> Result<()>
    where 
        J: ?Sized + Serialize
    {
        Ok(())
    }

    pub fn write_baniry(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
