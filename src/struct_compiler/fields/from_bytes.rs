use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType };
use crate::util::{ ident_from_str };
use super::derivable::Derivable;
use crate::struct_compiler::bit_index::{ get_byte_indices, get_slice_indices };

pub fn get_field(parent_data_type : ParentDataType, derivable : Derivable, preceeding_bits : &TokenStream) -> TokenStream
{
    let inner_type = derivable.get_inner_type();
    
    let address = match parent_data_type
    {
        //TODO: fix the array name for FromReader
        ParentDataType::FromReader => get_byte_indices(&inner_type, preceeding_bits, quote!{ reader }),
        ParentDataType::FromSlice => get_slice_indices(&inner_type, preceeding_bits),
        ParentDataType::FromBytes => get_byte_indices(&inner_type, preceeding_bits, quote!{ bytes }),
        _ => panic!("Packattack : FromBytes type can only be inside FromBytes, FromReader, or FromSlice parent types!")
    };
    
    quote!{ <#inner_type>::from_bytes(#address)? }
}