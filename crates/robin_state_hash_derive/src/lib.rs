//! `#[derive(StateHash)]` — proc-macro that emits a `StateHash` impl.
//!
//! Why a custom trait instead of `std::hash::Hash`? Floats. `f32` and
//! `f64` don't implement `Hash` because of NaN equality. We want a
//! deterministic byte-identical hash of the engine state that includes
//! float fields, so we define a separate `StateHash` trait whose float
//! impls go through `to_bits()`.
//!
//! The derived impl walks every field in declaration order, calling
//! each field's `StateHash::state_hash`. For enums it hashes the
//! discriminant first, then each variant's fields.
//!
//! `#[serde(skip)]` fields are also skipped from `StateHash` so the
//! derived hash matches the serialized shape — important since
//! `state_hash` is what the rollback checker uses to detect
//! determinism bugs and rolled-back state must hash to the same value
//! as live state.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Index, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(StateHash)]
pub fn derive_state_hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body: TokenStream2 = match input.data {
        Data::Struct(data) => struct_body(&data.fields),
        Data::Enum(data) => {
            let arms: TokenStream2 = data
                .variants
                .iter()
                .enumerate()
                .map(|(disc, variant)| {
                    let variant_ident = &variant.ident;
                    let disc = disc as u64;
                    match &variant.fields {
                        Fields::Unit => quote! {
                            Self::#variant_ident => {
                                ::core::hash::Hasher::write_u64(state, #disc);
                            }
                        },
                        Fields::Named(fields) => {
                            let bindings: Vec<_> = fields
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().unwrap())
                                .collect();
                            let calls = fields.named.iter().map(|f| {
                                let id = f.ident.as_ref().unwrap();
                                hash_call(&f.ty, quote! { #id })
                            });
                            quote! {
                                Self::#variant_ident { #(#bindings),* } => {
                                    ::core::hash::Hasher::write_u64(state, #disc);
                                    #(#calls)*
                                }
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let bindings: Vec<_> = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, f)| syn::Ident::new(&format!("__f{i}"), f.span()))
                                .collect();
                            let calls = bindings
                                .iter()
                                .zip(fields.unnamed.iter())
                                .map(|(b, f)| hash_call(&f.ty, quote! { #b }));
                            quote! {
                                Self::#variant_ident( #(#bindings),* ) => {
                                    ::core::hash::Hasher::write_u64(state, #disc);
                                    #(#calls)*
                                }
                            }
                        }
                    }
                })
                .collect();
            quote! {
                match self {
                    #arms
                }
            }
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(ident, "StateHash cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::robin_util::state_hash::StateHash for #ident #ty_generics #where_clause {
            fn state_hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
                #body
            }
        }
    };

    TokenStream::from(expanded)
}

/// True if any of `attrs` is `#[serde(skip)]` (or `skip_serializing`).
/// We skip those from the hash too, so the hash matches what the
/// snapshot would carry.
fn is_serde_skipped(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut skip = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") || meta.path.is_ident("skip_serializing") {
                skip = true;
            }
            Ok(())
        });
        if skip {
            return true;
        }
    }
    false
}

fn struct_body(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(fs) => {
            let calls = fs.named.iter().filter_map(|f| {
                if is_serde_skipped(&f.attrs) {
                    return None;
                }
                let id = f.ident.as_ref().unwrap();
                Some(hash_call(&f.ty, quote! { self.#id }))
            });
            quote! { #(#calls)* }
        }
        Fields::Unnamed(fs) => {
            let calls = fs.unnamed.iter().enumerate().filter_map(|(i, f)| {
                if is_serde_skipped(&f.attrs) {
                    return None;
                }
                let idx = Index {
                    index: i as u32,
                    span: f.span(),
                };
                Some(hash_call(&f.ty, quote! { self.#idx }))
            });
            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

/// Emit a call that hashes one field. Just delegates to the trait —
/// the float impls in `robin_util::state_hash` handle the to_bits
/// dance, and the macro itself stays type-agnostic.
fn hash_call(_ty: &syn::Type, accessor: TokenStream2) -> TokenStream2 {
    quote! {
        ::robin_util::state_hash::StateHash::state_hash(&#accessor, state);
    }
}
