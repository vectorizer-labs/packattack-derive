use proc_macro2::TokenStream;
use crate::attributes::{ ParentDataType, FieldDataType, FromBytesType };
use crate::util::{ ident_from_str };

pub fn bits_to_byte_address(leading_bits : &TokenStream) -> TokenStream
{
    quote!{ (#leading_bits / 8) }
}

pub fn bits_to_byte(leading_bits : &TokenStream) -> TokenStream
{
    quote!{ ((#leading_bits + 7) / 8) }
}

pub fn get_bitmask(size_in_bits : &TokenStream, bits_consumed_inside_byte : &TokenStream) -> TokenStream
{
    //create the bitmask for this byte // shift the mask by the number of bits already read in this byte
    quote!{ ((1u8 << (#size_in_bits % 8)) - 1u8) << ((8 - (#size_in_bits + #bits_consumed_inside_byte) ) as u8)  }
}


//this gets byte index of a var inside an array 
//i.e. [1..2]
//TODO: for some reason reading a u16 is one index off
pub fn get_byte_indices(derivable : &syn::Type, total_bits_consumed : &TokenStream, byte_token : TokenStream) -> (TokenStream, TokenStream)
{
    //the byte where the number of bits already read lands
    let array_start = bits_to_byte_address(total_bits_consumed);

    let size_in_bits = quote!{ (<#derivable>::SIZE_IN_BITS) };

    //the byte where the number of bits already read + the size of this var lands
    let array_end = bits_to_byte_address(&(quote!{ (#total_bits_consumed + #size_in_bits) }));

    //TODO: If for some reason I decide to have different array names 
    //add the option back to this function
    (quote!{ & #byte_token[#array_start .. #array_end] }, size_in_bits)
}

//This finds the byte that a var is inside
//indexes it inside the current array using that byte
pub fn get_bit_indices_from_array(derivable : &syn::Type, 
    total_bits_consumed : &TokenStream, 
    array_name : syn::Ident) -> (TokenStream, TokenStream)
{
    //find the byte I'm in
    let byte = bits_to_byte_address(total_bits_consumed);

    get_bit_indices(derivable, total_bits_consumed, quote!{ #array_name[#byte] })
}

//This returns the bits masked from the #byte_token byte and shifted back to the front of the byte
//ready for big endian reading
pub fn get_bit_indices(derivable : &syn::Type, 
    total_bits_consumed : &TokenStream, 
    byte_token : TokenStream) -> (TokenStream, TokenStream)
{
    //this is the number of 
    let bits_consumed_inside_byte = quote!{ (#total_bits_consumed % 8) };

    let size_in_bits = quote!{ (<#derivable>::SIZE_IN_BITS) };

    //byte bitmask
    let bitmask = get_bitmask(&size_in_bits, &bits_consumed_inside_byte);

    // read the byte, mask it for the bits we want, 
    //and bit shift them back to the beginning of the u8
    //finally pass that value into from_u8
    (quote!{ (#byte_token & #bitmask) >> (8 - (#size_in_bits + #bits_consumed_inside_byte)) }, size_in_bits)
}


pub fn get_read_clause(derivable : &syn::Type, preceeding_bits : &TokenStream, field_data_type : FieldDataType,
    parent_data_type : ParentDataType, array_count : usize) -> (TokenStream, TokenStream)
{
    match parent_data_type
    {
        //were in from reader which means we have `reader` and `array_1`, `array_2`... and so on
        ParentDataType::FromReader => handle_from_reader_parent(derivable, preceeding_bits, field_data_type, array_count),
        //were in from_bytes which means we only have the `bytes` array to read from
        ParentDataType::FromBytes => 
        {
            match field_data_type
            {
                FieldDataType::FromReader => unimplemented!("Can't read a from_reader type inside a from_bytes type!"),
                FieldDataType::FromBytes(from_bytes_type) => 
                {
                    match from_bytes_type
                    {
                        FromBytesType::WithLength(len) => unimplemented!("TODO: Implement copy slice into BUFFER starting from preceeding_bits"),
                        FromBytesType::SizeInBytes => {
                            let (address, size) = get_byte_indices(derivable, preceeding_bits, quote!{ bytes });
                            (quote!{ <#derivable>::from_bytes(#address)? }, size)
                        }
                    }
                },
                FieldDataType::FromBits => 
                {
                    let (address, size) = get_bit_indices_from_array(derivable, preceeding_bits, ident_from_str("bytes"));
                    (quote!{ <#derivable>::from_bytes((#address).to_be_bytes())? }, size)
                },
                FieldDataType::Payload => unimplemented!("TODO : let the payload take the remaining bytes")
            }
        }
    }
}

fn handle_from_reader_parent(derivable : &syn::Type, preceeding_bits : &TokenStream, field_data_type : FieldDataType, array_count : usize) -> (TokenStream, TokenStream)
{
    //Now we know we're in from_reader, what kind of field are we reading?
    match field_data_type
    {
        FieldDataType::FromReader => 
        {
            //from_reader is variable only
            (quote!{ <#derivable>::from_reader(reader).await? }, quote!{ 0 })
        },
        FieldDataType::FromBytes(from_bytes_type) =>
        {
            //from_bytes can be #[length], or SIZE_IN_BYTES large
            match from_bytes_type
            {
                //there's a length so find the slice 
                FromBytesType::WithLength(len) => unimplemented!(),
                /*{
                    
                    (quote!{{let buffer = vec![0; #len];
                            reader.read_exact(buffer.as_mut_slice()).await?;
                            <#derivable>::from_reader(buffer.as_slice()).await?
                        }},
                        //TODO: fix this length to assign to a local variable outside this scope
                        quote!{ (#len * 8) })
                },*/
                //there's no length so there's a fixed size_in_bytes
                //NOT BROKEN
                FromBytesType::SizeInBytes => 
                {
                    //create a new array with the count + "array_" as the identifier
                    let array_num = format!{"array_{}", array_count};

                    let array_name : syn::Ident = ident_from_str(array_num.as_str());

                    let (address, size) = get_byte_indices(derivable, preceeding_bits, quote!{ #array_name });
                    (quote!{ <#derivable>::from_bytes((#address).try_into()?)? }, size)
                }
            }
        },
        FieldDataType::FromBits => 
        {
            //create a new array with the count + "array_" as the identifier
            let array_num = format!{"array_{}", array_count};

            let array_name : syn::Ident = ident_from_str(array_num.as_str());
            let (address, size) = get_bit_indices_from_array(&derivable, &preceeding_bits, array_name);

            (quote!{ <#derivable>::from_bytes((#address).to_be_bytes())? }, size)

        },
        FieldDataType::Payload => 
        {
            //pass in the reader here because if we're a payload in from reader
            //then size_hint has already been defined which means reader is &mut &[u8]
            ( quote!{ <#derivable>::from_bytes(reader)? }, quote!{ 0 })
        }
    }
}