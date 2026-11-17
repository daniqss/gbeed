//! Proc macros used by `gbeed-core` to derive its C interface from the platform traits,
//! so the `#[repr(C)]` callback structs can never get out of sync with the Rust traits.

mod callbacks;
mod common;
mod controller;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, ItemTrait, parse_macro_input, spanned::Spanned};

/// Generates, besides the unchanged trait:
/// - `Gbeed<Trait>`: a `#[repr(C)]` struct with an optional `extern "C"` function pointer per method,
///   receiving the host `user_data` followed by the method arguments.
/// - `impl<D: Trait> Trait for crate::ffi::FfiController<D>`: methods without return value run the
///   fallback `D` and then the callback, if set. Methods returning a value use the callback if set,
///   or the fallback otherwise.
///
/// Arguments must be primitives, which are forwarded to C, or references (like `&Ppu`),
/// which are only given to the fallback. Return values must be primitives.
#[proc_macro_attribute]
pub fn ffi_callbacks(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand(attr, item, callbacks::expand)
}

/// Generates, besides the unchanged trait, `Gbeed<Trait>`: a `#[repr(C)]` struct with the host
/// `user_data` and a field with the callbacks struct of each supertrait, named after it in snake case.
/// Every supertrait must be annotated with [`macro@ffi_callbacks`].
#[proc_macro_attribute]
pub fn ffi_controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand(attr, item, controller::expand)
}

/// Emits the unchanged trait followed by the code generated from it, or the expansion error
fn expand(
    attr: TokenStream,
    item: TokenStream,
    generate: fn(&ItemTrait) -> syn::Result<TokenStream2>,
) -> TokenStream {
    let item_trait = parse_macro_input!(item as ItemTrait);
    let generated = no_arguments(attr)
        .and_then(|()| generate(&item_trait))
        .unwrap_or_else(Error::into_compile_error);

    quote!(#item_trait #generated).into()
}

fn no_arguments(attr: TokenStream) -> syn::Result<()> {
    match attr.is_empty() {
        true => Ok(()),
        false => Err(Error::new(
            TokenStream2::from(attr).span(),
            "this attribute takes no arguments",
        )),
    }
}
