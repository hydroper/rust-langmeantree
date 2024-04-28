use crate::*;

pub struct LmtHost {
    pub factory: LmtFactory,
    pub tree_semantics: TreeSemantics<Symbol>,
    pub output: TokenStream,
}

impl LmtHost {
    pub fn new() -> Self {
        Self {
            factory: LmtFactory::new(),
            tree_semantics: TreeSemantics::new(),
            output: TokenStream::new(),
        }
    }
}