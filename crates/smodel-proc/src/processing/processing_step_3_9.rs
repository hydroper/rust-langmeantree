use syn::Meta;
use crate::*;

pub struct ProcessingStep3_9();

impl ProcessingStep3_9 {
    // Process a method
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<MeaningMethod>, meaning: &Symbol, base_accessor: &str, asc_meaning_list: &[Symbol]) {
        // Skip if it is not mapped to an instance method slot.
        let Some(slot) = host.semantics.get(node) else {
            return;
        };

        let attr = node.attributes.borrow().clone();
        let type_params = [node.generics.lt_token.to_token_stream(), node.generics.params.to_token_stream(), node.generics.gt_token.to_token_stream()];
        let where_clause = node.generics.where_clause.as_ref().map(|c| c.to_token_stream()).unwrap_or(proc_macro2::TokenStream::new());
        let vis = node.visibility.clone();
        let name = node.name.clone();
        let mut result_annotation = proc_macro2::TokenStream::new();
        if let Some(t) = &node.result_type {
            result_annotation.extend::<proc_macro2::TokenStream>(quote!{->});
            result_annotation.extend::<proc_macro2::TokenStream>(t.to_token_stream());
        }

        // Remove the receiver
        let mut inputs1 = node.inputs.iter().cloned().collect::<Vec<_>>();
        inputs1.remove(0);
        let mut inputs = Punctuated::<FnArg, Comma>::new();
        inputs.extend(inputs1);

        // Define `nondispatch_name` as nondispatch prefix plus method name.
        let nondispatch_name = format!("{NONDISPATCH_PREFIX}{}", slot.name());

        // Define input argument list
        let input_args = convert_function_input_to_arguments(&inputs);

        // Contribute the method #method_name with prepended dynamic dispatch logic,
        // invoking `self.#nondispatch_name(#input_args)` at the end of the method body,
        // to the `impl` output.
        meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
            #(#attr)*
            #vis fn #name #(#type_params)*(&self, #inputs) #result_annotation #where_clause {
                #dynamic_dispatch
                self.#nondispatch_name(#input_args)
            }
        }.try_into().unwrap());
    }
}