use syn::Meta;
use crate::*;

pub const NONDISPATCH_PREFIX: &'static str = "__nd_";

pub struct ProcessingStep3_8();

impl ProcessingStep3_8 {
    // Process a method
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<MeaningMethod>, meaning: &Symbol) {
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
            meaning.method_output().borrow_mut().extend(quote! {
                #(#attr)*
                #vis fn #name #(#type_params)*(#input) #result_annotation #where_clause {
                    #stmt
                }
            });
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

        // Map node to slot
        host.semantics.set(&node, Some(slot.clone()));

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
                name.span().unwrap().error(format!("No method '{}' in base.", slot.name())).emit();
            }
        }

        // Define `nondispatch_name` as nondispatch prefix plus method name.
        let nondispatch_name = format!("{NONDISPATCH_PREFIX}{}", slot.name());
        let nondispatch_name_id = Ident::new(&nondispatch_name, Span::call_site());

        // Define input argument list
        let input_args = convert_function_input_to_arguments(&inputs);

        // Process super expressions
        let statements = self.process_super_expression(node.statements.clone(), meaning, &slot);

        // If the method is marked as "override"
        //
        // * Lookup for a method with the same name in one of the base meanings
        // * Contribute "overriding" return call code to the respective
        //   override logic mapping according to meaning inheritance.
        if node.is_override {
            if let Some(base_method) = meaning.lookup_method_in_base_meaning(&slot.name()) {
                self.perform_override(&slot.name(), base_method.override_logic_mapping(), &base_method.defined_in(), meaning, &input_args);
            } else {
                name.span().unwrap().error(format!("No method '{}' in base.", slot.name())).emit();
            }
        }

        let mut attr = node.attributes.borrow().clone();

        // Remove #[doc] attributes from nondispatch methods
        // for less cost.
        let mut indices = Vec::<usize>::new();
        let mut i: usize = 0;
        for attr in attr.iter() {
            if let Meta::List(list) = &attr.meta {
                if list.path.to_token_stream().to_string() == "doc" {
                    indices.push(i);
                }
            }
            i += 1;
        }
        for i in indices.iter().rev() {
            attr.remove(*i);
        }

        meaning.method_output().borrow_mut().extend(quote! {
            #(#attr)*
            fn #nondispatch_name_id #(#type_params)*(&self, #inputs) #result_annotation #where_clause {
                #statements
            }
        });
    }

    fn begins_with_no_receiver(input: &Punctuated<FnArg, Comma>) -> bool {
        if let Some(first) = input.first() {
            !(matches!(first, FnArg::Receiver(_)))
        } else {
            true
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

    fn process_super_expression(&self, input: proc_macro2::TokenStream, meaning: &Symbol, method_slot: &Symbol) -> proc_macro2::TokenStream {
        let mut input = input.into_iter();
        let mut output = proc_macro2::TokenStream::new();
        while let Some(token1) = input.next() {
            match &token1 {
                proc_macro2::TokenTree::Ident(id) => {
                    if id.to_string() != "super" {
                        output.extend([token1.clone()]);
                        continue;
                    }
                    let Some(token2) = input.next() else {
                        output.extend([token1.clone()]);
                        continue;
                    };
                    let proc_macro2::TokenTree::Punct(p) = &token2 else {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        continue;
                    };
                    if p.to_string() != "." {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        continue;
                    }
                    let Some(token3) = input.next() else {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        continue;
                    };
                    let proc_macro2::TokenTree::Ident(id) = &token3 else {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        output.extend([token3.clone()]);
                        continue;
                    };
                    let Some(token4) = input.next() else {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        output.extend([token3.clone()]);
                        continue;
                    };
                    let proc_macro2::TokenTree::Group(g) = &token4 else {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        output.extend([token3.clone()]);
                        output.extend([token4.clone()]);
                        continue;
                    };
                    if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
                        output.extend([token1.clone()]);
                        output.extend([token2.clone()]);
                        output.extend([token3.clone()]);
                        output.extend([token4.clone()]);
                        continue;
                    }

                    // Found super expression.

                    // Lookup for a method in one of the base meanings.
                    let Some(base_method) = meaning.lookup_method_in_base_meaning(&id.to_string()) else {
                        id.span().unwrap().error(format!("No method '{}' in base.", id.to_string())).emit();
                        continue;
                    };

                    // Let base be "self" followed by n = delta_of_descending_list_until_base_meaning
                    // (where `base_meaning` is the base found method's `.defined_in()` call)
                    // repeats of "".0".
                    let mut base = "self".to_owned();
                    let mut m = meaning.clone();
                    while let Some(m1) = m.inherits() {
                        base.push_str(".0");
                        if m1 == base_method.defined_in() {
                            break;
                        }
                        m = m1;
                    }
                    let base = proc_macro2::TokenStream::from_str(&base).unwrap();

                    // Replace super.m(...) by BaseM::#nondispatch_name_id(&#base, ...)
                    let nondispatch_name = format!("{NONDISPATCH_PREFIX}{}", base_method.name());
                    let nondispatch_name_id = Ident::new(&nondispatch_name, Span::call_site());
                    let base_meaning = Ident::new(&base_method.defined_in().name(), Span::call_site());
                    let super_args = self.process_super_expression(g.stream(), meaning, method_slot);
                    output.extend(quote! {
                        #base_meaning::#nondispatch_name_id(&#base, #super_args)
                    });
                },
                proc_macro2::TokenTree::Group(g) => {
                    let stream = self.process_super_expression(g.stream(), meaning, method_slot);
                    output.extend([proc_macro2::TokenTree::Group(proc_macro2::Group::new(g.delimiter(), stream))]);
                },
                _ => {
                    output.extend([token1.clone()]);
                },
            }
        }
        output
    }

    fn perform_override(&self, method_name: &str, mut override_logic_mapping: SharedMap<Symbol, Rc<OverrideLogicMapping>>, base_meaning: &Symbol, target_meaning: &Symbol, input_args: &Punctuated<proc_macro2::TokenStream, Comma>) {
        let meaning_list = target_meaning.asc_meaning_list();
        let mut i = 0usize;
        for m in meaning_list.iter() {
            if m == base_meaning {
                break;
            }
            i += 1;
        }
        for m in meaning_list[(i + 1)..(meaning_list.len() - 1)].iter() {
            if let Some(old_mapping) = override_logic_mapping.get(m) {
                override_logic_mapping = old_mapping.override_logic_mapping();
            } else {
                let new_mapping = Rc::new(OverrideLogicMapping::new());
                override_logic_mapping.set(m.clone(), new_mapping.clone());
                override_logic_mapping = new_mapping.override_logic_mapping();
            }
        }

        // Generate layers
        let mut layers = String::new();
        let mut parens = 0usize;
        for m in meaning_list[(i + 1)..].iter().rev() {
            layers.push_str(&format!("{}(", m.name()));
            parens += 1;
        }
        layers.push_str("self.clone()");
        layers.push_str(&")".repeat(parens));

        let new_mapping = Rc::new(OverrideLogicMapping::new());
        new_mapping.set_override_code(Some(quote! {
            return #layers.#method_name(#input_args);
        }));
        override_logic_mapping.set(target_meaning.clone(), new_mapping.clone());
    }
}