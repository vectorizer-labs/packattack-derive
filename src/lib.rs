#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Data};


mod struct_compiler;
mod enum_compiler;

#[allow(dead_code)]
mod util;

use util::{ ident_from_string, expr_from_meta_attribute };

#[proc_macro_derive(FromReader, attributes(size_in_bits, from_reader,expose, flag, length))]
pub fn from_reader(input: TokenStream) -> TokenStream 
{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    match ast.data {
        Data::Enum(ref data_enum) =>
        {
            //gather size attribute
            //we need to know the size of the discriminant of the enum 
            //so we can allocate the right size for it
            let size_in_bits : syn::Expr = match util::get_meta_attribute(&ast.attrs, "size_in_bits")
            {
                Some(attr) => expr_from_meta_attribute(attr),
                None => panic!("No size_in_bits attribute found!")
            };

            let wrp = wrap_from_reader(enum_compiler::enum_from_reader(&data_enum.variants, name, &size_in_bits), name);
            eprintln!("{}",wrp);
            wrp
        }
        Data::Struct(ref data_struct) =>
        {
            let wrp = wrap_from_reader(struct_compiler::get_struct(&data_struct.fields, name), name);
            eprintln!("{}",wrp);
            wrp
        },
        _ => panic!(
            "deriving `FromBytes` can be applied only to enums and structs, {} is neither",
            name
        ),
    }
}

fn wrap_from_reader(body : proc_macro2::TokenStream, name : &syn::Ident) -> TokenStream
{
    (quote!{
        #[async_trait]
        impl<R> FromReader<crate::ERROR, R> for #name 
        where Self : Sized,
            R : Read + std::marker::Unpin + std::marker::Send
        {
            #[allow(trivial_numeric_casts)]
            async fn from_reader(reader : &mut R) -> Result<Self, crate::ERROR>
            {
                #body
            }
        }
    }).into()
}

#[proc_macro_derive(FromBytes, attributes(size_in_bits))]
pub fn from_bytes(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let size_in_bits : syn::Expr = match util::get_meta_attribute(&ast.attrs, "size_in_bits")
    {
        Some(attr) => expr_from_meta_attribute(attr),
        None => panic!("No size_in_bits attribute found!")
    };
    
    match ast.data 
    {
        Data::Enum(ref data_enum) =>
        {
            let wrp = wrap_from_byte(enum_compiler::enum_from_bytes(&data_enum.variants, name, &size_in_bits), name, &size_in_bits);
            //eprintln!("{}",wrp);
            wrp
        }
        Data::Struct(ref data_struct) =>
        {
            let wrp = wrap_from_byte(struct_compiler::get_struct(&data_struct.fields, name), name, &size_in_bits);
            //eprintln!("{}",wrp);
            wrp
        },
        _ => panic!(
            "deriving `FromBytes` can be applied only to enums and structs, {} is neither",
            name
        ),
    }
}

fn wrap_from_byte(body : proc_macro2::TokenStream, 
                  name : &syn::Ident, size_in_bits : &syn::Expr) -> TokenStream
{

    let mut array_size_name = "arrray_size_of_".to_owned();
    array_size_name.push_str(name.to_string().as_str());

    let array_size_ident = ident_from_string(array_size_name);

    (quote!{

        //TODO: move the creation of this variable down to fields.rs
        //change #size_in_bits to the actual size of the whole struct

        const #array_size_ident : usize = ((8 - (#size_in_bits & 7)) & 7);

        impl Bitsize for #name
        {
            const SIZE_IN_BITS : u8 = #size_in_bits;
        }

        impl FromBytes<crate::ERROR, [u8; #array_size_ident]> for #name
        {
            fn from_bytes(slice : [u8; #array_size_ident]) -> std::result::Result<Self, crate::ERROR> 
            { 
                #body
            }
        } 
    }).into()
}