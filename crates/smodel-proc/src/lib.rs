#![feature(proc_macro_diagnostic)]
#![feature(decl_macro)]

mod shared_array;
use proc_macro2::Span;
use shared_array::*;

mod shared_map;
use shared_map::*;

mod symbol;
use symbol::*;

mod tree_semantics;
use syn::spanned::Spanned;
use tree_semantics::*;

mod processing;
use processing::*;

// use std::iter::FromIterator;
use proc_macro::TokenStream;
// use proc_macro2::Span;
use quote::{quote, ToTokens};
// use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::Comma;
// use syn::spanned::Spanned;
use syn::{braced, parenthesized, parse_macro_input, Attribute, Expr, FnArg, Generics, Ident, Pat, Path, Stmt, Token, Type, Visibility, WhereClause};

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use by_address::ByAddress;

/// Data module name.
const DATA: &'static str = "__data__";

/// Field name used for holding an enumeration of submeanings.
const DATA_VARIANT_FIELD: &'static str = "__variant";

/// Prefix used for enumerations of submeanings.
const DATA_VARIANT_PREFIX: &'static str = "__variant_";

/// Variant name used for indicating that no submeaning is instantiated.
const DATA_VARIANT_NO_SUBMEANING: &'static str = "__NoSubmeaning";

struct MeaningTree {
    arena_type_name: proc_macro2::TokenStream,
    meanings: Vec<Rc<Meaning>>,
}

struct Meaning {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    inherits: Option<Ident>,
    fields: Vec<Rc<MeaningField>>,
    constructor: Option<MeaningConstructor>,
    methods: Vec<Rc<MeaningMethod>>,
}

struct MeaningField {
    is_ref: bool,
    name: Ident,
    type_annotation: Type,
    default_value: Expr,
}

enum MeaningMethodOrConstructor {
    Method(MeaningMethod),
    Constructor(MeaningConstructor),
}

struct MeaningConstructor {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    generics: Generics,
    inputs: Punctuated<FnArg, Comma>,
    super_arguments: Punctuated<Expr, Comma>,
    statements: Vec<Stmt>,
}

struct MeaningMethod {
    attributes: RefCell<Vec<Attribute>>,
    visibility: Visibility,
    is_override: bool,
    name: Ident,
    generics: Generics,
    inputs: Punctuated<FnArg, Comma>,
    result_type: Option<Type>,
    statements: proc_macro2::TokenStream,
}

impl Parse for MeaningTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let arena_type_name = parse_meaning_arena_type_name(input)?.to_token_stream();
        let mut meanings = vec![];
        while !input.is_empty() {
            meanings.push(Rc::new(input.parse::<Meaning>()?));
        }
        Ok(Self {
            arena_type_name,
            meanings,
        })
    }
}

impl Parse for Meaning {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse::<Visibility>()?;
 
        input.parse::<Token![struct]>()?;
 
        let name = input.parse::<Ident>()?;
        let name_str = name.to_string();

        // Inherits
        let mut inherits: Option<Ident> = None;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            inherits = Some(input.parse::<Ident>()?);
        }

        let mut fields: Vec<Rc<MeaningField>> = vec![];
        let mut constructor: Option<MeaningConstructor> = None;
        let mut methods: Vec<Rc<MeaningMethod>> = vec![];
        let braced_content;
        let _ = braced!(braced_content in input);

        while !braced_content.is_empty() {
            if braced_content.peek(Token![let]) {
                fields.push(Rc::new(parse_meaning_field(&braced_content)?));
            } else {
                match parse_meaning_method(&braced_content, &name_str)? {
                    MeaningMethodOrConstructor::Constructor(ctor) => {
                        constructor = Some(ctor);
                    },
                    MeaningMethodOrConstructor::Method(m) => {
                        methods.push(Rc::new(m));
                    },
                }
            }
        }

        Ok(Self {
            attributes,
            visibility,
            name,
            inherits,
            fields,
            constructor,
            methods,
        })
    }
}

fn parse_meaning_field(input: ParseStream) -> Result<MeaningField> {
    input.parse::<Token![let]>()?;
    let is_ref = if input.peek(Token![ref]) {
        input.parse::<Token![ref]>()?;
        true
    } else {
        false
    };
    let name = input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let type_annotation = input.parse::<Type>()?;
    input.parse::<Token![=]>()?;
    let default_value = input.parse::<Expr>()?;
    input.parse::<Token![;]>()?;

    Ok(MeaningField {
        is_ref,
        name,
        type_annotation,
        default_value,
    })
}

fn parse_meaning_method(input: ParseStream, meaning_name: &str) -> Result<MeaningMethodOrConstructor> {
    let attributes = Attribute::parse_outer(input)?;
    let visibility = input.parse::<Visibility>()?;
    let is_override = if input.peek(Token![override]) {
        input.parse::<Token![override]>()?;
        true
    } else {
        false
    };
    input.parse::<Token![fn]>()?;
    let mut is_constructor = false;
    let id = input.parse::<Ident>()?;
    if !is_override && id.to_string() == meaning_name {
        // id.span().unwrap().error("Identifier must be equals \"constructor\"").emit();
        is_constructor = true;
    }
    let mut generics = input.parse::<Generics>()?;

    let parens_content;
    parenthesized!(parens_content in input);
    let inputs = parens_content.parse_terminated(FnArg::parse, Comma)?;

    let result_type: Option<Type> = if !is_constructor && input.peek(Token![->]) {
        input.parse::<Token![->]>()?;
        Some(input.parse::<Type>()?)
    } else {
        None
    };

    generics.where_clause = if input.peek(Token![where]) { Some(input.parse::<WhereClause>()?) } else { None };

    let braced_content;
    let _ = braced!(braced_content in input);

    if !is_constructor {
        let statements = braced_content.parse::<proc_macro2::TokenStream>()?;
        return Ok(MeaningMethodOrConstructor::Method(MeaningMethod {
            attributes: RefCell::new(attributes),
            visibility,
            is_override,
            name: id,
            generics,
            inputs,
            result_type,
            statements,
        }));
    }

    braced_content.parse::<Token![super]>()?;

    let paren_content;
    let _ = parenthesized!(paren_content in braced_content);
    let super_arguments = paren_content.parse_terminated(Expr::parse, Comma)?;
    braced_content.parse::<Token![;]>()?;

    let mut statements = vec![];
    while !braced_content.is_empty() {
        statements.push(braced_content.parse::<Stmt>()?);
    }

    Ok(MeaningMethodOrConstructor::Constructor(MeaningConstructor {
        attributes,
        visibility,
        generics,
        inputs,
        super_arguments,
        statements,
    }))
}

fn parse_meaning_arena_type_name(input: ParseStream) -> Result<Path> {
    input.parse::<Token![type]>()?;
    let id = input.parse::<Ident>()?;
    if id.to_string() != "Arena" {
        id.span().unwrap().error("Identifier must be equals \"Arena\"").emit();
    }
    input.parse::<Token![=]>()?;
    let path = Path::parse_mod_style(input)?;
    input.parse::<Token![;]>()?;
    Ok(path)
}

#[proc_macro]
pub fn smodel(input: TokenStream) -> TokenStream {
    let MeaningTree {
        arena_type_name, meanings
    } = parse_macro_input!(input as MeaningTree);

    let mut host = SModelHost::new();

    // # Validations

    // 1. Ensure there is at least one meaning.

    if meanings.is_empty() {
        panic!("There must be at least one meaning.");
    }

    // 2. Ensure the first meaning inherits no other one.

    if meanings[0].inherits.is_some() {
        meanings[0].name.span().unwrap().error("First meaning must inherit no any other meaning.").emit();
        return TokenStream::new();
    }
    let base_meaning_name = Ident::new(&meanings[0].name.to_string(), Span::call_site());

    // 3. Ensure all other meanings inherit another one.

    for m in meanings[1..].iter() {
        if m.inherits.is_none() {
            m.name.span().unwrap().error("Meaning must inherit another meaning.").emit();
            return TokenStream::new();
        }
    }

    // # Processing steps

    let data_id = Ident::new(DATA, Span::call_site());

    // 1. Output the arena type.
    host.output.extend::<TokenStream>(quote! {
        type #arena_type_name = ::smodel::Arena<#data_id::#base_meaning_name>;
    }.try_into().unwrap());

    // 2. Traverse each meaning in a first pass.
    for meaning_node in meanings.iter() {
        ProcessingStep2().exec(&mut host, meaning_node);
    }

    // 3. Traverse each meaning.
    for meaning_node in meanings.iter() {
        let Some(meaning) = host.semantics.get(meaning_node) else {
            continue;
        };

        let asc_meaning_list = meaning.asc_meaning_list();
        let mut field_output = proc_macro2::TokenStream::new();
        let meaning_name = meaning.name();
        let meaning_name_id = Ident::new(&meaning_name, Span::call_site());

        // 3.1. Write out the base data accessor
        //
        // A `Weak<#DATA::FirstM>` value.
        //
        // For example, for the base meaning data type, this
        // is always "self.0"; for a direct submeaning of the base
        // data type, this is always "self.0.0".

        let mut base_accessor = "self.0".to_owned();
        let mut m1 = meaning.clone();
        while let Some(m2) = m1.inherits() {
            base_accessor.push_str(".0");
            m1 = m2;
        }

        // 3.2. Traverse each field.
        for field in meaning_node.fields.iter() {
            ProcessingStep3_2().exec(&mut host, &meaning, field, &base_accessor, &asc_meaning_list, &mut field_output);
        }

        // 3.3. Contribute a #DATA_VARIANT_FIELD field to #DATA::M
        // holding the enumeration of submeanings.
        let submeaning_enum = Ident::new(&(DATA_VARIANT_PREFIX.to_owned() + &meaning_name), Span::call_site());
        let data_variant_field_id = Ident::new(DATA_VARIANT_FIELD, Span::call_site());
        field_output.extend(quote! {
            pub #data_variant_field_id: #submeaning_enum,
        });

        // 3.4. Contribute a #[non_exhaustive] enumeration of submeanings at the `#DATA` module.
        let mut variants: Vec<proc_macro2::TokenStream> = vec![];
        for submeaning in meaning.submeanings().iter() {
            let sn = submeaning.name();
            variants.push(proc_macro2::TokenStream::from_str(&format!("{sn}(::std::rc::Rc<{sn}>)")).unwrap());
        }
        let data_variant_no_submeaning = Ident::new(DATA_VARIANT_NO_SUBMEANING, Span::call_site());;
        host.data_output.extend(quote! {
            #[non_exhaustive]
            pub enum #submeaning_enum {
                #(#variants),*
                #data_variant_no_submeaning,
            }
        });

        // 3.5. Define the data structure #DATA::M at the #DATA module output,
        // containing all field output.
        host.data_output.extend(quote! {
            #[non_exhaustive]
            pub struct #meaning_name_id {
                #field_output
            }
        });

        // 3.6. Define the structure M
        ProcessingStep3_6().exec(&mut host, &meaning_node, &meaning, &base_accessor);

        // 3.7. Define the constructor
        ProcessingStep3_7().exec(&mut host, meaning_node.constructor.as_ref(), &meaning, &asc_meaning_list, &arena_type_name.to_string());

        // 3.8. Traverse each method
        for method in meaning_node.methods.iter() {
            ProcessingStep3_8().exec(&mut host, method, &meaning);
        }

        // 3.9. Traverse each method
        for method in meaning_node.methods.iter() {
            ProcessingStep3_9().exec(&mut host, method, &meaning);
        }

        // * Contribute a `to::<T: TryFrom<M>>()` method.
        // * Contribute an `is::<T>()` method.
        meaning.method_output().borrow_mut().extend(quote! {
            pub fn to<T: TryFrom<#meaning_name_id>>(&self) -> Result<T, ::smodel::SModelError> {
                T::try_from(self.clone())
            }
            pub fn is<T: TryFrom<#meaning_name_id>>(&self) -> bool {
                T::try_from(self.clone()).is_ok()
            }
        });

        let method_output = meaning.method_output().borrow().clone();

        // Output the code of all methods to an `impl` block for the meaning data type.
        host.output.extend::<TokenStream>(quote! {
            impl #meaning_name_id {
                #method_output
            }
        }.try_into().unwrap());
    }

    let data_output = host.data_output;

    // 4. Output the `mod #DATA { use super::*; ... }` module with its respective contents
    host.output.extend::<TokenStream>(quote! {
        mod #data_id {
            use super::*;

            #data_output
        }
    }.try_into().unwrap());

    // 5. Return output.
    host.output
}

fn convert_function_input_to_arguments(input: &Punctuated<FnArg, Comma>) -> Punctuated<proc_macro2::TokenStream, Comma> {
    let mut out = Punctuated::<proc_macro2::TokenStream, Comma>::new();
    for arg in input.iter() {
        if let FnArg::Receiver(_) = arg {
            arg.span().unwrap().error("Unexpected receiver.").emit();
            continue;
        } else {
            let FnArg::Typed(pt) = arg else {
                panic!();
            };
            let Pat::Ident(id) = pt.pat.as_ref() else {
                pt.pat.span().unwrap().error("Pattern must be an identifier.").emit();
                continue;
            };
            out.push(id.to_token_stream());
        }
    }
    out
}