use proc_macro2::TokenStream;

use crate::util::{ ident_from_str };
use crate::attributes::{ get_field_type, ParentDataType, FieldDataType, get_expose_attribute };

mod derivable;
pub mod from_bits;
pub mod from_bytes;
pub mod from_reader;
pub mod from_slice;

use derivable::{ Derivable, get_derivable};

///Collects the data for the fields in a first attribute reading pass

//TODO: 
//1) Split_point pass
//2) clause pass
//permanent bytes declare at the top of FromReader

/*
//if the parent type is from_reader and the first field is not from_reader
//push the first array on the arrays vec
if parent_data_type == ParentDataType::FromReader && data_types[0] != FieldDataType::FromReader { push_array_name(&mut arrays, &mut array_count); }
}
*/

//(Clauses, declares)
pub fn build_clauses(fields: &syn::Fields, predicate : syn::Expr, parent_data_type : ParentDataType) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream)
{
    let (split_points, preceeding_bits, field_types, derivables, total_size_in_bits, expose_attributes, idents) = get_field_data(fields, predicate, parent_data_type);

    let mut clauses : Vec<TokenStream> = Vec::new();

    let mut declares : Vec<TokenStream> = Vec::new();

    //start at 1 because we're always reading in the next set of bytes
    let mut split_count = 1;

    //add the 0th bytes array
    if(parent_data_type == ParentDataType::FromReader)
    {
        let first_size = &split_points[0];

        declares.push(quote!
        {
            let bytes : [u8; #first_size];
            reader.read_exact(&mut bytes).await?
        });
    }
    

    for i in 0..derivables.len()
    {

        let mut clause : TokenStream = match field_types[i]
        {
            FieldDataType::FromBits => from_bits::get_field(parent_data_type, derivables[i].clone(), &preceeding_bits[i]),
            FieldDataType::FromBytes => from_bytes::get_field(parent_data_type, derivables[i].clone(), &preceeding_bits[i]),
            //FieldDataType::FromSlice => from_slice::get_field(parent_data_type, derivable.clone(), &preceeding_bits),
            FieldDataType::FromReader => {
                
                let result = from_reader::get_field(parent_data_type, derivables[i].clone(), &preceeding_bits[i]);

                match parent_data_type == ParentDataType::FromReader
                {
                    true => 
                    {
                        //TODO: bits to bytes(split_point_size)??
                        let next_size = &split_points[split_count];
                        split_count += 1;

                        quote!
                        {
                            #result
                            bytes = [u8;#next_size];
                            reader.read_exact(&mut bytes).await?;
                            
                        }
                    },
                    false => result
                }
                

            },
            //TODO: Payload
            _ => unimplemented!("Lol gottem!")
        };

        //Option wrapper here
        clause = match &derivables[i]
        {
            //We've got an option type so we're going to match 
            //on the expression given in the attribute
            Derivable::InsideOption(typ, ident) => quote!{ 
                { 
                    match #ident
                    {
                        true => Some(#clause),
                        false => None
                    }
                } 
            },
            //do nothing
            Derivable::Naked(typ) => clause
        };

        //if this field is #[expose = ""] then push the declaration
        clause = match &expose_attributes[i]
        {
            Some(ident) => 
            {
                let raw_type = derivables[i].get_raw();
                declares.push(quote!{
                    let #ident : #raw_type;
                });

                quote!{ {
                    let result : #raw_type = #clause;
                    //TODO: Don't clone
                    #ident = result.clone(); 
                    result
                } }
            },
            None => clause
        };

        //the full clause associated with this var
        //this unwraps to return a derivable
        //if this field has an identifier then tack it on the front of the clause
        clauses.push(match &idents[i]
        {
            Some(ident) => quote!{ #ident : #clause },
            None => clause
        });
    }

    (clauses, declares, total_size_in_bits)
}

fn get_field_data(fields: &syn::Fields, predicate : syn::Expr, parent_data_type : ParentDataType) 
 -> ( Vec<TokenStream>, Vec<TokenStream>, Vec<FieldDataType>, Vec<Derivable>, TokenStream, Vec<Option<syn::Ident>>, Vec<Option<syn::Ident>>)
{
    let mut sizes : Vec<syn::Expr> = Vec::new();

    let mut current_sizes_slice : Vec<syn::Expr> = Vec::new();

    let mut preceeding_sizes : Vec<TokenStream> = Vec::new();//2

    let mut field_types : Vec<FieldDataType> = Vec::new();//3

    let mut derivables : Vec<Derivable> = Vec::new();//4

    let mut split_points : Vec<TokenStream> = Vec::new();//1

    let mut expose_attributes : Vec<Option<syn::Ident>> = Vec::new();//5

    let mut idents : Vec<Option<syn::Ident>> = Vec::new();

    for field in fields.iter()
    {
        let data_type : FieldDataType = get_field_type(&field.attrs, &parent_data_type);

        //collect the field_data_types in a first pass map()
        field_types.push(data_type);

        //collect all the var types
        let derivable = get_derivable(&field);

        derivables.push(derivable.clone());

        let preceeding_bits = quote! { (#(#current_sizes_slice)+*) };

        //if the parent is a reader and this type is a reader then the preceeding bits need to be reset
        //because this is the start of a new buffer
        //TODO: decide if there needs to be more done
        //flag for reading in bytes into the buffer? 
        //or can we tell from this if statement and just read the number of bytes from here until the next from reader or the end
        if(parent_data_type == ParentDataType::FromReader && data_type == FieldDataType::FromReader)
        {
            split_points.push(preceeding_bits.clone());
            current_sizes_slice.clear();
        } 

        preceeding_sizes.push(preceeding_bits);

        //push our size to the sizes vec
        sizes.push(derivable.get_size_in_bits());

        //if this field is #[expose = ""] then push the declaration
        //TODO: move to get data fn below
        let expose_attribute = get_expose_attribute(&field.attrs);
        expose_attributes.push(expose_attribute);

        idents.push(field.ident.clone());
    }

    let total_size_in_bits = quote! {( #(#sizes)+*) };

    (split_points, preceeding_sizes, field_types, derivables, total_size_in_bits, expose_attributes, idents)
}