use proc_macro2::TokenStream;
use crate::struct_compiler::bit_index::{ bits_to_byte_address, get_read_clause };

use crate::util::{ ident_from_str, get_meta_attribute, get_option_inner_type, expr_from_meta_attribute };
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
    
    //push a predicate so the first type knows its index starts at predicate_bits
    sizes.push(quote!{ #predicate });

    let mut array_count : usize = 0;

    //collect the field_data_types in a first pass map()
    let data_types : Vec<FieldDataType> = fields.iter().map(|field| { get_field_type(&field.attrs, &parent_data_type) }).collect();

    //if the parent type is from_reader and the first field is not from_reader
    //push the first array on the arrays vec
    if parent_data_type == ParentDataType::FromReader && data_types[0] != FieldDataType::FromReader { push_array_name(&mut arrays, &mut array_count); }

    let mut field_count = 0;

    //build the clauses from the field data
    let clauses : Vec<TokenStream> = fields.iter()
    .map(|field| 
    {
        //get the type we want to read
        //Get the type inside the Option if there is one
        let derivable = match get_option_inner_type(&field.ty)
        {
            Some(ty) => ty,
            None => &field.ty
        };

        //Add up all the previous vars to find this var's preceeding_bits (starting position)
        //The compiler optimizes all these adds away at compile time using constant folding
        let preceeding_bits = quote! {( #(#sizes)+*) };

        //if we're deriving from bits
        let field_data_type : FieldDataType = data_types[field_count].clone();

        let (mut clause, size_in_bits) = get_read_clause(&derivable, &preceeding_bits, field_data_type.clone(), parent_data_type, array_count.clone());

        //if this type is from_reader && (this isn't the last field && the next field isn't from_reader)
        if field_data_type == FieldDataType::FromReader && field_count < data_types.len() - 1  && data_types[field_count + 1] != FieldDataType::FromReader
        {
            //push the declaration for the array we just finished
            //Here we've figured out how big the array needs to be so we close 
            //this segment of vars to be read by pushing the array declaration

            //if there is an array to push, push it
            //there might not be if the first type is from_reader
            if array_count > 0 { push_array_declare(&mut arrays, &mut declares, preceeding_bits); }

            //create a new array with the count + "array_" as the identifier
            let array_num = format!{"array_{}", array_count + 1};

            let array_name : syn::Ident = ident_from_str(array_num.as_str());

            //reset sizes
            sizes.clear();

            //push a NEW name for the NEXT array the NEXT vars will read from
            push_array_name(&mut arrays, &mut array_count);
            
            //fill the last array
            clause  = quote!{ {let result = #clause; reader.read_exact(&mut #array_name).await?; result } }
        };

        field_count += 1;

        //TODO: for some reason I have it where from_reader fields can't be exposed or have an ident

        //push our size to the sizes vec
        sizes.push(quote!{ #size_in_bits });

        //if this field is #[expose = ""] then push the declaration
        clause = match get_expose_attribute(&field.attrs)
        {
            Some(expose_name) => push_exposed_attribute(&mut declares, expose_name, derivable, clause),
            None => clause
        };

        clause = match get_meta_attribute(&field.attrs, "flag")
        {
            Some(meta) => {

                let name_expr = expr_from_meta_attribute(meta);

                match get_option_inner_type(&field.ty)
                {
                    Some(_ty) => {}
                    None => panic!(" flag value 'flag' was set but marked type wasn't an 'Option' type!")
                }

                //We've got an option type so we're going to match 
                //on the expression given in the attribute
                quote!{ 
                    { 
                        match #name_expr
                        {
                            true => Some(#clause),
                            false => None
                        }
                    } 
                }
            },
            None => clause
        };

        //the full clause associated with this var
        //this unwraps to return a derivable
        //if this field has an identifier then tack it on the front of the clause
        match field.ident.clone()
        {
            Some(ident) => quote!{ #ident : #clause },
            None => clause
        }

    }).collect();

    //if the sizes vec isn't empty then we need to push the last array declaration
    //len should be greater than 1 because we always push a { 0 } first
    if sizes.len() > 1 && parent_data_type == ParentDataType::FromReader && array_count > 0
    { 
        let preceeding_bits = quote! {( #(#sizes)+*) };
        push_array_declare(&mut arrays, &mut declares, preceeding_bits);
    };

    let total_size_in_bits = quote! {( #(#sizes)+*) };

    (clauses, data_types, declares, total_size_in_bits)
}

pub fn push_exposed_attribute(declares : &mut Vec<TokenStream>, expose_name : syn::Ident, derivable : &syn::Type, clause : TokenStream) -> TokenStream
{
    declares.push(quote!{ let #expose_name : #derivable; });
    quote!{ {
        let result = #clause;
        #expose_name = result.clone(); 
        result
    } }
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
    //println!("oof : {} should be x > 0 ", arrays.len());

    //get the last name of the array
    let array_name : syn::Ident = arrays[arrays.len()-1].clone();

    //println!("{}", quote!{ #bit_index });

    let byte_size = bits_to_byte_address(&bit_index);

    //push the declaration of the array
    declares.push( quote!{ let mut #array_name : [u8; #byte_size ] = [0; #byte_size ];});
}