use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, Path, Result};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(Error::new_spanned(&input, "Props requires named fields")),
        },
        _ => {
            return Err(Error::new_spanned(
                &input,
                "Props requires a struct with named fields",
            ));
        }
    };
    let mut component: Option<Path> = None;
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("props")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("component") {
                return Err(meta.error("unknown props option; expected component"));
            }
            if component.is_some() {
                return Err(meta.error("duplicate component option"));
            }
            component = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    let component = component
        .ok_or_else(|| Error::new_spanned(&input.ident, "missing #[props(component = path)]"))?;
    let name = &input.ident;
    let vis = &input.vis;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut conversion_generics = input.generics.clone();
    conversion_generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(#name #ty_generics: ::core::cmp::PartialEq + 'static));
    let (conversion_impl_generics, _, conversion_where_clause) =
        conversion_generics.split_for_impl();
    let mut setters = Vec::new();
    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let field_name = ident.unraw().to_string();
        if field_name == "key" || field_name == "child" {
            return Err(Error::new_spanned(ident, "field name is reserved by Props"));
        }
        let mut into = false;
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("props")) {
            attr.parse_nested_meta(|meta| {
                if !meta.path.is_ident("into") {
                    return Err(meta.error("unknown props field option; expected into"));
                }
                if into {
                    return Err(meta.error("duplicate into option"));
                }
                if field_name == "children" {
                    return Err(meta.error("children cannot have #[props(into)]"));
                }
                into = true;
                Ok(())
            })?;
        }
        let docs: Vec<_> = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .collect();
        let ty = &field.ty;
        if field_name == "children" {
            // Spanned on the field type, so a wrong type reports there.
            let list = quote_spanned! {ty.span()=>
                let list: &mut ::std::vec::Vec<::ink_rs::Element> = &mut self.#ident;
            };
            setters.push(quote! {
                #(#docs)*
                #vis fn child(mut self, child: impl ::core::convert::Into<::ink_rs::Element>) -> Self {
                    #list
                    list.push(::core::convert::Into::into(child));
                    self
                }

                #(#docs)*
                #vis fn #ident(
                    mut self,
                    children: impl ::core::iter::IntoIterator<
                        Item = impl ::core::convert::Into<::ink_rs::Element>,
                    >,
                ) -> Self {
                    #list
                    list.extend(
                        ::core::iter::IntoIterator::into_iter(children).map(::core::convert::Into::into),
                    );
                    self
                }
            });
        } else {
            let (argument, value) = if into {
                (
                    quote! { impl ::core::convert::Into<#ty> },
                    quote! { ::core::convert::Into::into(#ident) },
                )
            } else {
                (quote! { #ty }, quote! { #ident })
            };
            setters.push(quote! {
                #(#docs)*
                #vis fn #ident(mut self, #ident: #argument) -> Self {
                    self.#ident = #value;
                    self
                }
            });
        }
    }
    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*

            #vis fn key(self, key: impl ::core::convert::Into<::ink_rs::Key>) -> ::ink_rs::Element
            where
                Self: ::core::cmp::PartialEq + 'static,
            {
                ::ink_rs::Element::from(self).key(key)
            }
        }

        impl #conversion_impl_generics ::core::convert::From<#name #ty_generics>
            for ::ink_rs::Element #conversion_where_clause
        {
            fn from(props: #name #ty_generics) -> Self {
                ::ink_rs::component(#component, props)
            }
        }
    })
}
