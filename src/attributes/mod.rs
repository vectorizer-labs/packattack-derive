use crate::util::{ get_meta_attribute, lit_from_meta_attribute, ident_from_lit };

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
    FromBits
}

//Enum discriminant type also uses this getter
pub fn get_field_type(attrs : &Vec<syn::Attribute>, parent_data_type : &ParentDataType) -> FieldDataType
{
    //If we're from bytes
    //we can only read fields that are FromBytes
    //override field type
    if *parent_data_type == ParentDataType::FromBytes { return FieldDataType::FromBits; }

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

    // from bytes is the default
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
        None => None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FromBytesType
{
    WithLength(syn::Lit),
    SizeInBytes
}

/*
pub fn get_size_in_bytes(attrs : &Vec<syn::Attribute>) -> syn::Lit
{
    match get_meta_attribute(attrs, "size_in_bytes")
    {
        Some(meta) => lit_from_meta_attribute(meta),
        None => { panic!("Data Structures deriving FromBytes must define #[size_in_bytes]"); }
    }
}*/