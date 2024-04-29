use crate::*;

const CTOR_INIT_NAME: &'static str = "__ctor";

pub struct ProcessingStep3_7();

impl ProcessingStep3_7 {
    // Define the constructor
    pub fn exec(&self, host: &mut SModelHost, node: Option<&MeaningConstructor>, meaning: &Symbol, asc_meaning_list: &[Symbol], arena_type_name: &str) {
        let input = node.map(|node| node.inputs.clone()).unwrap_or(Punctuated::new());
        let type_params = node.map(|node| [node.generics.lt_token.to_token_stream(), node.generics.params.to_token_stream(), node.generics.gt_token.to_token_stream()]).unwrap_or([
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        ]);
        let where_clause = node.map(|node| node.generics.where_clause.as_ref().map(|c| c.to_token_stream()).unwrap_or(proc_macro2::TokenStream::new())).unwrap_or(proc_macro2::TokenStream::new());

        // Define the the instance `#CTOR_INIT_NAME` method,
        // containing everything but `super()` and structure initialization.
        let statements = node.map(|node| node.statements.clone()).unwrap_or(vec![]);
        meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
            fn #CTOR_INIT_NAME #(#type_params)*(&self, #input) #where_clause {
                #(#statements)*
            }
        }.try_into().unwrap());
    }
}