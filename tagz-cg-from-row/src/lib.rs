extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(FromRow, attributes(field_default))]
pub fn from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let implementation = match input.data {
        Data::Struct(ref ds) => match ds.fields {
            Fields::Unnamed(ref list) => {
                let fields = list.unnamed.iter().enumerate().map(|(idx, field)| {
                    if has_skip(&field) {
                        quote! {
                            Default::default()
                        }
                    } else {
                        quote! {
                            row.get(#idx)?
                        }
                    }
                });

                quote! {
                    ( #( #fields ),* )
                }
            }

            Fields::Named(ref list) => {
                let fields = list.named.iter().enumerate().map(|(_idx, field)| {
                    let ident = &field.ident;

                    if has_skip(&field) {
                        quote! {
                            #ident: Default::default()
                        }
                    } else {
                        quote! {
                            #ident: row.get(stringify!(#ident))?
                        }
                    }
                });

                quote! { { #( #fields ),* } }
            }

            _ => panic!("unsupported"),
        },
        _ => panic!("unsupported"),
    };

    let output = quote! {
        impl crate::from_row::FromRow for #ident {
            fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
                Ok(Self #implementation)
            }
        }
    };

    TokenStream::from(output)
}

fn has_skip(f: &syn::Field) -> bool {
    f.attrs
        .iter()
        .find(|section| match section.parse_meta().unwrap() {
            Meta::Path(path) => path.segments[0].ident == "field_default",

            _ => false,
        })
        .is_some()
}
