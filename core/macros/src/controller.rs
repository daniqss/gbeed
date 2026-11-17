//! Expansion of [`macro@crate::ffi_controller`]

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, Ident, ItemTrait, TypeParamBound, spanned::Spanned};

use crate::common::{field_ident, reject_generics, struct_ident};

pub fn expand(item_trait: &ItemTrait) -> syn::Result<TokenStream2> {
    reject_generics(&item_trait.generics, "generic traits are not supported")?;

    let c_struct = struct_ident(&item_trait.ident);
    let supertraits = item_trait
        .supertraits
        .iter()
        .map(supertrait)
        .collect::<syn::Result<Vec<_>>>()?;

    let fields = supertraits.iter().copied().map(field_ident).collect::<Vec<_>>();
    let types = supertraits.iter().copied().map(struct_ident);

    let struct_doc = format!(
        "C version of the `{}` trait, made of the callbacks of each of its supertraits.\n\
         A zeroed struct is valid, as every NULL callback is skipped.",
        item_trait.ident
    );

    Ok(quote! {
        #[doc = #struct_doc]
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        pub struct #c_struct {
            /// Passed as first argument to every callback
            pub user_data: *mut ::core::ffi::c_void,
            #(pub #fields: crate::ffi::#types,)*
        }

        impl ::core::default::Default for #c_struct {
            fn default() -> Self {
                Self {
                    user_data: ::core::ptr::null_mut(),
                    #(#fields: ::core::default::Default::default(),)*
                }
            }
        }
    })
}

/// Name of a supertrait, which must have been annotated with [`macro@crate::ffi_callbacks`]
fn supertrait(bound: &TypeParamBound) -> syn::Result<&Ident> {
    let TypeParamBound::Trait(bound) = bound else {
        return Err(Error::new(bound.span(), "only trait bounds are supported"));
    };

    bound
        .path
        .segments
        .last()
        .map(|segment| &segment.ident)
        .ok_or_else(|| Error::new(bound.span(), "expected a trait"))
}
