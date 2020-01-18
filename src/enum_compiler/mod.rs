use syn::{ Fields, Variant, token::Comma, punctuated::Punctuated };

mod clauses;

pub fn get_enum_match(size_in_bits : &syn::Expr, byte_token : proc_macro2::TokenStream) -> proc_macro2::TokenStream
{
    //create the bitmask for this byte
    let bitmask : proc_macro2::TokenStream = quote!{ ((1 << #size_in_bits) - 1) << (8 - #size_in_bits) };

    // read the byte, mask it for the bits we want, 
    //and bit shift them back to the beginning of the u8
    //finally pass that value into from_u8
    quote!{ (#byte_token & #bitmask) >> (8 - #size_in_bits) }
}

pub fn enum_from_bytes(variants: &Punctuated<Variant, Comma>, 
    name : &proc_macro2::Ident, 
    size_in_bits : &syn::Expr) -> proc_macro2::TokenStream
{
    let clauses = clauses::get_clauses(variants, name, size_in_bits);
    
    let reader_literal = get_enum_match(size_in_bits, quote!{ byte });

    quote! 
    {
        let num = #reader_literal;

        match num
        {
            #(#clauses)*
            _ => panic!("uh oh no match")
        }
    }
}

pub fn enum_from_reader(variants: &Punctuated<Variant, Comma>, 
    name : &proc_macro2::Ident, 
    size_in_bits : &syn::Expr) -> proc_macro2::TokenStream
{
    let clauses = clauses::get_clauses(variants, name, size_in_bits);
    
    let reader_literal = get_enum_match(&size_in_bits, quote!{ first_byte[0] });

    quote! 
    {   
        //read in the first byte
        reader.read_exact(&mut first_byte).await?;

        let num = #reader_literal;

        match num
        {
            #(#clauses)*
            _ => panic!("uh oh no match")
        }
    }
}

/*
fn get_reader_literal(size_in_bits : &syn::Lit) -> proc_macro2::TokenStream
{
    match size_in_bits
    {
        Lit::Int(lit_int) => //TODO: implement conversion,
        Lit::Str(lit_str) => 
        {
            let identifier = Ident::new(lit_str.value().as_str(),lit_str.span());

            quote!{ #identifier::from_bitreader(reader).await? }
        },
        _=> panic!(" Packattack only supports type literals and usizes as size_in_bytes!")
    }
}*/