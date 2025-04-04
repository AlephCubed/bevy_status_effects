use proc_macro_error::proc_macro_error;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput};

#[cfg(feature = "bevy_butler")]
use proc_macro2::Span;
#[cfg(feature = "bevy_butler")]
use syn::Token;

#[proc_macro_derive(StatusEffect, attributes(add_component))]
#[proc_macro_error]
pub fn stat_container_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree: DeriveInput = syn::parse(item).expect("TokenStream must be valid.");

    #[cfg(feature = "bevy_butler")]
    let mut systems = Vec::new();

    let struct_name = &tree.ident;

    for attr in &tree.attrs {
        let path = attr.meta.path();

        let Some(ident) = path.get_ident() else {
            continue;
        };

        match ident.to_string().as_str() {
            #[cfg(feature = "bevy_butler")]
            "add_component" => parse_add_component(attr, struct_name, &mut systems),
            _ => continue,
        };
    }

    #[cfg(feature = "bevy_butler")]
    return quote! {
        impl bevy_status_effects::StatusEffect for #struct_name {}
        #(#systems)*
    }
    .into();

    #[cfg(not(feature = "bevy_butler"))]
    quote! {
        impl bevy_status_effects::StatusEffect for #struct_name {}
        #trait_impl
    }
    .into()
}

#[cfg(feature = "bevy_butler")]
fn parse_add_component(attr: &Attribute, struct_name: &Ident, systems: &mut Vec<TokenStream>) {
    attr.parse_nested_meta(|meta| {
        let Some(var_name) = meta.path.segments.first() else {
            return Ok(())
        };

        if var_name.ident.to_string() != "plugin" {
            return Ok(())
        };

        let input = &meta.input;

        input.parse::<Token![=]>().expect("An equals sign.");
        let input = input.parse::<Ident>().expect("An identifier.");

        let use_as = Ident::new(&format!("__{struct_name}_init_effect_hook"), Span::call_site());

        systems.push(quote! {
            #[bevy_butler::add_system(generics = <#struct_name>, plugin = #input, schedule = bevy_status_effects::Startup)]
            use bevy_status_effects::init_effect_hook as #use_as;
        });

        Ok(())
    }).unwrap();
}
