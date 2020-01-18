use syn::{Fields};

pub mod fields;
pub mod bit_index;

pub use fields::get_fields;

use proc_macro2::TokenStream;

//reexport useful addressing functions
pub use bit_index::
{
    get_bit_address,
    get_array_address,
    bits_to_byte_address
};

//Compiles the clauses into a struct
pub fn get_struct(fields : &Fields, name : &proc_macro2::Ident, clauses : Vec<TokenStream>) 
-> (TokenStream, Vec<TokenStream>)
{
    //no predicate because because these just return structs therefor predicate = 0
    let (clauses, declares) = get_fields(fields, quote!{ 0 });

    //this literally just wraps the fields in a either a {} or () 
    //depending on if the fields are named or unnamed
    let wrapped_fields : TokenStream = match fields
    {
        Fields::Named(_fields_named) => quote!{ Ok(#name{ #(#clauses),* }) },
        Fields::Unnamed(_fields_unnamed) => quote!{ Ok(#name( #(#clauses),* )) },
        _ => panic!("Packattack only supports reading from structs with named or unamed fields")
    };

    (wrapped_fields, declares)
}