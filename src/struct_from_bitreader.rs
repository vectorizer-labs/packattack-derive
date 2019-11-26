extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields};

pub fn struct_from_bitreader(fields : &Fields, name : &proc_macro2::Ident) -> TokenStream
{
    let clauses = match fields 
    {
        Fields::Named(fields_named) => 
        {
            let clauses : Vec<proc_macro2::TokenStream> = fields_named.named
            .iter()
            .map(|field| 
            {
                //get the name of the field
                let identifier = match &field.ident
                {
                    Some(n) => n,
                    None => panic!("Found named field without name.")
                };

                let derivable = &field.ty;

                //otherwise we're still inside the byte so just read from the byte
                let clause = quote! 
                {
                    #identifier : <#derivable>::from_bitreader(reader).unwrap()
                };

                clause
            }).collect();

            clauses
        },
        _ => panic!("Packattack only supports reading from structs with named fields")

    };

    let blah = quote!{
        use bitreader::BitReader;
        use std::error::Error;
        use std::result;

        pub type Result<T> = result::Result<T, Box<dyn Error>>;

        impl FromBitReader for #name 
        {
            #[allow(trivial_numeric_casts)]
            fn from_bitreader(reader : &mut BitReader) -> Result<#name>
            {
                Ok(#name
                {
                    #(#clauses),*
                })
            }
        }
    };

    //println!("{}", blah);

    blah.into()
}