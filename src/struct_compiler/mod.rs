use syn::Fields;

pub mod fields;
pub mod bit_index;

pub use fields::build_clauses;

use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType };

//Compiles the clauses into a struct
pub fn get_struct(fields : &Fields, name : &proc_macro2::Ident, parent_data_type : ParentDataType) -> (TokenStream, Vec<TokenStream>, TokenStream)
{
    //no predicate because there's no enum discrimnant to read
    let no_predicate : syn::Expr = syn::parse(quote!{ 0 }.into()).unwrap();

    let (clauses, declares, total_size_in_bits) = build_clauses(fields, no_predicate, parent_data_type);

    //this literally just wraps the fields in a either a {} or () 
    //depending on if the fields are named or unnamed
    let wrapped_fields : TokenStream = match fields
    {
        Fields::Named(_fields_named) => quote!{ Ok(#name{ #(#clauses),* }) },
        Fields::Unnamed(_fields_unnamed) => quote!{ Ok(#name( #(#clauses),* )) },
        _ => panic!("Packattack only supports reading from structs with named or unamed fields")
    };

    (wrapped_fields, declares, total_size_in_bits)
}