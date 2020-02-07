use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType };
use crate::util::{ ident_from_str };
use super::derivable::Derivable;
use crate::struct_compiler::bit_index::get_bit_indices_from_array;

pub fn get_field(parent_data_type : ParentDataType, derivable : Derivable, preceeding_bits : &TokenStream) -> TokenStream
{
    let inner_type = derivable.get_inner_type();

    //TODO: scale this to [u8; ?] to allow FromBits types withS > 8 bits
    let address = match parent_data_type
    {
        ParentDataType::FromBytes => get_bit_indices_from_array(&inner_type, preceeding_bits, ident_from_str("bytes")),
        ParentDataType::FromReader => unimplemented!("oof"),
        _ => panic!("Packattack : FromBits type can only be inside FromBytes or FromReader parent type!")
    };
    
    quote!{ <#inner_type>::from_bits(#address)? }
}