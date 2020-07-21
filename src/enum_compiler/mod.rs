use syn::{ Variant, token::Comma, punctuated::Punctuated };
use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType, enums::EnumDiscriminant };

mod clauses;


pub fn get_enum(variants: &Punctuated<Variant, Comma>, 
                name : &proc_macro2::Ident, 
                attributes : &EnumDiscriminant,
                parent_data_type : ParentDataType)  -> TokenStream
{
    let (clauses, declares) = clauses::get_clauses(variants, name, attributes, parent_data_type);
    
    let reader_literal = &attributes.reader_literal;

    let name_string = name.to_string();

    let fin_enum = quote!
    {
        #(#declares)*

        match #reader_literal
        {
            #(#clauses)*
            _ => Err(packattack::error::PackattackParserError::NoEnumMatch(#name_string.to_owned()))?
        }
    };

    fin_enum
}