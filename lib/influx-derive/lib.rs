#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

extern crate influx_db_client;
extern crate serde_json;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitInt, Path, Type};

fn get_series_name(attrs: Vec<syn::Attribute>) -> String
{
    let mut series_name = String::new();

    for attr in attrs.iter()
    {
        let meta = attr.interpret_meta().unwrap();
        let meta_list = match meta
        {
            syn::Meta::List(x) => x,
            _ => panic!("unexpected attribute type")
        };

        let name = meta_list.ident.to_string();
        if name != "influx"
        {
            panic!("unexpected attribute {}", name);
        }

        for nested_meta in meta_list.nested
        {
            let denested_meta = match nested_meta
            {
                syn::NestedMeta::Meta(x) => x,
                _ => panic!("unexpected attribute layout")
            };
            let name_value = match denested_meta
            {
                syn::Meta::NameValue(x) => x,
                _ => panic!("unexpected attribute argument")
            };
            let key = name_value.ident.to_string();
            let value = match name_value.lit
            {
                syn::Lit::Str(x) => x.value(),
                _ => panic!("unexpected attribute value type")
            };

            match key.as_ref()
            {
                "measurement_name" => series_name = value,
                _ => panic!("unexpected attribute key")
            }
        }
    }
    return series_name;
}

fn record_info(data: syn::Data, suppress_time: bool)
    -> (Vec<LitInt>, Vec<String>, Vec<Type>)
{
    let mut indices: Vec<LitInt> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut types: Vec<Type> = Vec::new();

    let struct_data = match data
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
                if !suppress_time || name != "timestamp"
                {
                    indices.push(LitInt::new(i as u64,
                        syn::IntSuffix::None, Span::call_site()));
                    names.push(name.to_string());
                    types.push(field.ty);
                }
            }
        }
        _ => panic!("not a normal struct")
    }
    return (indices, names, types);
}

#[proc_macro_derive(InfluxData, attributes(influx))]
pub fn derive_influx_data(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let mut series_name = get_series_name(ast.attrs);
    let struct_name = ast.ident;

    if series_name.is_empty()
    {
        series_name = struct_name.to_string().to_lowercase();
    }

    let load_fn = impl_load_fn(ast.data.clone(), &struct_name, &series_name);
    let to_point_fn = impl_to_point_fn(ast.data.clone(), &series_name);
    let first_fn = impl_first_fn(&struct_name, &series_name);
    let last_fn = impl_last_fn(&struct_name, &series_name);
    let save_fn = impl_save_fn();
    let save_all_fn = impl_save_all_fn(&struct_name);

    let result = quote!
    {
        use influx_db_client::{Client, Point, Precision, Series, Value};
        // TODO: use some traits for this
        impl #struct_name
        {
            #load_fn
            #to_point_fn
            #first_fn
            #last_fn
            #save_fn
            #save_all_fn
        }
    };
    return TokenStream::from(result);
}

fn impl_load_fn(ast_data: syn::Data, struct_name: &Ident, series_name: &String)
    -> proc_macro2::TokenStream
{
    let (indices, mut names, types) = record_info(ast_data, false);

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
        if series.name != #series_name
        {
            return Err(LoadError::new(
                "invalid series name received".to_string()));
        }

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
                    return Err(LoadError::new("mapping error".to_string()))
                }
            }
        }

        let mut mapping = (#(#mapping_values),*);
        #(mapping.#indices1 = match raw_mapping.#indices2
        {
            Some(x) => x,
            None => return Err(LoadError::new("unmapped index".to_string()))
        };)*

        let data: Result<Vec<#struct_name>, LoadError> =
            series.values.into_iter().map(|val|
        {
            #(let #fields: #types = match &val[mapping.#indices3]
            {
                Number(x) => match x.#as_types()
                {
                    Some(x) => x,
                    None => return Err(LoadError::new(format!(
                        "cannot convert to {}", #type_names)))
                }
                _ => return Err(LoadError::new("expected number".to_string()))
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
        pub fn load(conn: &influx_db_client::Client, query: String)
            -> Result<Vec<#struct_name>, LoadError>
        {
            use serde_json::Value::Number;
            let series = load_series(conn, query)?;
            #load_fn_body
        }
    };
    return result;
}

fn impl_to_point_fn(ast_data: syn::Data, series_name: &String)
    -> proc_macro2::TokenStream
{
    let (_, names, types) = record_info(ast_data, true);

    let fields: Vec<Ident> = names.iter().map(|name|
    {
        return Ident::new(name, Span::call_site());
    }).collect();

    let value_types: Vec<Path> = types.iter().map(|ty|
    {
        let typename = quote!(#ty).to_string();
        match typename.as_ref()
        {
            "f64" => syn::parse_str::<syn::Path>("Value::Float").unwrap(),
            _ => panic!("Unsupported type {}", typename)
        }
    }).collect();

    let to_point_fn_body = quote!
    {
        let mut point = Point::new(#series_name);
        point.add_timestamp(self.timestamp);
        #(point.add_field(#names, #value_types(self.#fields));)*
        return point;
    };

    let result = quote!
    {
        pub fn to_point(&self) -> Point
        {
            #to_point_fn_body
        }
    };
    return result;
}

fn impl_first_fn(struct_name: &Ident, series_name: &String)
    -> proc_macro2::TokenStream
{
    let first_fn_body = quote!
    {
        let mut queried = #struct_name::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" ASC LIMIT 1",
            #series_name))?;
        if queried.len() != 1
        {
            return Err(LoadError::new(format!(
                "Received {} series, expected 1", queried.len())));
        }
        return Ok(queried.pop().unwrap());
    };

    let result = quote!
    {
        pub fn first(conn: &Client) -> Result<#struct_name, LoadError>
        {
            #first_fn_body
        }
    };
    return result;
}

fn impl_last_fn(struct_name: &Ident, series_name: &String)
    -> proc_macro2::TokenStream
{
    let last_fn_body = quote!
    {
        let mut queried = #struct_name::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" DESC LIMIT 1",
            #series_name))?;
        if queried.len() != 1
        {
            return Err(LoadError::new(format!(
                "Received {} series, expected 1", queried.len())));
        }
        return Ok(queried.pop().unwrap());
    };

    let result = quote!
    {
        pub fn last(conn: &Client) -> Result<#struct_name, LoadError>
        {
            #last_fn_body
        }
    };
    return result;
}

fn impl_save_fn()
    -> proc_macro2::TokenStream
{
    let save_fn_body = quote!
    {
        return match conn.write_point(self.to_point(),
            Some(Precision::Seconds), None)
        {
            Ok(x) =>
            {
                // TODO: use logger
                //println!("wrote {:?} to influx", self);
                Ok(x)
            }
            Err(e) => Err(format!("Writing point to influx failed, {}", e))
        };
    };

    let result = quote!
    {
        pub fn save(&self, conn: &Client) -> Result<(), String>
        {
            #save_fn_body
        }
    };
    return result;
}

fn impl_save_all_fn(struct_name: &Ident)
    -> proc_macro2::TokenStream
{
    let save_all_fn_body = quote!
    {
        let points: Points = data.into_iter().map(|x|
        {
            return x.to_point();
        }).collect();

        return match conn.write_points(points, Some(Precision::Seconds), None)
        {
            Ok(x) =>
            {
                // TODO: use logger
                //println!("wrote points to influx");
                Ok(x)
            }
            Err(e) => Err(format!("Writing points to influx failed, {}", e))
        };
    };

    let result = quote!
    {
        pub fn save_all(conn: &Client, data: Vec<#struct_name>)
            -> Result<(), String>
        {
            #save_all_fn_body
        }
    };
    return result;
}
