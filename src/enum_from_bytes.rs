extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields, Variant, punctuated,token };

pub fn enum_from_bytes(variants: &punctuated::Punctuated<Variant, token::Comma>, name : &proc_macro2::Ident) -> TokenStream
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
                                #express => #name::#ident,
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
                            <#field>::read_from_bytes(&bytes, count)
                        }
                    })
                    .collect();

                    match discriminant
                    {
                        Some((_eq,express)) =>
                        {

                            quote!
                            {
                                #express => #name::#ident(#(#fields),*),
                            }
                        },
                        None => { panic!("must define discriminant") }
                    }
                        
                },
                Fields::Named(_fields_named) => 
                {
                    panic!("Yep. You found a Named Field... Packattack doesn't support these at the moment.")//anonymous struct variant
                }
            }
        })
        .collect();

    let blah = quote! {
        impl FromBytes for #name {
            fn read_from_bytes(bytes: &[u8], count : &mut usize) -> #name
            {
                //We want to read the bits 


                match bytes[*count]
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