//! Helper proc-macros.
//!
//! TODO!

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

extern crate proc_macro;
use proc_macro2::Span;
use quote::quote_spanned;

/// Report an error with the given `span` and message.
#[allow(unused)]
pub(crate) fn spanned_err(span: Span, msg: impl Into<String>) -> proc_macro::TokenStream {
    let msg = msg.into();
    quote_spanned!(span => {
        compile_error!(#msg);
    })
    .into()
}

// pub mod lifetime_to_identifier;

// pub use lifetime_to_identifier::lifetime_to_ident;

use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse::Result as ParseResult, Ident};
use syn::{parse_macro_input, Lifetime};

/// Takes a lifetime and turns it into an identifier.
#[proc_macro]
pub fn lifetime_to_ident(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as Lifetime);

    proc_macro::TokenStream::from(quote!(#input.ident))
}

struct LifetimeVarDecl {
    ident: Ident,
    rest: proc_macro2::TokenTree,
}

impl Parse for LifetimeVarDecl {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        let lifetime = input.parse::<Lifetime>()?;

        let tree = input.cursor().token_tree().ok_or_else(|| {
            syn::Error::new(Span::call_site(), "Expected a lifetime and a token tree.")
        })?;

        Ok(LifetimeVarDecl {
            ident: lifetime.ident,
            rest: tree.0,
        })
    }
}

#[proc_macro]
pub fn create_label(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as LifetimeVarDecl);

    let ident = input.ident;
    let tree = input.rest;

    proc_macro::TokenStream::from(quote!(let #ident: Addr = #tree;))
}

use syn::DeriveInput;

// TODO: this should eventually become the macro we've waiting for, for ordered enums
// with variants that have no associated data. It should bestow upon such enums:
//  - an associated const specifying the number of enums
//  - optionally, to and from impls for the variant's number (i.e. 0 -> R0, R0 -> 0)
//  - optionally, an array type w/Deref+Index impls
//  - a display impl (indep of Debug? not sure)
#[proc_macro_derive(DisplayUsingDebug)]
pub fn derive_display_from_debug(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let ty_name = item.ident;

    quote! (
        impl core::fmt::Display for #ty_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <Self as core::fmt::Debug>::fmt(self, f)
            }
        }
    ).into()
}
