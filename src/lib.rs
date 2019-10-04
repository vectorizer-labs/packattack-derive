#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;

extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Data };

mod enum_from_bytes;
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

#[proc_macro_derive(FromByte, attributes())]
pub fn from_byte(input: TokenStream) -> TokenStream {
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


