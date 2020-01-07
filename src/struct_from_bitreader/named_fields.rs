

pub fn get_named_fields(fields_named: &syn::FieldsNamed, name : &proc_macro2::Ident ) -> proc_macro::TokenStream
{
    let clauses : Vec<proc_macro2::TokenStream> = fields_named.named
    .iter()
    .map(|field| 
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
            #identifier : <#derivable>::from_bitreader(reader).await?
        };

        clause
    }).collect();

    let blah = quote!{
        #[async_trait]
        impl<R> FromBitReader<crate::ERROR, R> for #name 
        where Self : Sized,
            R : Read + std::marker::Unpin + std::marker::Send
        {
            #[allow(trivial_numeric_casts)]
            async fn from_bitreader(reader : &mut bitreader_async::BitReader<R>) -> Result<Self, crate::ERROR>
            {
                Ok(#name
                {
                    #(#clauses),*
                })
            }
        }
    };

    //println!("{}", blah);

    blah.into()
}