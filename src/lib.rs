#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;

extern crate proc_macro;

extern crate proc_macro2;

use proc_macro::TokenStream;

use syn::{Data};

mod enum_from_bitreader;
mod struct_from_bitreader;
mod util;

#[proc_macro_derive(FromBitReader, attributes(size_in_bits, expose, flag, length))]
pub fn from_bit_reader(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    match ast.data {
        Data::Enum(ref data_enum) =>
        {
            //gather size attribute
            //we need to know the size of the discriminant of the enum 
            //so we can allocate the right size for it
            if let Some(size_in_bits) = util::get_meta_attribute(&ast.attrs, "size_in_bits")
            {
                enum_from_bitreader::enum_from_bitreader(&data_enum.variants, name, size_in_bits)
            }
            else
            {
                panic!("No size_in_bits attribute found!")
            }
        }
        Data::Struct(ref data_struct) =>
        {
            struct_from_bitreader::struct_from_bitreader(&data_struct.fields, name)
        },
        _ => panic!(
            "deriving `FromBitReader` can be applied only to enums and structs, {} is neither",
            name
        ),
    }
}