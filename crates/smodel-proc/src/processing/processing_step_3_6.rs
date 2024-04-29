use crate::*;

pub struct ProcessingStep3_6();

impl ProcessingStep3_6 {
    pub fn exec(&self, host: &mut LmtHost, meaning: &Symbol, base_accessor: &str) {
        let meaning_name = meaning.name();

        // Define the structure M, as in
        //
        // ```
        // #[derive(Clone)]
        // struct M(Weak<__data__::M>);
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
                #[derive(Clone, PartialEq, Hash)]
                pub struct #meaning_name(#inherited_name);

                impl ::std::ops::Deref for #meaning_name {
                    type Target = #inherited_name;
                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }
            }.try_into().unwrap());
        } else {
            host.output.extend::<TokenStream>(quote! {
                #[derive(Clone)]
                pub struct #meaning_name(Weak<__data__::#meaning_name>);

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

        // Output a TryFrom<InheritedM> for M implementation (contravariant conversion)
        for sm in meaning.submeanings().iter() {
            self.contravariance(host, base_accessor, meaning, &sm);
        }
    }

    fn contravariance(&self, host: &mut LmtHost, base_accessor: &str, base_meaning: &Symbol, submeaning: &Symbol) {
        //
    }
}