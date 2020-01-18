use syn::{ Fields, Variant, token::Comma, punctuated::Punctuated };
use crate::struct_compiler;
use super::get_enum_match;


pub fn get_clauses(variants: &Punctuated<Variant, Comma>, 
    name : &proc_macro2::Ident, 
    size_in_bits : &syn::Expr) -> Vec<proc_macro2::TokenStream>
{
    variants
    .iter()
    .map(|variant|
    {
        let ident = &variant.ident;
        let discr = match &variant.discriminant
        {
            Some((_eq,express)) => express,
            None => panic!("To derive FromBytes enums must define discriminants for all options!")
        };

        //wrap the clauses
        match &variant.fields {
            Fields::Unit => 
            {
                quote!{ #discr => Ok(#name::#ident), }
            },
            Fields::Unnamed(_) => 
            {
                //get the clauses for this enum variant
                let (clauses, declares) = struct_compiler::get_fields(&variant.fields, quote!{ #size_in_bits });
                quote!{ #discr => Ok(#name::#ident( #(#clauses),* )), }
            },
            Fields::Named(_) => 
            {
                //get the clauses for this enum variant
                let (clauses, declares) = struct_compiler::get_fields(&variant.fields, quote!{ #size_in_bits });
                quote!{ #discr => Ok(#name::#ident{ #(#clauses),* }), }
            }
        }
    })
    .collect()
}