use syn::Meta;

use crate::*;

pub struct ProcessingStep3_8();

impl ProcessingStep3_8 {
    // Process a method
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<MeaningMethod>, meaning: &Symbol, base_accessor: &str, asc_meaning_list: &[Symbol]) {
        let input = &node.inputs;
        let type_params = [node.generics.lt_token.to_token_stream(), node.generics.params.to_token_stream(), node.generics.gt_token.to_token_stream()];
        let where_clause = node.generics.where_clause.as_ref().map(|c| c.to_token_stream()).unwrap_or(proc_macro2::TokenStream::new());
        let vis = node.visibility.clone();
        let name = node.name.clone();
        let mut result_annotation = proc_macro2::TokenStream::new();
        if let Some(t) = &node.result_type {
            result_annotation.extend::<proc_macro2::TokenStream>(quote!{->});
            result_annotation.extend::<proc_macro2::TokenStream>(t.to_token_stream());
        }

        // Static method
        if Self::begins_with_no_receiver(&node.inputs) {
            let attr = node.attributes.borrow().clone();
            let stmt = &node.statements;
            meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
                #(#attr)*
                #vis fn #name #(#type_params)*(#input) #result_annotation #where_clause {
                    #stmt
                }
            }.try_into().unwrap());
            return;
        }

        // Validate receiver
        if !Self::begins_with_instance_receiver(&node.inputs) {
            node.inputs.span().unwrap().error("Instance receiver must be exactly `&self`.").emit();
            return;
        }

        // Remove the receiver
        let mut inputs1 = node.inputs.iter().cloned().collect::<Vec<_>>();
        inputs1.remove(0);
        let mut inputs = Punctuated::<FnArg, Comma>::new();
        inputs.extend(inputs1);

        // * Look for the #[doc] attribute.
        // * Look for the #[inheritdoc] attribute.
        let mut doc_attr: Option<syn::Attribute> = None;
        let mut inheritdoc_index: Option<usize> = None;
        let mut i = 0usize;
        for attr in node.attributes.borrow().iter() {
            if let Meta::List(list) = &attr.meta {
                if list.path.to_token_stream().to_string() == "doc" {
                    doc_attr = Some(attr.clone());
                }
            } else if let Meta::Path(p) = &attr.meta {
                if p.to_token_stream().to_string() == "inheritdoc" {
                    inheritdoc_index = Some(i);
                }
            }
            i += 1;
        }

        // Create a `MethodSlot` with the appropriate settings.
        let slot = host.factory.create_method_slot(name.to_string(), meaning.clone(), doc_attr);

        // Contribute the method slot to the meaning.
        meaning.methods().set(slot.name(), slot.clone());

        // Check if the method has a `#[inheritdoc]` attribute; if it has one:
        //
        // * Remove it
        // * Lookup method in one of the base meanings
        // * Inherit RustDoc comment
        if let Some(i) = inheritdoc_index {
            node.attributes.borrow_mut().remove(i);

            if let Some(base_method) = meaning.lookup_method_in_base_meaning(&slot.name()) {
                slot.set_doc_attribute(base_method.doc_attribute());
                if let Some(attr) = base_method.doc_attribute() {
                    node.attributes.borrow_mut().push(attr);
                }
            } else {
                name.span().unwrap().error(format!("No inherited method '{}'.", slot.name())).emit();
            }
        }
    }

    fn begins_with_no_receiver(input: &Punctuated<FnArg, Comma>) -> bool {
        if let Some(first) = input.first() {
            matches!(first, FnArg::Receiver(rec))
        } else {
            false
        }
    }

    // Checks whether method formally begins with the exact `&self` receiver.
    fn begins_with_instance_receiver(input: &Punctuated<FnArg, Comma>) -> bool {
        if let Some(first) = input.first() {
            if let FnArg::Receiver(rec) = first {
                if !rec.attrs.is_empty() || rec.mutability.is_some() {
                    return false;
                }
                let Some(reference) = rec.reference.as_ref() else {
                    return false
                };
                if reference.1.is_some() {
                    return false;
                }
                // Ignore the type for now, assuming Self.
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}