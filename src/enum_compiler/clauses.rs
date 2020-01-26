use syn::{ Fields, Variant, token::Comma, punctuated::Punctuated };

use crate::struct_compiler;
use crate::attributes::{ ParentDataType, FieldDataType, get_discriminant, enums::EnumDiscriminant };
use proc_macro2::TokenStream;

pub fn get_clauses(variants: &Punctuated<Variant, Comma>, 
    name : &proc_macro2::Ident, 
    attributes : &EnumDiscriminant,
    parent_data_type : ParentDataType) -> (Vec<TokenStream>, Vec<TokenStream>)
{
    let mut global_declares : Vec<TokenStream> = Vec::new();

    if parent_data_type == ParentDataType::FromReader
    {
        match attributes.data_type.clone()
        {
            //Do nothing
            FieldDataType::FromReader => {},
            _ =>
            {
                //TODO: size this array appropriately for the size of a from_bytes type
                global_declares.push(quote!
                {
                    let mut bytes : [u8; 1] = [0; 1];
                    reader.read_exact(&mut bytes).await?;
                });
            }
            
        }
    }

    let fin_vars : Vec<TokenStream> = variants
    .iter()
    .map(|variant|
    {
        let ident = &variant.ident;
        let discr : syn::Lit = match &variant.discriminant
        {
            Some((_eq,express)) => 
            {
                match express.clone()
                {
                    syn::Expr::Lit(expr_lit) => expr_lit.lit,
                    _ => panic!("Couldn't parse enum discriminant as syn::Lit")
                }
            },
            None => get_discriminant(&variant.attrs)
        };

        //if this is a unit field just return
        match variant.fields
        {
            Fields::Unit => { return quote!{ #discr => Ok(#name::#ident), }; },
            _ => {}
        }

        //We should only copy into the next byte if the discriminant itself is NOT from_reader
        let size_in_bits = &attributes.size_in_bits;

        //get the clauses for this enum variant
        let (clauses, data_types, declares, _total_size_in_bits) = struct_compiler::get_fields(&variant.fields, quote!{ #size_in_bits }, parent_data_type);

        //wrap the clauses
        let fin_clause = match &variant.fields {
            Fields::Unnamed(_) => quote!{ Ok(#name::#ident( #(#clauses),* )) },
            Fields::Named(_) => quote!{ Ok(#name::#ident{ #(#clauses),* }) },
            Fields::Unit => unreachable!()
        };

        let first_field_data_type : FieldDataType = data_types[0].clone();

        let first_byte_assignment = get_first_byte_assignment(&first_field_data_type, &attributes.data_type, &parent_data_type);

        quote!{ 
            #discr => 
            {
                #(#declares)*

                #first_byte_assignment

                #fin_clause
            },
        }
    })
    .collect();

    (fin_vars, global_declares)
}


//This whole business is a big mess due to the nature of allowing the enum discriminant 
//to be an arbitrary DataType

//In most cases we'll do nothing
//but If we have enum discriminant that derives from_bits or from_bytes and the parent DataType
//is from_reader then we have to copy the bytes that have already been read into the first array 
//Thats basically all this is doing but the matches are here for clarity
fn get_first_byte_assignment(first_field_data_type : &FieldDataType, 
    enum_discriminant_data_type : &FieldDataType,
    parent_data_type : &ParentDataType) -> TokenStream
{
    match parent_data_type
    {
        ParentDataType::FromReader =>
        {
            match enum_discriminant_data_type
            {
                FieldDataType::FromReader => 
                {
                    //Enum discriminant is from_reader
                    //in this case there's no data thats already been read but the first array needs to be filled
                    match first_field_data_type
                    {
                        //do nothing because there is no first array
                        FieldDataType::FromReader => quote!{},
                        //Otherwise we have a from_bits or from_bytes type that needs the first array to be filled
                        _ => quote! { reader.read_exact(&mut array_1).await?; }
                    }
                },
                //if the enum discriminant is from_bytes or from_bits then we're definitely making a first array
                _ => 
                {
                    
                    //In these cases the enum was from_bits or from_bytes so data needs to be copied to 
                    //the first array from the discriminant data thats already been read
                    match first_field_data_type
                    {
                        //If the first field is from reader then there's no first array
                        FieldDataType::FromReader => quote!{},
                        //TODO: match on the from_bytes_type?
                        _ => quote!{
                            //copy the byte into the first array
                            array_1[0] = bytes[0]; 
                            //fill the rest of the first array
                            let end_len = array_1.len();
                            //TODO: change this 1 to bytes.len()
                            reader.read_exact(&mut array_1[1..end_len]).await?;
                        }
                    }
                }
            }
        },
        //if the parent type isn't from reader then we're not creating arrays anyway
        //so do nothing
        ParentDataType::FromBytes => quote!{}
    }
}