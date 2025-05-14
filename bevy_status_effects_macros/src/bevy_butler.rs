use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Expr, Meta, Path};

/// Returns code that will register the effect hook using Bevy Butler.
pub fn register_systems(input: DeriveInput) -> darling::Result<TokenStream> {
    let struct_name = &input.ident;

    let mut butler_attributes = ButlerAttribute::new(struct_name);

    for attr in input.attrs {
        if attr.path().is_ident("add_component") {
            let plugin = PluginPath::from_meta(&attr.meta)?;
            butler_attributes.plugin = Some(plugin);
        }
    }

    Ok(butler_attributes.into_token_stream())
}

struct ButlerAttribute<'a> {
    ident: &'a Ident,
    plugin: Option<PluginPath>,
}

impl<'a> ButlerAttribute<'a> {
    pub fn new(ident: &'a Ident) -> Self {
        Self {
            ident,
            plugin: None,
        }
    }
}

impl<'a> ToTokens for ButlerAttribute<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(plugin_path) = &self.plugin {
            let ident = &self.ident;
            let plugin = &plugin_path.0;
            let use_as = format_ident!("__{ident}_status_effect");

            // Due to some strange import scoping issues, we cannot use the plugins.
            // Instead, we can just recreate the plugin's functionality.
            tokens.extend(quote! {
                #[bevy_butler::add_system(
                    generics = <#ident>,
                    plugin = #plugin,
                    schedule = bevy_status_effects::__Startup
                )]
                use bevy_status_effects::init_effect_hook as #use_as;
            });
        }
    }
}

/// Represents a `plugin(PATH)` or `plugin = PATH` attribute meta.
pub struct PluginPath(pub Path);

impl FromMeta for PluginPath {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        for item in items {
            return match item {
                NestedMeta::Meta(meta) => match meta {
                    Meta::Path(_) => Err(Error::custom("Expected a value for `plugin`")),
                    Meta::List(list) => {
                        if list.path.require_ident()? != "plugin" {
                            continue;
                        }

                        let mut path = None;

                        list.parse_nested_meta(|value_meta| {
                            path = Some(value_meta.path);
                            Ok(())
                        })?;

                        match path {
                            None => Err(Error::custom("Expected `plugin` attribute")),
                            Some(path) => Ok(PluginPath(path)),
                        }
                    }
                    Meta::NameValue(name_value) => match &name_value.value {
                        Expr::Path(p) => Ok(PluginPath(p.path.clone())),
                        _ => Err(Error::custom("Expected a path to a butler plugin")),
                    },
                },
                NestedMeta::Lit(_) => Err(Error::custom("Expected `plugin` attribute")),
            };
        }

        Err(Error::custom("Expected `plugin` attribute"))
    }
}
