#![feature(proc_macro_diagnostic)]
#![feature(decl_macro)]

mod shared_array;
use shared_array::*;

mod shared_map;
use shared_map::*;

mod symbol;
use symbol::*;

mod tree_semantics;
use tree_semantics::*;

mod processing;
use processing::*;

// use std::iter::FromIterator;
use proc_macro::TokenStream;
// use proc_macro2::Span;
use quote::{quote, ToTokens};
// use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseBuffer, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma};
// use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, Token, Path, Visibility, Attribute, Type, Expr, Generics, FnArg, Stmt, braced, WhereClause, parenthesized};

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use by_address::ByAddress;

fn parse_full_qualified_id(input: ParseStream) -> Result<Path> {
    Ok(Path::parse_mod_style(input)?)
}

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
    attributes: Vec<Attribute>,
    visibility: Visibility,
    is_override: bool,
    name: Ident,
    generics: Generics,
    inputs: Punctuated<FnArg, Comma>,
    statements: Brace,
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
            input.parse::<Token![:]>();
            inherits = Some(input.parse::<Ident>()?);
        }

        let mut fields: Vec<Rc<MeaningField>> = vec![];
        let mut constructor: Option<MeaningConstructor> = None;
        let mut methods: Vec<Rc<MeaningMethod>> = vec![];
        let braced_content;
        let _ = braced!(braced_content in input);

        while !braced_content.is_empty() {
            if input.peek(Token![let]) {
                fields.push(Rc::new(parse_meaning_field(input)?));
            } else {
                match parse_meaning_method(input, &name_str)? {
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

    generics.where_clause = if input.peek(Token![where]) { Some(input.parse::<WhereClause>()?) } else { None };

    let braced_content;
    let brace_token = braced!(braced_content in input);

    if !is_constructor {
        return Ok(MeaningMethodOrConstructor::Method(MeaningMethod {
            attributes,
            visibility,
            is_override,
            name: id,
            generics,
            inputs,
            statements: brace_token,
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
pub fn langmeantree(input: TokenStream) -> TokenStream {
    let MeaningTree {
        arena_type_name, meanings
    } = parse_macro_input!(input as MeaningTree);

    let mut host = LmtHost::new();

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
    let base_meaning_name = meanings[0].name.to_string();

    // 3. Ensure all other meanings inherit another one.

    for m in meanings[1..].iter() {
        if m.inherits.is_none() {
            m.name.span().unwrap().error("Meaning must inherit another meaning.").emit();
            return TokenStream::new();
        }
    }

    // # Processing steps

    // 1. Output the arena type
    host.output.extend::<TokenStream>(quote! {
        type #arena_type_name = ::langmeantree::Arena<__data__::#base_meaning_name>;
    }.try_into().unwrap());

    // 2.

    ProcessingStep2().exec(&mut host, &meanings);

    host.output
}