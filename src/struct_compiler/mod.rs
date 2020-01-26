use syn::Fields;

pub mod fields;
pub mod bit_index;

pub use fields::get_fields;

use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType, FieldDataType };

//Compiles the clauses into a struct
pub fn get_struct(fields : &Fields, name : &proc_macro2::Ident, parent_data_type : ParentDataType) -> (TokenStream,TokenStream)
{
    //no predicate because because these just return structs therefor predicate = 0
    let (clauses, data_types, declares, total_size_in_bits) = get_fields(fields, quote!{ 0 }, parent_data_type);

    let first_field_data_type = data_types[0].clone();

    let fill_first_array = match parent_data_type == ParentDataType::FromReader && 
                                 first_field_data_type != FieldDataType::FromReader
    {
        true => quote!{ reader.read_exact(&mut array_1).await?; },
        false => quote!{ }
    };
    

    //this literally just wraps the fields in a either a {} or () 
    //depending on if the fields are named or unnamed
    let wrapped_fields : TokenStream = match fields
    {
        Fields::Named(_fields_named) => quote!
        {
            #(#declares)*

            #fill_first_array

            Ok(#name{ #(#clauses),* }) 
        },
        Fields::Unnamed(_fields_unnamed) => quote!
        { 
            #(#declares)*

            #fill_first_array

            Ok(#name( #(#clauses),* )) 
        },
        _ => panic!("Packattack only supports reading from structs with named or unamed fields")
    };

    (wrapped_fields, total_size_in_bits)
}