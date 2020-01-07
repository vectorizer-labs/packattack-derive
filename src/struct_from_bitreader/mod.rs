extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields};


mod named_fields;
mod unnamed_fields;


//TODO: Break each match out into its own functions
pub fn struct_from_bitreader(fields : &Fields, name : &proc_macro2::Ident) -> TokenStream
{
    match fields 
    {
        Fields::Named(fields_named) => 
        {
            named_fields::get_named_fields(fields_named, name)
        },
        Fields::Unnamed(fields_unnamed) => 
        {
            
            unnamed_fields::get_unamed_fields(fields_unnamed, name)
        },
        _ => panic!("Packattack only supports reading from structs with named fields")

    }
    
}



