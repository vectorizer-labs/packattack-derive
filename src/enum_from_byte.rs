extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::{TokenStream, Literal};

use syn::{Data, Fields, Variant, punctuated,token };

macro_rules! build_binary_literal {
    () => {
        
    };
}

pub fn enum_from_byte(variants: &punctuated::Punctuated<Variant, token::Comma>, 
                      name : &proc_macro2::Ident, 
                      size_in_bits : usize) -> TokenStream
{

    let clauses: Vec<_> = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let discriminant = &variant.discriminant;
            match discriminant
            {
                Some((_eq,express)) =>
                {
                    quote!
                    {
                        #express => #name::#ident,
                    }
                },
                None => { panic!("must define discriminant") }
            }
        })
        .collect();

    
    let u8_size_in_bits : u8 = size_in_bits as u8;

    //convert our number of bits to a bitmask
    let bitmask_string : u8 = 0b1111_1111 << (8u8 - u8_size_in_bits);

    let blah = quote! {
        impl FromByte for #name {
            #[inline(always)]
            fn read_from_byte(byte : &u8, start_index : u8) -> #name
            {
                let value : u8 = byte & (<#name>::bitmask >> start_index);

                //shift the bits back for evaluation
                match value >> start_index
                {
                    #(#clauses)*
                    _ => panic!("uh oh no match")
                }
            }

            const size_in_bits : u8 = #u8_size_in_bits;
            const bitmask : u8 = #bitmask_string; 
        }
    };

    //println!("{}", blah);

    blah.into()
}