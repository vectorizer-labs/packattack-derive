use proc_macro2::TokenStream;
use super::{ get_array_address, bits_to_byte_address };

use crate::util::{get_meta_attribute, ident_from_string};

///returns a vec of fields
pub fn get_fields(fields: &syn::Fields, predicate : TokenStream) -> (Vec<TokenStream>,Vec<TokenStream>)
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

    //push the first array on the arrays vec
    push_array_name(&mut arrays, &mut declares, &mut array_count);

    let clauses = fields.iter()
    .map(|field| 
    {
        //get the type we want to read
        let derivable = field.ty.clone();

        //Add up all the previous vars to find this var's preceeding_bits (starting position)
        //The compiler optimizes all these adds away at compile time using constant folding
        let preceeding_bits = quote! {( #(#sizes)+*) };
        
        //Find out if we're reading from a reader or from bytes already read
        let clause = match get_meta_attribute(&field.attrs, "from_reader")
        {
            //if this variable is from a reader then reset the size vec
            //and add a new array
            Some(_attr) =>
            { 
                //reset sizes
                sizes.clear();

                //push a zero to the sizes vec so the following vars know where to start
                sizes.push(quote! { 0 });

                //push the declaration for the array we just finished
                //Here we've figured out how big the array needs to be so we close 
                //this segment of vars to be read by pushing the array declaration 
                push_array_declare(&mut arrays, &mut declares, preceeding_bits);

                //push a NEW name for the NEXT array the NEXT vars will read from
                push_array_name(&mut arrays, &mut declares, &mut array_count);

                let array_name : syn::Ident = arrays[arrays.len()-1].clone();

                //TODO: make sure to properly wrap the read_exact
                quote!{ 
                    {
                        let result = <#derivable>::from_reader(reader).await?;
                        //fill the new array (before we figure out how big it is)
                        reader.read_exact(&mut #array_name).await?;
                        result
                    }
                } 
            },
            //otherwise its just a normal from_bytes type so we find the preceeding bits and
            //read from there using the array address and from_bytes
            None => 
            {
                //TODO: push the array declarations to the top
                let current_array_name = arrays[arrays.len()-1].clone();

                //get the array address, which is the bitmask plus the array indices 
                //from which we are reading 
                let array_address = get_array_address(&derivable, &preceeding_bits, &current_array_name);

                //push our size to the sizes vec
                sizes.push(quote!{ <#derivable>::SIZE_IN_BITS });

                quote!{ <#derivable>::from_bytes(#array_address)? }
            }
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
    if !sizes.len() > 1 
    { 
        let preceeding_bits = quote! {( #(#sizes)+*) };
        push_array_declare(&mut arrays, &mut declares, preceeding_bits) 
    };

    (clauses, declares)
}

//TODO: push the name at the beginning of the cycle
//push the declaration at the close of the cycle

pub fn push_array_name(arrays : &mut Vec<syn::Ident>, 
    declares : &mut Vec<TokenStream>,
    array_count : &mut usize) -> syn::Ident
{
    *array_count += 1;

    //create a new array with the count + "array_" as the identifier
    let array_num = format!{"array_{}", array_count};

    let array_name : syn::Ident = ident_from_string(array_num);

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

    let byte_size = bits_to_byte_address(&bit_index);

    //push the declaration of the array
    declares.push( quote!{ let #array_name = [u8; #byte_size ] = [0; #byte_size ]; } );
}