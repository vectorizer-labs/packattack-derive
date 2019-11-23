#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;

extern crate proc_macro;

extern crate proc_macro2;

use proc_macro::TokenStream;

use syn::{Data, Meta };

mod enum_from_bytes;
mod enum_from_byte;

mod struct_from_bytes;


#[proc_macro_derive(FromBytes,attributes(start_byte,end_byte))]
pub fn from_bytes(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    match ast.data {
        Data::Enum(ref data_enum) =>
        {
            enum_from_bytes::enum_from_bytes(&data_enum.variants, name)
        },
        Data::Struct(ref data_struct) =>
        {
            struct_from_bytes::struct_from_bytes(&data_struct.fields, name)
        }
        _ => panic!(
            "`FromBytes` can be applied only to enums or structs, {} is neither",
            name
        ),
    }
}

#[proc_macro_derive(FromByte, attributes(size_in_bits, bitmask))]
pub fn from_byte(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    //gather size and bitmask attributes
    let size_in_bits : Option<syn::Lit> = get_attribute_value(&ast, "size_in_bits");

    //gather size and bitmask attributes
    let byte_size : Option<syn::Lit> = get_attribute_value(&ast, "repr(u8)");

    let size_in_bits_value = match size_in_bits
    {
        Some(sib) => 
        {
            (quote!{#sib}).to_string().parse::<usize>().unwrap()
        },
        None => panic!("No size_in_bits attribute found!")
    };

    //TODO parse attributes and pass them down
    match ast.data {
        Data::Enum(ref data_enum) =>
        {
            enum_from_byte::enum_from_byte(&data_enum.variants, name, size_in_bits_value)
        },
        _ => panic!(
            "deriving `FromByte` can be applied only to enums, {} is neither",
            name
        ),
    }
}

fn get_attribute_value(ast: &syn::DeriveInput, token : &str) -> Option<syn::Lit>
{
    for attr in ast.attrs.iter()
    {
        match attr.parse_meta()
        {
            Ok(meta_attribute) =>
            {
                match meta_attribute
                {
                    Meta::NameValue(meta_name_value) => 
                    {
                        let path_to_print = &meta_name_value.path;

                        //println!("Meta Value {}", quote!{#path_to_print}.to_string().as_str());

                        match &*quote!{#path_to_print}.to_string() == token
                        {
                            true => return Some(meta_name_value.lit),
                            false => return None
                        }
                        
                    },
                    Meta::Path(_path) => {},
                    Meta::List(_meta_list) => {}
                }
            },
            _ => return None
        }
    }

    return None
}


