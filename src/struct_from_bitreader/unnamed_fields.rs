use crate::util::{ get_meta_attribute_as_ident, get_meta_attribute_as_expr, get_option_inner_type};

pub fn get_unamed_fields(fields_unnamed: &syn::FieldsUnnamed, name : &proc_macro2::Ident) -> proc_macro::TokenStream
{
    //these are the fields marked with the attribute #[expose]
    let exposed_fields : Vec<_> = fields_unnamed.unnamed
    .iter().filter(|field| if let Some(_name_lit) = get_meta_attribute_as_ident(&field.attrs, "expose") { true } else { false }).map(|field|
    {
       let name_lit = get_meta_attribute_as_ident(&field.attrs, "expose").unwrap();
       let field_lit = &field.ty;

       quote!{ let #name_lit : #field_lit; }
    }).collect();

    //Generate clauses for all fields 
    let clauses : Vec<proc_macro2::TokenStream> = fields_unnamed.unnamed
    .iter()
    .map(|field| 
    {
        let derivable = &field.ty;

        //TODO: Make this a reference instead of a clone into a local variable
        //if this field has the expose meta attribute then we assign to the global variable 
        if let Some(name) = get_meta_attribute_as_ident(&field.attrs, "expose")
        {
            //otherwise we're still inside the byte so just read from the byte
            return quote! { { #name = <#derivable>::from_bitreader(reader).await?; #name.clone() } }
        }
        else if let Some(name_expr) = get_meta_attribute_as_expr(&field.attrs, "flag")
        {
            //Get the type inside the Option
            let inner_type = match get_option_inner_type(&derivable)
            {
                Some(ty) => ty,
                None => panic!(" flag value 'flag' was set but marked type wasn't an 'Option' type!")
            };

            //otherwise we're still inside the byte so just read from the byte
            return quote! { { 
                    match #name_expr
                    {
                        true => Some(<#inner_type>::from_bitreader(reader).await?),
                        false => None
                    } 
            } }
        }
        else if let Some(length_expr) = get_meta_attribute_as_expr(&field.attrs, "length")
        {

            //otherwise we're still inside the byte so just read from the byte
            return quote! {
                {
                    let vec_buffer : Vec<u8> = reader.read_u8_slice_aligned( (#length_expr) as usize).await?;

                    vec_buffer
                }
            }
        }

        //otherwise we're still inside the byte so just read from the byte
        quote! { <#derivable>::from_bitreader(reader).await? }

        
    }).collect();

    let blah = quote!{

        #[async_trait::async_trait]
        impl<R> FromBitReader<R> for #name where Self : Sized, R : Read + std::marker::Unpin + std::marker::Send
        {
            async fn from_bitreader(reader : &mut bitreader_async::BitReader<R>) -> Result<#name>
            {

                #(#exposed_fields)*

                Ok(#name
                (
                    #(#clauses),*
                ))
            }
        }
    };

    blah.into()
}