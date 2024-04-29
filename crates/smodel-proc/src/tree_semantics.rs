use crate::*;

pub struct TreeSemantics<T> {
    meanings: RefCell<HashMap<ByAddress<Rc<Meaning>>, Option<T>>>,
    methods: RefCell<HashMap<ByAddress<Rc<MeaningMethod>>, Option<T>>>,
    fields: RefCell<HashMap<ByAddress<Rc<MeaningField>>, Option<T>>>,
}

impl<T> TreeSemantics<T> {
    pub fn new() -> Self {
        Self {
            meanings: RefCell::new(HashMap::new()),
            methods: RefCell::new(HashMap::new()),
            fields: RefCell::new(HashMap::new()),
        }
    }
}

pub trait TreeSemanticsAccessor<T, S: Clone> {
    fn get(&self, node: &Rc<T>) -> Option<S>;
    fn set(&self, node: &Rc<T>, symbol: Option<S>);
    fn _delete(&self, node: &Rc<T>) -> bool;
    fn _has(&self, node: &Rc<T>) -> bool;
}

impl<S: Clone> TreeSemanticsAccessor<Meaning, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<Meaning>) -> Option<S> {
        self.meanings.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<Meaning>, symbol: Option<S>) {
        self.meanings.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<Meaning>) -> bool {
        self.meanings.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<Meaning>) -> bool {
        self.meanings.borrow().contains_key(&ByAddress(node.clone()))
    }
}

impl<S: Clone> TreeSemanticsAccessor<MeaningField, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<MeaningField>) -> Option<S> {
        self.fields.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<MeaningField>, symbol: Option<S>) {
        self.fields.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<MeaningField>) -> bool {
        self.fields.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<MeaningField>) -> bool {
        self.fields.borrow().contains_key(&ByAddress(node.clone()))
    }
}

impl<S: Clone> TreeSemanticsAccessor<MeaningMethod, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<MeaningMethod>) -> Option<S> {
        self.methods.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<MeaningMethod>, symbol: Option<S>) {
        self.methods.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<MeaningMethod>) -> bool {
        self.methods.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<MeaningMethod>) -> bool {
        self.methods.borrow().contains_key(&ByAddress(node.clone()))
    }
}