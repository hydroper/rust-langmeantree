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

#[cfg(test)]
mod test {
    fn test() {
        use crate::smodel;

        smodel! {
            type Arena = MeaningArena;
        
            struct Meaning {
                let x: f64 = 0.0;
                let ref y: String = "".into();
        
                pub fn Meaning() {
                    super();
                    println!("{}", this.m());
                }
        
                pub fn m(&self) -> String {
                    "".into()
                }
        
                pub fn m1(&self) {
                    println!("base");
                }
            }
        
            struct FooMeaning: Meaning {
                pub fn FooMeaning() {
                    super();
                }
        
                pub override fn m(&self) -> String {
                    "Foo".into()
                }
        
                pub override fn m1(&self) {
                    if true {
                        super.m1();
                    }
                }
            }
        }
    }
}