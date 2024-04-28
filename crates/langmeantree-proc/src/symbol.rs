use crate::*;

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

pub struct LmtHost {
    arena: Arena<Symbol1>,
}

impl LmtHost {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn create_field_slot(&self, is_ref: bool, name: String, field_type: syn::Type, field_init: syn::Expr) -> Symbol {
        Symbol(self.arena.allocate(Symbol1::FieldSlot(Rc::new(FieldSlot1 {
            is_ref,
            name,
            field_type,
            field_init,
        }))))
    }
}

#[derive(Clone)]
pub struct Symbol(Weak<Symbol1>);

impl Eq for Symbol {}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl Hash for Symbol {
    /// Performs hashing of the symbol by reference.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}

macro access {
    ($symbol:expr) => { $symbol.0.upgrade().unwrap().as_ref() },
}

impl Symbol {
    pub fn is_field_slot(&self) -> bool {
        matches!(access!(self), Symbol1::FieldSlot(_))
    }

    pub fn name(&self) -> String {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.name.clone(),
            _ => panic!(),
        }
    }

    pub fn field_type(&self) -> syn::Type {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.field_type.clone(),
            _ => panic!(),
        }
    }

    pub fn field_init(&self) -> syn::Expr {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.field_init.clone(),
            _ => panic!(),
        }
    }

    pub fn is_ref(&self) -> bool {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.is_ref.clone(),
            _ => panic!(),
        }
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match access!(self) {
            Symbol1::FieldSlot(_) => self.name(),
            _ => "".into(),
        }
    }
}

enum Symbol1 {
    Unresolved,
    FieldSlot(Rc<FieldSlot1>),
    MethodSlot(Rc<MethodSlot1>),
}

struct FieldSlot1 {
    name: String,
    field_type: syn::Type,
    field_init: syn::Expr,
    is_ref: bool,
}

/// A field slot.
/// 
/// # Supported methods
/// 
/// * `is_field_slot()` — Returns `true`.
/// * `is_ref()`
/// * `name()`
/// * `field_type()`
/// * `field_init()`
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FieldSlot(pub Symbol);

impl Deref for FieldSlot {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        assert!(self.0.is_field_slot());
        &self.0
    }
}