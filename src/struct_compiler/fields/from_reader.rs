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
        ParentDataType::FromReader => quote!{ reader },
        _ => panic!("Packattack : FromReader type can only be inside FromReader parent types!")
    };
    
    quote!{ <#inner_type>::from_reader(#address)? }
}