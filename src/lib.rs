#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Data};


mod struct_compiler;
//mod enum_compiler;
mod attributes;

use attributes::{ ParentDataType };
//use struct_compiler::bit_index::{ bits_to_byte_ceiling };

#[allow(dead_code)]
mod util;

//use util::ident_from_str;

#[proc_macro_derive(FromBits, attributes(size_in_bits))]
pub fn from_bits(input: TokenStream) -> TokenStream 
{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident.clone();

    let (fin_data_structure, size_in_bits) = build_data_structure(ast, &name, ParentDataType::FromBits);

    let fin = quote!{

        impl Bitsize for #name
        {
            const SIZE_IN_BITS : usize = #size_in_bits;
        }

        impl FromBits<crate::ERROR> for #name
        {
            fn from_bits(byte : u8) -> std::result::Result<Self, crate::ERROR> 
            {
                #fin_data_structure
            }
        } 
    };

    //eprintln!("{}", fin);

    fin.into()
}


#[proc_macro_derive(FromReader, attributes(size_in_bits, str, from_bytes, from_bits, expose, flag, length, discriminant, hint, payload))]
pub fn from_reader(input: TokenStream) -> TokenStream 
{
    let ast : syn::DeriveInput = syn::parse(input).unwrap();

    let name = ast.ident.clone();

    let (fin_data_structure, size_in_bits) = build_data_structure(ast, &name, ParentDataType::FromBytes);

    let fin = quote!{
        #[async_trait]
        impl<R> FromReader<crate::ERROR, R> for #name 
        where Self : Sized, R : Read + std::marker::Unpin + std::marker::Send
        {
            async fn from_reader(reader : &mut R) -> Result<Self, crate::ERROR>
            {
                #fin_data_structure
            }
        }
    };

    //eprintln!("{}", fin);

    fin.into()
}

/*
#[proc_macro_derive(FromBytes, attributes(size_in_bits, from_bits, from_bytes))]
pub fn from_bytes(input: TokenStream) -> TokenStream 
{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = ast.ident.clone();

    let (fin_data_structure, size_in_bits) = build_data_structure(ast, &name, ParentDataType::FromBytes);

    let fin = quote!{

        const #const_name : usize = #size;

        impl Bitsize for #name
        {
            const SIZE_IN_BITS : usize = #size_in_bits;
        }

        impl FromBytes<crate::ERROR,[u8; #const_name]> for #name
        {
            fn from_bytes(bytes : [u8; #const_name]) -> std::result::Result<Self, crate::ERROR> 
            {
                #fin_data_structure
            }
        } 
    };

    //eprintln!("{}", fin);

    fin.into()
}*/


fn build_data_structure(input: syn::DeriveInput, name : &syn::Ident, parent_data_type : ParentDataType) 
-> (proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    match input.data 
    {
        Data::Enum(ref data_enum) =>
        {
            unimplemented!("OOF!");
            /*
            let attri = get_enum_attributes(&input.attrs, &parent_data_type);

            let fin_enum = enum_compiler::get_enum(&data_enum.variants, &name, &attri, parent_data_type);
            let size_in_bits = attri.size_in_bits;

            (fin_enum, quote!{ #size_in_bits })*/
        }
        Data::Struct(ref data_struct) =>
        {
            struct_compiler::get_struct(&data_struct.fields, name, parent_data_type)
        },
        _ => panic!(
            "Deriving `FromBytes`, `FromBytes`, and `FromReader` can be applied only to enums and structs, {} is neither",
            name
        ),
    }
}