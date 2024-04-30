use crate::*;

pub struct ProcessingStep3_6();

impl ProcessingStep3_6 {
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<Meaning>, meaning: &Symbol, base_accessor: &str, smodel_path: &proc_macro2::TokenStream) {
        let meaning_name = Ident::new(&meaning.name(), Span::call_site());
        let attributes = node.attributes.clone();
        let visi = node.visibility.clone();

        // Define the structure M, as in
        //
        // ```
        // #[derive(Clone)]
        // struct M(Weak<#DATA::M>);
        // ```
        //
        // or as in:
        //
        // ```
        // #[derive(Clone, PartialEq, Hash)]
        // struct M(InheritedM);
        // ```
        //
        // if there is an inherited meaning.
        if let Some(inherits) = meaning.inherits() {
            let inherited_name = Ident::new(&inherits.name(), Span::call_site());
            host.output.extend::<TokenStream>(quote! {
                #(#attributes)*
                #[derive(Clone, PartialEq, Hash)]
                #visi struct #meaning_name(#inherited_name);

                impl ::std::ops::Deref for #meaning_name {
                    type Target = #inherited_name;
                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }
            }.try_into().unwrap());
        } else {
            let data_id = Ident::new(DATA, Span::call_site());
            host.output.extend::<TokenStream>(quote! {
                #(#attributes)*
                #[derive(Clone)]
                #visi struct #meaning_name(::std::rc::Weak<#data_id::#meaning_name>);

                impl PartialEq for #meaning_name {
                    fn eq(&self, other: &Self) -> bool {
                        self.0.ptr_eq(&other.0)
                    }
                }

                impl ::std::hash::Hash for #meaning_name {
                    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                        self.0.as_ptr().hash(state)
                    }
                }
            }.try_into().unwrap());
        }

        // Output From<M> for InheritedM implementation (covariant conversion)
        let mut base = "v.0.0".to_owned();
        let mut m = meaning.clone();
        while let Some(m1) = m.inherits() {
            let inherited_name = Ident::new(&m1.name(), Span::call_site());
            let base_tokens = proc_macro2::TokenStream::from_str(&base).unwrap();
            host.output.extend::<TokenStream>(quote! {
                impl From<#meaning_name> for #inherited_name {
                    fn from(v: #meaning_name) -> Self {
                        #inherited_name(#base_tokens.clone())
                    }
                }
            }.try_into().unwrap());
            m = m1;
            base = format!("{base}.0");
        }

        // Output a TryFrom<M> for SubmeaningM implementation (contravariant conversion)
        for sm in meaning.submeanings().iter() {
            self.contravariance(host, &base_accessor.replacen("self", "v", 1), meaning, &sm, smodel_path);
        }
    }

    fn contravariance(&self, host: &mut SModelHost, base_accessor: &str, base_meaning: &Symbol, submeaning: &Symbol, smodel_path: &proc_macro2::TokenStream) {
        let base_meaning_name = Ident::new(&base_meaning.name(), Span::call_site());
        let submeaning_name = Ident::new(&submeaning.name(), Span::call_site());
        let m = proc_macro2::TokenStream::from_str(&self.match_contravariant(&submeaning.asc_meaning_list(), 0, &format!("{base_accessor}.upgrade().unwrap()"), &base_accessor, smodel_path)).unwrap();

        host.output.extend::<TokenStream>(quote! {
            impl TryFrom<#base_meaning_name> for #submeaning_name {
                type Error = #smodel_path::SModelError;
                fn try_from(v: #base_meaning_name) -> Result<Self, Self::Error> {
                    #m
                }
            }
        }.try_into().unwrap());

        for sm1 in submeaning.submeanings().iter() {
            self.contravariance(host, base_accessor, base_meaning, &sm1, smodel_path);
        }
    }

    /// Matches a contravariant meaning.
    /// 
    /// * `base` is assumed to be a `Rc<#DATA::M>` value.
    /// * `original_base` is assumed to be a `Weak<#DATA::FirstM>` value.
    fn match_contravariant(&self, asc_meaning_list: &[Symbol], meaning_index: usize, base: &str, original_base: &str, smodel_path: &proc_macro2::TokenStream) -> String {
        let (meaning, inherited) = if meaning_index + 1 >= asc_meaning_list.len() {
            (asc_meaning_list[meaning_index].clone(), None)
        } else {
            (asc_meaning_list[meaning_index + 1].clone(), Some(asc_meaning_list[meaning_index].clone()))
        };

        let Some(inherited) = inherited else {
            return format!("Ok({})", Symbol::create_layers_over_weak_root(original_base, asc_meaning_list));
        };
        format!("if let {DATA}::{}::{}(_o) = &{base}.{DATA_VARIANT_FIELD} {{ {} }} else {{ Err({}::SModelError::Contravariant) }}",
            DATA_VARIANT_PREFIX.to_owned() + &inherited.name(),
            meaning.name(),
            self.match_contravariant(asc_meaning_list, meaning_index + 1, "_o", original_base, smodel_path),
            smodel_path.to_string())
    }
}