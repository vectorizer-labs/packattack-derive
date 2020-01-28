use crate::util::{ get_meta_attribute, lit_from_meta_attribute, ident_from_lit, ident_from_str };

pub mod enums;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParentDataType
{
    FromReader,
    FromBytes
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldDataType
{
    FromReader,
    FromBytes(FromBytesType),
    FromBits,
    Payload
}

//TODO: Implement #[payload]

//TODO: pass parent_has_size_hint : bool all the way down to fields.rs

//Enum discriminant type also uses this getter
pub fn get_field_type(attrs : &Vec<syn::Attribute>, parent_data_type : &ParentDataType) -> FieldDataType
{
    //if this field is marked #[from_bytes]
    if let Some(_attr) = get_meta_attribute(attrs, "from_bytes")
    {
        
        let from_bytes_type : FromBytesType = match get_meta_attribute(attrs, "length")
        {
            Some(meta) => FromBytesType::WithLength(lit_from_meta_attribute(meta)),
            None => FromBytesType::SizeInBytes
        };

        return FieldDataType::FromBytes(from_bytes_type);
    }

    //if this field is marked #[from_bits]
    if let Some(_attr) = get_meta_attribute(attrs, "from_bits")
    {
        return FieldDataType::FromBits;
    }

    //if this field is marked #[payload]
    if let Some(_attr) = get_meta_attribute(attrs, "payload")
    {
        return FieldDataType::Payload;
    }

    //If we're from bytes
    //we read fields that are FromBits by default
    if *parent_data_type == ParentDataType::FromBytes { return FieldDataType::FromBits; }

    //Otherwise FromReader is the default
    return FieldDataType::FromReader;
}

//This gets the discriminant for fields in clauses.rs
pub fn get_discriminant(attrs : &Vec<syn::Attribute>) -> syn::Lit
{
    match get_meta_attribute(attrs, "discriminant")
    {
        Some(meta) => lit_from_meta_attribute(meta),
        None => panic!("Packattack enums must define discriminants for all options and for itself!")
    }
}

pub fn get_expose_attribute(attrs : &Vec<syn::Attribute>) -> Option<syn::Ident>
{
    match get_meta_attribute(attrs, "expose")
    {
        Some(meta) => Some(ident_from_lit(lit_from_meta_attribute(meta))),
        None => 
        {
            //if there's a hint then we need to expose this 
            match get_meta_attribute(attrs, "hint")
            {
                Some(meta) => Some(ident_from_str("size_hint")),
                None => None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FromBytesType
{
    WithLength(syn::Lit),
    SizeInBytes
}

pub fn get_hint_reader_literal(attrs : &Vec<syn::Attribute>, parent_data_type : &ParentDataType) -> Option<proc_macro2::TokenStream>
{
    if let Some(meta) = get_meta_attribute(attrs, "hint")
    {
        assert_eq!(parent_data_type, &ParentDataType::FromReader,"Can't use a hint on a type other than from_reader");

        //get the hint type literal
        let hint_type = lit_from_meta_attribute(meta);

        //for now hard coded to FromReader but should easily be changable in the future
        let reader_literal = enums::get_reader_literal(&hint_type, &FieldDataType::FromReader, false);

        return Some(quote!{

            let size_hint : usize = #reader_literal;
            let mut buffer : Vec<u8> = vec![0; size_hint];
            reader.read_exact(&mut buffer).await?;
            let reader : &mut &[u8] =  &mut buffer.as_slice();
        });
    }

    return None;
}