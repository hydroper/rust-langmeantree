#![feature(proc_macro_diagnostic)]

// use std::iter::FromIterator;
use proc_macro::TokenStream;
// use proc_macro2::Span;
use quote::{quote, ToTokens};
// use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseBuffer, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma, Pub};
// use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, Token, Path, Visibility, Attribute, Type, Expr, Generics, FnArg, Stmt, braced, WhereClause, parenthesized};

struct MeaningExtends {
    /// Types in descending order
    type_sequence: Vec<Path>,
    component_type: Path,
    oop_inheritance_crate: Option<proc_macro2::TokenStream>,
}

fn parse_full_qualified_id(input: ParseStream) -> Result<Path> {
    Ok(Path::parse_mod_style(input)?)
}

struct MeaningTree {
    arena_type_name: Option<proc_macro2::TokenStream>,
    meanings: Vec<Meaning>,
}

struct Meaning {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    inherits: Option<Vec<Path>>,
    fields: Vec<MeaningField>,
    constructor: MeaningConstructor,
    methods: Vec<MeaningMethod>,
}

struct MeaningField {
    is_mut: bool,
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
    }
}

fn parse_meaning_field(input: ParseStream) -> Result<MeaningField> {
    input.parse::<Token![let]>()?;
    let mut is_ref = if input.peek(Token![ref]) {
        input.parse::<Token![ref]>()?;
        true
    } else {
        false
    };
    let is_mut = if input.peek(Token![mut]) {
        input.parse::<Token![mut]>()?;
        true
    } else {
        false
    };
    is_ref = if input.peek(Token![ref]) {
        input.parse::<Token![ref]>()?;
        true
    } else {
        is_ref
    };
    let name = input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let type_annotation = input.parse::<Type>()?;
    input.parse::<Token![=]>()?;
    let default_value = input.parse::<Expr>()?;

    Ok(MeaningField {
        is_mut,
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
    if id.to_string() == meaning_name {
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

fn parse_meaning_oop_inheritance_crate_ref(input: ParseStream) -> Result<Path> {
    input.parse::<Token![use]>()?;
    let id = input.parse::<Ident>()?;
    if id.to_string() != "oop_inheritance" {
        id.span().unwrap().error("Identifier must be equals \"oop_inheritance\"").emit();
    }
    input.parse::<Token![=]>()?;
    let path = Path::parse_mod_style(input)?;
    input.parse::<Token![;]>()?;
    Ok(path)
}

#[proc_macro]
pub fn class(input: TokenStream) -> TokenStream {
    let MeaningTree {
        arena_type_name, meanings
    } = parse_macro_input!(input as MeaningTree);

    let mut expanded = TokenStream::new();

    //

    expanded
}