use crate::*;

pub struct ProcessingStep3_6();

impl ProcessingStep3_6 {
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<Meaning>, meaning: &Symbol, base_accessor: &str) {
        let meaning_name = meaning.name();
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
            let inherited_name = inherits.name();
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
            host.output.extend::<TokenStream>(quote! {
                #(#attributes)*
                #[derive(Clone)]
                #visi struct #meaning_name(::std::rc::Weak<#DATA::#meaning_name>);

                impl PartialEq for #meaning_name {
                    fn eq(&self, other: &Self) -> bool {
                        self.0.ptr_eq(&other.0)
                    }
                }

                impl ::std::hash::Hash for #meaning_name {
                    /// Performs hashing of the symbol by reference.
                    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                        self.0.as_ptr().hash(state)
                    }
                }
            }.try_into().unwrap());
        }

        // Output From<M> for InheritedM implementation (covariant conversion)
        let mut base = "self.0.0".to_owned();
        let mut m = meaning.clone();
        while let Some(m1) = m.inherits() {
            let inherited_name = m1.name();
            host.output.extend::<TokenStream>(quote! {
                impl From<#meaning_name> for #inherited_name {
                    fn from(v: #meaning_name) -> Self {
                        #inherited_name(#base.clone())
                    }
                }
            }.try_into().unwrap());
            m = m1;
            base = format!("{base}.0");
        }

        // Output a TryFrom<M> for SubmeaningM implementation (contravariant conversion)
        for sm in meaning.submeanings().iter() {
            self.contravariance(host, base_accessor, meaning, &sm);
        }
    }

    fn contravariance(&self, host: &mut SModelHost, base_accessor: &str, base_meaning: &Symbol, submeaning: &Symbol) {
        let base_meaning_name = base_meaning.name();
        let submeaning_name = submeaning.name();
        let base_accessor = base_accessor.replacen("self", "v", 1);
        let m = self.match_contravariant(&submeaning.asc_meaning_list(), 0, &format!("{base_accessor}.upgrade().unwrap()"), &base_accessor);

        host.output.extend::<TokenStream>(quote! {
            impl TryFrom<#base_meaning_name> for #submeaning_name {
                type Err = ::smodel::SModelError;
                fn try_from(v: #base_meaning_name) -> Result<Self, Self::Err> {
                    #m
                }
            }
        }.try_into().unwrap());
    }

    /// Matches a contravariant meaning.
    /// 
    /// * `base` is assumed to be a `Rc<#DATA::M>` value.
    /// * `original_base` is assumed to be a `Weak<#DATA::FirstM>` value.
    fn match_contravariant(&self, asc_meaning_list: &[Symbol], meaning_index: usize, base: &str, original_base: &str) -> String {
        let inherited = if asc_meaning_list.len() - meaning_index == 1 {
            None
        } else {
            Some(asc_meaning_list[meaning_index].clone())
        };
        let meaning = asc_meaning_list[meaning_index + if inherited.is_some() { 1 } else { 0 }].clone();

        let Some(inherited) = meaning.inherits() else {
            return Symbol::create_layers_over_weak_root(original_base, asc_meaning_list);
        };
        format!("(if {DATA}::{}::{}(_o) = &{base}.{DATA_VARIANT_FIELD} {{ {} }} else {{ Err(::smodel::SModelError::Contravariant) }})",
            DATA_VARIANT_PREFIX.to_owned() + &inherited.name(),
            meaning.name(),
            self.match_contravariant(asc_meaning_list, meaning_index + 1, "_o", original_base))
    }
}