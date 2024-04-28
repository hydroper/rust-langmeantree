use crate::*;

pub struct LmtHost {
    pub factory: LmtFactory,
    pub semantics: TreeSemantics<Symbol>,
    pub meaning_slots: HashMap<String, Symbol>,
    pub output: TokenStream,
    pub data_output: TokenStream,
}

impl LmtHost {
    pub fn new() -> Self {
        Self {
            factory: LmtFactory::new(),
            semantics: TreeSemantics::new(),
            meaning_slots: HashMap::new(),
            output: TokenStream::new(),
            data_output: TokenStream::new(),
        }
    }
}