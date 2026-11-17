//! Expansion of [`macro@crate::ffi_callbacks`]

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Error, FnArg, Ident, ItemTrait, Pat, PatType, Receiver, ReturnType, Signature, TraitItem,
    TraitItemFn, Type, spanned::Spanned,
};

use crate::common::{docs, field_ident, is_primitive, reject_generics, struct_ident};

pub fn expand(item_trait: &ItemTrait) -> syn::Result<TokenStream2> {
    reject_generics(&item_trait.generics, "generic traits are not supported")?;

    let trait_ident = &item_trait.ident;
    let c_struct = struct_ident(trait_ident);
    let field = field_ident(trait_ident);

    let methods = item_trait
        .items
        .iter()
        .map(|item| match item {
            TraitItem::Fn(method) => Method::parse(method),
            _ => Err(Error::new(item.span(), "only methods are supported")),
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let fields = methods.iter().map(Method::callback_field);
    let impls = methods.iter().map(|method| method.forwarding_impl(&field));

    let struct_doc = format!(
        "C callbacks of the `{trait_ident}` trait, every NULL callback is skipped.\n\
         Callbacks returning nothing run after the default implementation, \
         the ones returning a value replace it."
    );

    Ok(quote! {
        #[doc = #struct_doc]
        #[repr(C)]
        #[derive(Debug, Default, Clone, Copy)]
        pub struct #c_struct {
            #(#fields)*
        }

        impl<D: #trait_ident> #trait_ident for crate::ffi::FfiController<D> {
            #(#impls)*
        }
    })
}

/// Trait method validated to be expressible as a C callback
struct Method<'a> {
    docs: Vec<&'a Attribute>,
    ident: &'a Ident,
    receiver: &'a Receiver,
    args: Vec<Arg<'a>>,
    output: &'a ReturnType,
    returns_value: bool,
}

/// Method argument, besides the receiver
struct Arg<'a> {
    name: Ident,
    ty: &'a Type,
    /// Primitives are forwarded to C, references are only given to the fallback
    to_c: bool,
}

impl<'a> Method<'a> {
    fn parse(method: &'a TraitItemFn) -> syn::Result<Self> {
        let sig = &method.sig;
        check_signature(sig)?;

        let mut inputs = sig.inputs.iter();
        let receiver = receiver(inputs.next(), sig)?;
        let args = inputs
            .enumerate()
            .map(|(i, input)| Arg::parse(i, input))
            .collect::<syn::Result<_>>()?;

        Ok(Self {
            docs: docs(&method.attrs).collect(),
            ident: &sig.ident,
            receiver,
            args,
            output: &sig.output,
            returns_value: returns_value(&sig.output)?,
        })
    }

    fn c_args(&self) -> impl Iterator<Item = &Arg<'a>> { self.args.iter().filter(|arg| arg.to_c) }

    /// Optional `extern "C"` function pointer receiving `user_data` and the C arguments
    fn callback_field(&self) -> TokenStream2 {
        let Self {
            docs, ident, output, ..
        } = self;
        let c_names = self.c_args().map(|arg| &arg.name);
        let c_types = self.c_args().map(|arg| arg.ty);

        quote! {
            #(#docs)*
            pub #ident: ::core::option::Option<
                unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void, #(#c_names: #c_types),*) #output
            >,
        }
    }

    /// Trait method of `FfiController`, combining the fallback with the callback stored in `field`
    fn forwarding_impl(&self, field: &Ident) -> TokenStream2 {
        let Self {
            ident,
            receiver,
            output,
            ..
        } = self;
        let names = self.args.iter().map(|arg| &arg.name).collect::<Vec<_>>();
        let types = self.args.iter().map(|arg| arg.ty);
        let c_names = self.c_args().map(|arg| &arg.name);

        let callback = quote!(self.callbacks.#field.#ident);
        let fallback = quote!(self.fallback.#ident(#(#names),*));
        // SAFETY: the host guarantees its callbacks are valid for the `user_data` it registered
        let call = quote!(unsafe { callback(self.callbacks.user_data, #(#c_names),*) });

        let body = match self.returns_value {
            true => quote! {
                match #callback {
                    Some(callback) => #call,
                    None => #fallback,
                }
            },
            false => quote! {
                #fallback;
                if let Some(callback) = #callback {
                    #call
                }
            },
        };

        quote! {
            #[inline]
            fn #ident(#receiver, #(#names: #types),*) #output { #body }
        }
    }
}

impl<'a> Arg<'a> {
    fn parse(i: usize, input: &'a FnArg) -> syn::Result<Self> {
        let FnArg::Typed(PatType { pat, ty, .. }) = input else {
            unreachable!("receiver can only be the first input")
        };

        let to_c = match &**ty {
            Type::Reference(_) => false,
            ty if is_primitive(ty) => true,
            ty => {
                return Err(Error::new(
                    ty.span(),
                    "arguments must be primitives, forwarded to C, or references, only given to the fallback",
                ));
            }
        };

        Ok(Self {
            name: arg_name(i, pat),
            ty,
            to_c,
        })
    }
}

fn check_signature(sig: &Signature) -> syn::Result<()> {
    reject_generics(&sig.generics, "generic methods are not supported")?;

    match sig
        .asyncness
        .as_ref()
        .map(Spanned::span)
        .or(sig.unsafety.map(|u| u.span))
    {
        Some(token) => Err(Error::new(token, "async and unsafe methods are not supported")),
        None => Ok(()),
    }
}

fn receiver<'a>(first: Option<&'a FnArg>, sig: &Signature) -> syn::Result<&'a Receiver> {
    match first {
        Some(FnArg::Receiver(receiver)) if receiver.reference.is_some() => Ok(receiver),
        _ => Err(Error::new(sig.span(), "methods must take `&self` or `&mut self`")),
    }
}

/// Argument name without leading underscores, so it can be used, or `arg{i}` if there is none
fn arg_name(i: usize, pat: &Pat) -> Ident {
    let Pat::Ident(pat) = pat else {
        return format_ident!("arg{i}");
    };

    match pat.ident.to_string().trim_start_matches('_') {
        "" => format_ident!("arg{i}"),
        trimmed => format_ident!("{trimmed}"),
    }
}

fn returns_value(output: &ReturnType) -> syn::Result<bool> {
    match output {
        ReturnType::Default => Ok(false),
        ReturnType::Type(_, ty) if is_primitive(ty) => Ok(true),
        ReturnType::Type(_, ty) => Err(Error::new(ty.span(), "return type must be a primitive")),
    }
}
