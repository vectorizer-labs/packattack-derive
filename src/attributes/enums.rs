use super::*;
use syn::{ Lit, Ident };
use crate::util::{ lit_from_meta_attribute };

use proc_macro2::TokenStream;

//This gets the attributes of the parent enum
pub fn get_enum_attributes(attrs : &Vec<syn::Attribute>, parent_data_type : &ParentDataType) -> EnumDiscriminant
{
    let discriminant_data_type : FieldDataType = get_field_type(attrs, parent_data_type);

    let discriminant : syn::Lit = match get_meta_attribute(attrs, "size_in_bits")
    {
        //If its not a num then parse it as a Lit
        Some(meta) => lit_from_meta_attribute(meta),
        //otherwise parse the num to a Lit
        None => get_discriminant(attrs)
    };

    let is_str = match get_meta_attribute(attrs, "str")
    {
        Some(_st) => true,
        None => false
    };

    let reader = get_reader_literal(&discriminant, &discriminant_data_type, is_str);

    let size_in_bits = match discriminant_data_type
    {
        FieldDataType::FromReader => quote!{ 0 },
        //otherwise the discriminant is the size_in_bits so we return it as the predicate
        _ => quote!{ #discriminant }
    };

    return EnumDiscriminant 
    { 
        size_in_bits : size_in_bits, 
        data_type : discriminant_data_type,
        reader_literal : reader
    };
}

pub struct EnumDiscriminant
{
    pub size_in_bits : TokenStream,
    pub data_type : FieldDataType,
    pub reader_literal : TokenStream
}

fn get_reader_literal(discriminant : &syn::Lit, discriminant_data_type : &FieldDataType, is_str : bool) -> TokenStream
{
    match discriminant
    {
        Lit::Int(lit_int) => 
        {
            //create the bitmask for this byte
            let bitmask : TokenStream = quote!{ ((1 << #lit_int) - 1) << (8 - #lit_int) };

            let byte_token = match discriminant_data_type
            {
                FieldDataType::FromReader => panic!("Packattack Internal Error: No byte token when reading from_reader!"),
                _ => quote!{ bytes[0] }
            };

            // read the byte, mask it for the bits we want, 
            //and bit shift them back to the beginning of the u8
            //finally pass that value into from_u8
            quote!{ (#byte_token & #bitmask) >> (8 - #lit_int) }
        }
        Lit::Str(lit_str) => 
        {
            let identifier = Ident::new(lit_str.value().as_str(),lit_str.span());

            if is_str { return quote!{ &*<String>::from_reader(reader).await? }; }

            quote!{ usize::from(<#identifier>::from_reader(reader).await?) }
        },
        _=> panic!(" Packattack only supports type literals and usizes as size_in_bytes!")
    }
}