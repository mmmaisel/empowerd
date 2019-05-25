#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitInt};

#[proc_macro_derive(InfluxLoad)]
pub fn derive_influx_load(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = ast.ident;
    let mut indices: Vec<LitInt> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut types: Vec<syn::Type> = Vec::new();

    let struct_data = match ast.data
    {
        syn::Data::Struct(data) => data,
        _ => panic!("not a struct")
    };

    match struct_data.fields
    {
        syn::Fields::Named(fields) =>
        {
            for (i, field) in fields.named.into_iter().enumerate()
            {
                let name = field.ident.expect("missing field name");
                indices.push(LitInt::new(i as u64,
                    syn::IntSuffix::None, Span::call_site()));
                names.push(name.to_string());
                types.push(field.ty);
            }
        }
        _ => panic!("not a normal struct")
    }

    let raw_mapping_types = vec!
    [
        syn::parse_str::<syn::Path>("Option<usize>").unwrap();
        names.len()
    ];
    let raw_mapping_values = vec!
    [
        Ident::new("None", Span::call_site());
        names.len()
    ];
    let mapping_values = vec![0usize; names.len()];

    let as_types: Vec<Ident> = types.iter().map(|ty|
    {
        let mut method_name = "as_".to_string();
        method_name.push_str(&quote!(#ty).to_string());
        return Ident::new(&method_name, Span::call_site())
    }).collect();

    let type_names: Vec<String> = types.iter().map(|ty|
    {
        return quote!(#ty).to_string();
    }).collect();

    let fields: Vec<Ident> = names.iter().map(|name|
    {
        return Ident::new(name, Span::call_site());
    }).collect();

    for name in names.iter_mut()
    {
        if name == "timestamp"
        {
            *name = "time".to_string();
        }
    }

    let indices1 = indices.clone();
    let indices2 = indices.clone();
    let indices3 = indices.clone();

    let fields1 = fields.clone();
    let fields2 = fields.clone();

    let load_fn_body = quote!
    {
        let mut raw_mapping: (#(#raw_mapping_types),*) =
            (#(#raw_mapping_values),*);

        for (i, col_name) in series.columns.iter().enumerate()
        {
            match col_name.as_ref()
            {
                #(#names =>
                  {
                      raw_mapping.#indices = Some(i)
                  }
                ),*,
                _ =>
                {
                    return Err("mapping error".to_string())
                }
            }
        }

        let mut mapping = (#(#mapping_values),*);
        #(mapping.#indices1 = match raw_mapping.#indices2
        {
            Some(x) => x,
            None => return Err("unmapped index".to_string())
        };)*

        let data: Result<Vec<#struct_name>, String> =
            series.values.into_iter().map(|val|
        {
            #(let #fields: #types = match &val[mapping.#indices3]
            {
                Number(x) => match x.#as_types()
                {
                    Some(x) => x,
                    None => return Err(format!(
                        "cannot convert to {}", #type_names))
                }
                _ => return Err("expected number".to_string())
            };)*

            return Ok(#struct_name
            {
                #(#fields1: #fields2),*
            });
        }).collect();
        return data;
    };

    let result = quote!
    {
        impl #struct_name
        {
            pub fn load(conn: &influx_db_client::Client, query: String)
                -> Result<Vec<#struct_name>, String>
            {
                let series = load_raw(conn, query)?;
                #load_fn_body
            }
        }
    };

    return TokenStream::from(result);
}
