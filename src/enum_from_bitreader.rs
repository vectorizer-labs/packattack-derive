extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields, Variant, punctuated,token };

pub fn enum_from_bitreader(variants: &punctuated::Punctuated<Variant, token::Comma>, 
    name : &proc_macro2::Ident, 
    size_in_bits : usize) -> TokenStream
{
    let clauses: Vec<_> = variants
        .iter()
        .map(|variant| 
        {
            let ident = &variant.ident;
            let discriminant = &variant.discriminant;
            match &variant.fields {
                Fields::Unit => 
                {
                    match discriminant
                    {
                        Some((_eq,express)) =>
                        {
                            quote!  {
                                #express => Ok(#name::#ident),
                            }
                        },
                        None => { panic!("must define discriminant") }
                    }
                    
                },
                Fields::Unnamed(fields_unnamed) => 
                {
                    let fields : Vec::<_> = fields_unnamed.unnamed
                    .iter().map(|field|
                    {
                        quote!
                        {
                            <#field>::from_bitreader(reader).unwrap()
                        }
                    })
                    .collect();

                    match discriminant
                    {
                        Some((_eq,express)) =>
                        {

                            quote!
                            {
                                #express => Ok(#name::#ident(#(#fields),*)),
                            }
                        },
                        None => { panic!("must define discriminant") }
                    }
                        
                },
                Fields::Named(_fields_named) => 
                {
                    panic!("You found a Named Field... Packattack doesn't support these at the moment.")//anonymous struct variant
                }
            }
        })
        .collect();

    let reader_literal = get_reader_literal(size_in_bits);

    let blah = quote! {
        use bitreader::BitReader;
        use std::result;

        //TODO: Expand this definition to more errors
        /// Result type for those BitReader operations that can fail.
        pub type Result<T> = result::Result<T, bitreader::BitReaderError>;

        impl FromBitReader for #name {
            fn from_bitreader(reader : &mut BitReader) -> Result<#name>
            {
                match #reader_literal
                {
                    #(#clauses)*
                    _ => panic!("uh oh no match")
                }
            }
        }
    };

    //println!("{}", blah);

    blah.into()
}

fn get_reader_literal(size_in_bits : usize) -> proc_macro2::TokenStream
{   
    match size_in_bits
    {
        0 => panic!("Tried to build a reader for with a size of 0. Did you forget to put [size_in_bytes]?"),
        1..=8 => quote!{ reader.read_u8(#size_in_bits as u8).unwrap() },
        9..=16 => quote!{ reader.read_u16(#size_in_bits as u16).unwrap()  },
        17..=32 => quote!{ reader.read_u32(#size_in_bits as u32).unwrap() },
        33..=64 => quote!{ reader.read_u64(#size_in_bits as u64).unwrap() },
        _ => panic!("Tried to buld a reader with a size of greater than 64 bits! This isn't yet supported.")
    }
}