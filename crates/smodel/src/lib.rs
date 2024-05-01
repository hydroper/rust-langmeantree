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

            type Arena = Arena;
        
            /// My unified data type.
            struct Symbol {
                let x: f64 = 0.0;
                let ref y: String = "".into();
        
                pub fn Symbol() {
                    super();
                }
        
                /// Empty, Foo, FooBar or FooQux
                pub fn name(&self) -> String {
                    "".into()
                }

                pub fn base_example(&self) -> String {
                    "from base".into()
                }
            }
        
            struct Foo: Symbol {
                pub fn Foo() {
                    super();
                }

                pub override fn name(&self) -> String {
                    "Foo".into()
                }
            }
        
            struct FooBar: Foo {
                pub fn FooBar() {
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
            
            struct FooBarBar: FooBar {
                pub fn FooBarBar() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooBarBar".into()
                }
            }
        
            struct FooQux: Foo {
                pub fn FooQux() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooQux".into()
                }
            }
        }


        let arena = Arena::new();

        let symbol = Foo::new(&arena);
        let base_symbol: Symbol = symbol.into();
        assert_eq!("Foo", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(false, base_symbol.is::<FooBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from base", base_symbol.base_example());

        let symbol = FooBar::new(&arena);
        let base_symbol: Symbol = symbol.into();
        assert_eq!("FooBar", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(true, base_symbol.is::<FooBar>());
        assert_eq!(false, base_symbol.is::<FooBarBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from bar; from base", base_symbol.base_example());

        let symbol = FooBarBar::new(&arena);
        let base_symbol: Symbol = symbol.into();
        assert_eq!("FooBarBar", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(true, base_symbol.is::<FooBar>());
        assert_eq!(true, base_symbol.is::<FooBarBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from bar; from base", base_symbol.base_example());

        let symbol = FooQux::new(&arena);
        let base_symbol: Symbol = symbol.into();
        assert_eq!("FooQux", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(false, base_symbol.is::<FooBar>());
        assert_eq!(true, base_symbol.is::<FooQux>());
    }
}