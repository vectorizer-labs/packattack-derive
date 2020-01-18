use proc_macro2::TokenStream;

pub fn bits_to_byte_address(leading_bits : &TokenStream) -> TokenStream
{
    quote!{ ((8 - ((#leading_bits) & 7)) & 7) }
}

pub fn get_bitmask(size_in_bits : &TokenStream, bits_consumed_inside_byte : &TokenStream) -> TokenStream
{
    //create the bitmask for this byte
    quote!{ ((1 << #size_in_bits) - 1) << (8 - (#size_in_bits + #bits_consumed_inside_byte)) }
}

pub fn get_array_indices(array_name : &syn::Ident, 
    total_bits_consumed : &TokenStream,
    size_in_bits : &TokenStream) -> TokenStream
{
    //the byte where the number of bits already read lands
    let array_start = bits_to_byte_address(total_bits_consumed);

    //the byte where the number of bits already read + the size of this var lands
    let array_end = bits_to_byte_address(quote!{ (#total_bits_consumed + #size_in_bits) });

    quote!{ #array_name[#array_start .. #array_end]  }
}

pub fn get_array_address(derivable : &syn::Type, 
    total_bits_consumed : &TokenStream, 
    array_name : &syn::Ident) -> TokenStream
{
    //find the byte I'm in
    let byte = 

    //this is the number of 
    let bits_consumed_inside_byte = quote!{ (#total_bits_consumed % 8) };

    let size_in_bits = quote!{ (<#derivable>::SIZE_IN_BITS) };

    //first byte bitmask
    let bitmask = get_bitmask(&size_in_bits, &bits_consumed_inside_byte)

    let byte_token = get_array_indices(array_name, total_bits_consumed ,size_in_bits);

    // read the byte, mask it for the bits we want, 
    //and bit shift them back to the beginning of the u8
    //finally pass that value into from_u8
    quote!{ 
        let result_array = 
        (#byte_token & #bitmask) >> (8 - (#size_in_bits + #bits_consumed_inside_byte)) 
    }
}