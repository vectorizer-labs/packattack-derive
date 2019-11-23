extern crate proc_macro;

extern crate proc_macro2;

extern crate syn;

use proc_macro::TokenStream;

use syn::{Fields};


pub fn struct_from_bytes(fields : &Fields, name : &proc_macro2::Ident) -> TokenStream
{
    let clauses = match fields 
    {
        Fields::Named(fields_named) => 
        {
            let mut clauses : Vec<proc_macro2::TokenStream> = Vec::new();

            let mut field_iter = fields_named.named.iter();

            loop
            {
                //while we have a next field
                let field = match field_iter.next()
                {
                    Some(field) => field,
                    None => break
                };
                
                match field.attrs.len()
                {
                    //no attributes which means the data type is made of bytes so we return a read bytes
                    0 => 
                    {
                        process_multi_byte(&mut clauses, &field)
                    },
                    //attributes which mean we've got at least one bitmapped type
                    _ => 
                    {
                        //take control over the iterator
                        //as we go over the fields inside the next byte
                        //then return control to the parent loop
                        process_byte(&mut clauses, &mut field_iter, &field)
                    }
                }
            }

            clauses
        },
        _ => panic!("Packattack only supports reading from structs with named fields")
    };

    let blah = quote! {
        impl FromBytes for #name {
            #[allow(trivial_numeric_casts)]
            fn read_from_bytes(bytes: &[u8], count : &mut usize) -> #name
            {
                #name
                {
                    #(#clauses),*
                }
            }
        }
    };

    //println!("{}", blah);

    blah.into()
}

fn get_size_expression_from_type(field: &syn::Type) -> proc_macro2::TokenStream
{
    quote!
    {
        <#field>::size_in_bits
    }
}

fn process_byte(clause_vec : &mut Vec<proc_macro2::TokenStream>, 
                field_iter : &mut syn::punctuated::Iter<'_, syn::Field>,
                first_bit_field : &syn::Field )
{
    //the size of all the preceding types
    let mut preceding_type_sizes: Vec<proc_macro2::TokenStream> = Vec::new();

    //push a zero so the first bitmask type in the byte has a reference to start at 
    preceding_type_sizes.push(quote!{0});

    //push the bit type we've got right now from the iterator

    clause_vec.push(build_non_terminating_bit_type_clause(&mut preceding_type_sizes, first_bit_field));

    loop
    {
        //while we have a next field
        let field = match field_iter.next()
        {
            Some(field) => field,
            None => panic!("Loop terminated without finding an #[end_byte] attribute. Perhaps you forgot to add it?")
        };

        //if this item has and #[end_byte] attribute then 
        //we need to increment the byte index and return to the parent iterator
        if is_end_byte(&field)
        {
            clause_vec.push(build_terminating_bit_type_clause(&preceding_type_sizes, field));
            //we found the last bit type so return
            return;   
        }

        //otherwise we're still inside the byte so just read from the byte
        clause_vec.push(build_non_terminating_bit_type_clause(&mut preceding_type_sizes, field));
    }
}


//build the token stream for a normal bitfield
fn build_non_terminating_bit_type_clause(preceding : &mut Vec<proc_macro2::TokenStream>, field : &syn::Field) -> proc_macro2::TokenStream
{
    //get the name of the field
    let identifier = match &field.ident
    {
        Some(n) => n,
        None => panic!("Found named field without name.")
    };

    let derivable = &field.ty;

    //otherwise we're still inside the byte so just read from the byte
    let clause = quote! 
    {
        #identifier : <#derivable>::read_from_byte(&bytes[*count], #(#preceding)+*)
    };

    //add our size to the size_vec so we can be a reference for the next bitmask
    //by adding these fixed sizes together and then pushing a bitmask according to size
    //we can take advantage of constant folding https://en.wikipedia.org/wiki/Constant_folding
    //at compile time
    preceding.push(get_size_expression_from_type(&field.ty));

    clause
}

//build the token stream for the last bitfield in the sequence
fn build_terminating_bit_type_clause(preceding : &Vec<proc_macro2::TokenStream>, field : &syn::Field) -> proc_macro2::TokenStream
{
    //get the name of the field
    let identifier = match &field.ident
    {
        Some(n) => n,
        None => panic!("Found named field without name.")
    };

    let derivable = &field.ty;

    quote! 
    {
        #identifier : {
            *count += 1; 
            <#derivable>::read_from_byte(&bytes[*count - 1], 
            //add up all the sizes of the types before this type
            //that determines which index to push the bitmask to
            #(#preceding)+* ) 
        }
    }
}

//TODO: Implement this in case someone wants to use more than one attribute
//on a type inside a struct
/*
fn is_start_byte() -> bool
{
    true
}*/

fn is_end_byte(field : &syn::Field) -> bool
{
    match field.attrs.len()
    {
        //no attributes which means the data type is made of bytes so we return a read bytes
        0 => false,
        //attributes which mean we've got at least one bitmapped type
        _ => true
    }
}

//generate a clause that calls read bytes the default way
fn process_multi_byte(clause_vec : &mut Vec<proc_macro2::TokenStream>, field : &syn::Field )
{
    let identifier = match &field.ident
    {
        Some(n) => n,
        None => panic!("Found named field without name.")
    };

    let derivable = &field.ty;

    clause_vec.push(quote! 
    {
        #identifier : <#derivable>::read_from_bytes(bytes, count)
    });
}