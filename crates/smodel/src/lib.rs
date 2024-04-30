use std::{cell::RefCell, rc::{Rc, Weak}};
use std::fmt::Debug;

pub mod util;

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
                }
        
                /// Empty, FooBar or FooQux
                pub fn name(&self) -> String {
                    "".into()
                }

                pub fn base_example(&self) -> String {
                    "from base".into()
                }
            }
        
            struct FooMeaning: Meaning {
                pub fn FooMeaning() {
                    super();
                }

                pub override fn name(&self) -> String {
                    "Foo".into()
                }
            }
        
            struct FooBarMeaning: FooMeaning {
                pub fn FooBarMeaning() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooBar".into()
                }

                pub override fn base_example(&self) -> String {
                    format!("from bar; {}", super.base_example())
                }
            }
        
            struct FooQuxMeaning: FooMeaning {
                pub fn FooQuxMeaning() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooQux".into()
                }
            }
        }


        let arena = MeaningArena::new();

        let meaning = FooBarMeaning::new(&arena);
        let base_meaning: Meaning = meaning.into();
        assert_eq!("FooBar", base_meaning.name());
        assert_eq!(true, base_meaning.is::<FooMeaning>());
        assert_eq!(true, base_meaning.is::<FooBarMeaning>());
        assert_eq!(false, base_meaning.is::<FooQuxMeaning>());
        assert_eq!("from bar; from base", base_meaning.base_example());

        let meaning = FooQuxMeaning::new(&arena);
        let base_meaning: Meaning = meaning.into();
        assert_eq!("FooQux", base_meaning.name());
        assert_eq!(true, base_meaning.is::<FooMeaning>());
        assert_eq!(false, base_meaning.is::<FooBarMeaning>());
        assert_eq!(true, base_meaning.is::<FooQuxMeaning>());
    }
}