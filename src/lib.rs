#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;

extern crate proc_macro;

extern crate proc_macro2;

use proc_macro::TokenStream;

use syn::{Data, Meta};

mod enum_from_bitreader;
mod struct_from_bitreader;

#[proc_macro_derive(FromBitReader, attributes(size_in_bits))]
pub fn from_bit_reader(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    //TODO parse attributes and pass them down
    match ast.data {
        Data::Enum(ref data_enum) =>
        {
            //gather size attribute
            //we need to know the size of the discriminant of the enum 
            //so we can allocate the right size for it
            let size_in_bits : Option<syn::Lit> = get_attribute_value(&ast, "size_in_bits");

            let size_in_bits_value = match size_in_bits
            {
                Some(sib) => 
                {
                    //parse and unwrap the size_in_bits
                    (quote!{#sib}).to_string().parse::<usize>().unwrap()
                },
                None => panic!("No size_in_bits attribute found!")
            };

            enum_from_bitreader::enum_from_bitreader(&data_enum.variants, name, size_in_bits_value)
        },
        Data::Struct(ref data_struct) =>
        {
            struct_from_bitreader::struct_from_bitreader(&data_struct.fields, name)
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


