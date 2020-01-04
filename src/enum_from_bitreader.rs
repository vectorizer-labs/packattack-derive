extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields, Variant, punctuated, token };

pub fn enum_from_bitreader(variants: &punctuated::Punctuated<Variant, token::Comma>, 
    name : &proc_macro2::Ident, 
    size_in_bits : syn::Lit) -> TokenStream
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
                            <#field>::from_bitreader(reader).await?
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
        #[async_trait::async_trait]
        impl<R> FromBitReader<R> for #name where Self : Sized, R : Read + std::marker::Unpin + std::marker::Send
        {
            async fn from_bitreader(reader : &mut bitreader_async::BitReader<R>) -> Result<#name>
            {

                match usize::from(#reader_literal)
                {
                    #(#clauses)*
                    _ => panic!("uh oh no match")
                }
            }
        }
    };

    blah.into()
}

use syn::{Lit, Ident};

fn get_reader_literal(size_in_bits : syn::Lit) -> proc_macro2::TokenStream
{
    match size_in_bits
    {
        Lit::Int(lit_int) => get_size_reader_literal(lit_int.base10_parse::<usize>().unwrap()),
        Lit::Str(lit_str) => 
        {
            let identifier = Ident::new(lit_str.value().as_str(),lit_str.span());

            quote!{ #identifier::from_bitreader(reader).await? }
        },
        _=> panic!(" Packattack only supports type literals and usizes as size_in_bytes!")
    }
}

fn get_size_reader_literal(size_in_bits : usize) -> proc_macro2::TokenStream
{
    match size_in_bits
    {
        0 => panic!("Tried to build a reader with a size of 0."),
        1..=8 => quote!{ reader.read_be_bits(#size_in_bits).await? },
        //9..=16 => quote!{ reader.read_be_bits::<u16>(#size_in_bits).await?  },
        //17..=32 => quote!{ reader.read_bits::<u32>(#size_in_bits).await? },
        //33..=64 => quote!{ reader.read_bits::<u64>(#size_in_bits).await? },
        //65..=128 => quote!{ reader.read_bits::<u128>(#size_in_bits).await? },
        _ => panic!("Tried to buld a reader with a size of greater than 8 bits! This isn't yet supported.")
    }
}