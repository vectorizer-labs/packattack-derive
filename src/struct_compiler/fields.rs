use proc_macro2::TokenStream;
use crate::struct_compiler::bit_index::{ bits_to_byte_address, get_read_clause };

use crate::util::{ident_from_str};
use crate::attributes::{ get_field_type, ParentDataType, FieldDataType, get_expose_attribute };

///returns a vec of fields
pub fn get_fields(fields: &syn::Fields, 
    predicate : TokenStream, 
    parent_data_type : ParentDataType) -> (Vec<TokenStream>,Vec<FieldDataType>, Vec<TokenStream>, TokenStream)
{
    //TODO: turn these into syn:: Types so they have more parser checks
    let mut sizes : Vec<TokenStream> = Vec::new();

    //a list of declarations that will go at the top of the function
    let mut declares : Vec<TokenStream> = Vec::new();

    //a list of array identifiers and their declarations
    let mut arrays : Vec<syn::Ident> = Vec::new();

    let mut data_types : Vec<FieldDataType> = Vec::new();
    
    //push a predicate so the first type knows its index starts at predicate_bits
    sizes.push(quote!{ #predicate });

    let mut array_count : usize = 0;

    //push the first array on the arrays vec
    if parent_data_type == ParentDataType::FromReader { push_array_name(&mut arrays, &mut array_count); }

    let clauses : Vec<TokenStream> = fields.iter()
    .map(|field| 
    {
        //get the type we want to read
        let derivable = field.ty.clone();

        //Add up all the previous vars to find this var's preceeding_bits (starting position)
        //The compiler optimizes all these adds away at compile time using constant folding
        let preceeding_bits = quote! {( #(#sizes)+*) };

        //if we're deriving from bits
        let field_data_type : FieldDataType = get_field_type(&field.attrs, &parent_data_type);

        data_types.push(field_data_type.clone());

        let (clause, size_in_bits) = get_read_clause(&derivable, &preceeding_bits, field_data_type.clone(), parent_data_type, array_count.clone());
        
        if field_data_type == FieldDataType::FromReader 
        {
            //if there were fields before this field
            if sizes.len() > 1 
            {
                //push the declaration for the array we just finished
                //Here we've figured out how big the array needs to be so we close 
                //this segment of vars to be read by pushing the array declaration 

                //TODO: maybe if we're here we need to fill the last array
                push_array_declare(&mut arrays, &mut declares, preceeding_bits);
            }

            //reset sizes
            sizes.clear();

            //push a NEW name for the NEXT array the NEXT vars will read from
            push_array_name(&mut arrays, &mut array_count);

            return quote!{ #clause }
        }

        //push our size to the sizes vec
        sizes.push(quote!{ #size_in_bits });

        //if this field is #[exposed] then push the declaration
        let exposed_clause = match get_expose_attribute(&field.attrs)
        {
            Some(expose_name) =>{
                declares.push(quote!{ let #expose_name : #derivable; });
                quote!{ {
                    let result = #clause;
                    #expose_name = result.clone(); 
                    result
                } }
            },
            None => clause
        };

        //the full clause associated with this var
        //this unwraps to return a derivable
        //if this field has an identifier then tack it on the front of the clause
        match field.ident.clone()
        {
            Some(ident) => quote!{ #ident : #exposed_clause },
            None => exposed_clause
        }

    }).collect();

    //if the sizes vec isn't empty then we need to push the last array declaration
    //len should be greater than 1 because we always push a { 0 } first
    if sizes.len() > 1 && parent_data_type == ParentDataType::FromReader
    { 
        let preceeding_bits = quote! {( #(#sizes)+*) };
        push_array_declare(&mut arrays, &mut declares, preceeding_bits) 
    };

    let total_size_in_bits = quote! {( #(#sizes)+*) };

    (clauses, data_types, declares, total_size_in_bits)
}

pub fn push_array_name(arrays : &mut Vec<syn::Ident>,
    array_count : &mut usize) -> syn::Ident
{
    *array_count += 1;

    //create a new array with the count + "array_" as the identifier
    let array_num = format!{"array_{}", array_count};

    let array_name : syn::Ident = ident_from_str(array_num.as_str());

    //push the name of the array the previous values will be read from
    arrays.push(array_name.clone());

    array_name
}

pub fn push_array_declare(arrays : &mut Vec<syn::Ident>, 
    declares : &mut Vec<TokenStream>,
    bit_index : TokenStream)
{
    //get the last name of the array
    let array_name : syn::Ident = arrays[arrays.len()-1].clone();

    //println!("{}", quote!{ #bit_index });

    let byte_size = bits_to_byte_address(&bit_index);

    //push the declaration of the array
    declares.push( quote!{ let mut #array_name : [u8; #byte_size ] = [0; #byte_size ];});
}