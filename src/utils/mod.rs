use std::{collections::HashMap, mem::take};

pub mod url;

pub type Values = HashMap<String, String>;
pub type Configuration = HashMap<String, String>;

pub struct Pointer<T> {
    pointer: T,
}

impl <T>Pointer<T>
where
    T: Default
{
    pub fn new(p: T) -> Self {
        return Self { pointer: p };
    }

    pub fn point(&mut self) -> T {
        return take(&mut self.pointer);
    }

    pub fn clone(p: &mut T) -> T{
        return take(p);
    }
}