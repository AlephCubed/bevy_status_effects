#[cfg(feature = "bevy_butler")]
mod bevy_butler;

use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(StatusEffect, attributes(add_component))]
#[proc_macro_error]
pub fn stat_container_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree: DeriveInput = parse_macro_input!(item as DeriveInput);
    let ident = &tree.ident;

    let trait_impl = quote! {
        impl StatusEffect for #ident {}
    };

    #[cfg(feature = "bevy_butler")]
    {
        let systems =
            bevy_butler::register_systems(tree).unwrap_or_else(darling::Error::write_errors);
        quote! { #trait_impl #systems }.into()
    }

    #[cfg(not(feature = "bevy_butler"))]
    trait_impl.into()
}
