extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields, GenericArgument, PathArguments, Type};
use super::util::{get_attribute_value, has_meta_path, get_attribute_path};

//TODO: Break each match out into its own functions

pub fn struct_from_bitreader(fields : &Fields, name : &proc_macro2::Ident) -> TokenStream
{
    match fields 
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
                    #identifier : <#derivable>::from_bitreader(reader).await?
                };

                clause
            }).collect();

            let blah = quote!{
        
                #[async_trait::async_trait]
                impl<R> FromBitReader<R> for #name where Self : Sized, R : Read + std::marker::Unpin + std::marker::Send
                {
                    #[allow(trivial_numeric_casts)]
                    async fn from_bitreader(reader : &mut bitreader_async::BitReader<R>) -> Result<#name>
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
        },
        Fields::Unnamed(fields_unnamed) => 
        {
            //these are the fields marked with the attribute #[expose]
            let exposed_fields : Vec<_> = fields_unnamed.unnamed
            .iter().filter(|field| has_meta_path(&field.attrs, "expose")).map(|field|
            {
                let name = get_attribute_value(&field.attrs, "expose").unwrap();

                let name_lit = get_ident(name);
                let field_lit = &field.ty;

                quote!{ let #name_lit : #field_lit; }

            }).collect();

            //Generate clauses for all fields 
            let clauses : Vec<proc_macro2::TokenStream> = fields_unnamed.unnamed
            .iter()
            .map(|field| 
            {
                let derivable = &field.ty;

                //TODO: Make this a reference instead of a clone into a local variable
                //if this field has the expose meta attribute then we assign to the global variable 
                if has_meta_path(&field.attrs, "expose")
                {
                    let name = get_attribute_value(&field.attrs, "expose").unwrap();

                    let name_lit = get_ident(name);

                    //otherwise we're still inside the byte so just read from the byte
                    let clause = quote! 
                    {
                        { #name_lit = <#derivable>::from_bitreader(reader).await?; #name_lit.clone() } 
                    };

                    clause
                }
                else if has_meta_path(&field.attrs, "flag")
                {
                    let name_lit = get_attribute_path(&field.attrs, "flag").unwrap();

                    let name_ident = ident_from_lit(name_lit);

                    //println!("path : {:#?}", name_ident);


                    //Get the type inside the Option
                    let inner_type = match derivable
                    {
                        Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => {
                            // Get the first segment of the path (there is only one, in fact: "Option"):
                            let type_params = &typepath.path.segments.iter().nth(0).unwrap().arguments;
                            // It should have only on angle-bracketed param ("<String>"):
                            let generic_arg = match type_params {
                                PathArguments::AngleBracketed(params) => params.args.iter().nth(0).unwrap(),
                                _ => panic!("Packattack: No Type or Bad Type Found Inside Option!"),
                            };

                            // This argument must be a type:
                            match generic_arg {
                                GenericArgument::Type(ty) => ty,
                                _ => panic!("Packattack: Type Found Inside Option Doesn't Parse!"),
                            }
                        }
                        _ => panic!(" flag value 'flag' was set but marked type wasn't an 'Option' type!"),
                    };

                    //otherwise we're still inside the byte so just read from the byte
                    let clause = quote! 
                    {
                        { 
                            match #name_ident
                            {
                                true => Some(<#inner_type>::from_bitreader(reader).await?),
                                false => None
                            } 
                        } 
                    };

                    clause
                }
                else
                {
                    //otherwise we're still inside the byte so just read from the byte
                    let clause = quote! 
                    {
                        <#derivable>::from_bitreader(reader).await?
                    };

                    clause
                }

                
            }).collect();

            let blah = quote!{
        
                #[async_trait::async_trait]
                impl<R> FromBitReader<R> for #name where Self : Sized, R : Read + std::marker::Unpin + std::marker::Send
                {
                    async fn from_bitreader(reader : &mut bitreader_async::BitReader<R>) -> Result<#name>
                    {

                        #(#exposed_fields)*

                        Ok(#name
                        (
                            #(#clauses),*
                        ))
                    }
                }
            };

            blah.into()
                
        },
        _ => panic!("Packattack only supports reading from structs with named fields")

    }
    
}

//Taken from 
//https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn
fn path_is_option(path: &syn::Path) -> bool 
{
    path.leading_colon.is_none()
    && path.segments.len() == 1
    && path.segments.iter().next().unwrap().ident == "Option"
}

fn get_ident( name : syn::Lit) -> proc_macro2::TokenStream
{
    match name
    {
        syn::Lit::Str(lit_str) => 
        {
            let identifier = syn::Ident::new(lit_str.value().as_str(),lit_str.span());
            quote!{ #identifier }
        },
        _=> panic!(" Packattack only supports string literals as expose name!")
    }
}

//convert a syn::Lit to a syn::Identifier stream split by "."
//bad hack but for some reason syn::parse() doesn't work yet
fn ident_from_lit( name : syn::Lit) -> proc_macro2::TokenStream
{
    match name
    {
        syn::Lit::Str(lit_str) => 
        {
            let span_lit = lit_str.span();
            //println!("span : {:#?}", span_lit);

            let ident_value = lit_str.value();

            let ident_vec : Vec<syn::Ident> = ident_value.as_str()
            .split(".")
            .map(|string|
            {
                //clone the span and prepare to edit it
                let local_span = span_lit.clone();
                syn::Ident::new(string, local_span)

            }).collect();

            //println!("ident_string : {:#?}", ident_vec);
            quote!{ #(#ident_vec).* }
        },
        _=> panic!(" Packattack only supports string literals as expose name!")
    }
}