#![feature(arbitrary_enum_discriminant)]
#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;

extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Data, Fields };

#[proc_macro_derive(FromBytes)]
pub fn from_bytes(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let variants = match ast.data {
        Data::Enum(ref data_enum) => &data_enum.variants,
        _ => panic!(
            "`FromPrimitive` can be applied only to enums, {} is neither",
            name
        ),
    };

    let clauses: Vec<_> = variants
        .iter()
        .map(|variant| {
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
                                #express => Some(#name::#ident),
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
                            <#field>::read_from_bytes(&bytes)
                        }
                    })
                    .collect();

                    match discriminant
                    {
                        Some((_eq,express)) =>
                        {

                            quote!
                            {
                                #express => Some(#name::#ident(#(#fields),*)),
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
            #[allow(trivial_numeric_casts)]
            fn read_from_bytes(bytes: &[u8]) -> Option<&Self>
            {
                match &bytes[0]
                {
                    #(#clauses)*
                    _ => None
                }
            }
        }
    };

    println!("{}", blah);

    blah.into()
}