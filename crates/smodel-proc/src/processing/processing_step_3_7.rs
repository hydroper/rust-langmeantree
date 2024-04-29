use crate::*;

const CTOR_INIT_NAME: &'static str = "__ctor";

pub struct ProcessingStep3_7();

impl ProcessingStep3_7 {
    // Define the constructor
    pub fn exec(&self, _host: &mut SModelHost, node: Option<&MeaningConstructor>, meaning: &Symbol, asc_meaning_list: &[Symbol], arena_type_name: &str) {
        let input = node.map(|node| node.inputs.clone()).unwrap_or(Punctuated::new());
        let type_params = node.map(|node| [node.generics.lt_token.to_token_stream(), node.generics.params.to_token_stream(), node.generics.gt_token.to_token_stream()]).unwrap_or([
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        ]);
        let where_clause = node.map(|node| node.generics.where_clause.as_ref().map(|c| c.to_token_stream()).unwrap_or(proc_macro2::TokenStream::new())).unwrap_or(proc_macro2::TokenStream::new());
        let attr = node.map(|node| node.attributes.clone()).unwrap_or(vec![]);
        let vis = node.map(|node| node.visibility.to_token_stream()).unwrap_or(proc_macro2::TokenStream::new());

        // Define the the instance `#CTOR_INIT_NAME` method,
        // containing everything but `super()` and structure initialization.
        let statements = node.map(|node| node.statements.clone()).unwrap_or(vec![]);
        meaning.method_output().borrow_mut().extend(quote! {
            #(#attr)*
            fn #CTOR_INIT_NAME #(#type_params)*(&self, #input) #where_clause {
                #(#statements)*
            }
        });

        // `M::new` output
        let mut m_new_out = TokenStream::new();

        // At `M::new`, let `this` be a complex `M2(M1(__arena.allocate(#DATA::M1 { ... })))`
        // (notice the meaning layers) allocation initializing all meaning variants's fields
        // with their default values.
        let initlayer1 = self.init_data(asc_meaning_list, 0);
        let initlayer2 = Symbol::create_layers_over_weak_root(&format!("arena.allocate({})", initlayer1.to_string()), asc_meaning_list);
        m_new_out.extend::<TokenStream>(quote! {
            let this = #initlayer2;
        }.try_into().unwrap());

        // If the meaning inherits another meaning:
        //
        // * At `M::new`, invoke `InheritedM::#CTOR_INIT_NAME(&this.0, ...super_arguments)`,
        //   passing all `super(...)` arguments.
        if let Some(inherited_m) = meaning.inherits() {
            let inherited_m_name = inherited_m.name();
            let super_arguments = node.map(|node| node.super_arguments.clone()).unwrap_or(Punctuated::new());
            m_new_out.extend::<TokenStream>(quote! {
                #inherited_m_name::#CTOR_INIT_NAME(&this.0, #super_arguments);
            }.try_into().unwrap());
        }

        // * Output a `this.#CTOR_INIT_NAME(...arguments);` call to `M::new`.
        // * Output a `this` return to `M::new`.
        let input_args = convert_function_input_to_arguments(&input);
        m_new_out.extend::<TokenStream>(quote! {
            this.#CTOR_INIT_NAME(#input_args);
            this
        }.try_into().unwrap());

        // Output the constructor as a static `new` method (`M::new`) with
        // a prepended `arena: &#arena_type_name` parameter.

        let m_new_out: proc_macro2::TokenStream = m_new_out.into();

        meaning.method_output().borrow_mut().extend(quote! {
            #(#attr)*
            #vis fn new #(#type_params)*(arena: &#arena_type_name, #input) -> Self #where_clause {
                #m_new_out
            }
        });
    }

    fn init_data(&self, asc_meaning_list: &[Symbol], meaning_index: usize) -> proc_macro2::TokenStream {
        let meaning = &asc_meaning_list[meaning_index];
        let meaning_name = meaning.name();
        let mut fields = proc_macro2::TokenStream::new();
        for (name, field) in meaning.fields().borrow().iter() {
            let fv = field.field_init();
            fields.extend(quote! {
                #name: #fv,
            });
        }
        let submeaning_enum = format!("{DATA}::{DATA_VARIANT_PREFIX}{meaning_name}");
        let variant = if meaning_index + 1 < asc_meaning_list.len() {
            let next_m = asc_meaning_list[meaning_index + 1].name();
            let i = self.init_data(asc_meaning_list, meaning_index + 1);
            quote! { #submeaning_enum::#next_m(Rc::new(#i)) }
        } else {
            quote! { #submeaning_enum::#DATA_VARIANT_NO_SUBMEANING }
        };
        quote! {
            #DATA::#meaning_name {
                #fields
                #DATA_VARIANT_FIELD: #variant
            }
        }
    }
}