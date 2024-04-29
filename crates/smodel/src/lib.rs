use std::{cell::RefCell, rc::{Rc, Weak}};
use std::error::Error;
use std::fmt::{Debug, Display};

pub use smodel_proc::smodel;

pub struct Arena<T> {
    data: RefCell<Vec<Rc<T>>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(vec![]),
        }
    }

    pub fn allocate(&self, value: T) -> Weak<T> {
        let obj = Rc::new(value);
        self.data.borrow_mut().push(obj.clone());
        Rc::downgrade(&obj)
    }
}

#[derive(Debug)]
pub enum SModelError {
    Contravariant,
}