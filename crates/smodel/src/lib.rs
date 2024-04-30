use std::{cell::RefCell, rc::{Rc, Weak}};
use std::fmt::Debug;

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
    #[test]
    fn test() {
        use crate::smodel;

        smodel! {
            mod smodel = crate;
            type Arena = MeaningArena;
        
            struct Meaning {
                let x: f64 = 0.0;
                let ref y: String = "".into();
        
                pub fn Meaning() {
                    super();
                    println!("{}", self.m());
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


        let arena = MeaningArena::new();
        let meaning = FooMeaning::new(&arena);
        let base_meaning: Meaning = meaning.into();
        println!("{}", base_meaning.m());
        println!("{}", base_meaning.is::<FooMeaning>());
    }
}