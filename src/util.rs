use syn::{Meta};

pub fn ident_from_meta_attribute(attr : syn::Meta) -> syn::Ident
{
    match attr
    {
        Meta::NameValue(meta_name_value) => 
        {
            return ident_from_lit(meta_name_value.lit)
        },
        _ => panic!("Expected meta attribute '{}' to be a NameValue pair, but it was another type of attribute", quote!{ #attr } )
    }
}

pub fn expr_from_meta_attribute(attr : syn::Meta) -> syn::Expr
{
    match attr
    {
        Meta::NameValue(meta_name_value) => 
        {
            return expr_from_lit(meta_name_value.lit)
        },
        _ => panic!("Expected meta attribute '{}' to be a NameValue pair, but it was another type of attribute", quote!{ #attr } )
    }
}

pub fn lit_from_meta_attribute(attr : syn::Meta) -> syn::Lit
{
    match attr
    {
        Meta::NameValue(meta_name_value) => 
        {
            return meta_name_value.lit
        },
        _ => panic!("Expected meta attribute '{}' to be a NameValue pair, but it was another type of attribute", quote!{ #attr } )
    }
}

pub fn get_meta_attribute(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Meta>
{
    //for all the attributes
    for attr in attrs.iter()
    {
        if let Ok(meta) = attr.parse_meta() 
        {
            //if the attribute matches the token, return true
            if meta.path().is_ident(token) 
            {
                return Some(meta);
            }
        }
    }

    return None;
}



/*
If you're going to be reading attributes you should match using an 
if let Meta::NameValue(name) = blah()
to ensure you're using the right type of attribute

Meta::NameValue(meta_name_value) => 
{
    return Some(meta_name_value.lit)
},
Meta::Path(path) => panic!("Packattack requires attributes to be meta name value pairs but a path {} was found!", quote!{ #path } ),
Meta::List(meta_list) => panic!("Packattack requires attributes to be meta name value pairs but a meta list {} was found!", quote!{ #meta_list } )

*/

pub fn ident_from_lit( literal : syn::Lit) -> syn::Ident
{
    match literal
    {
        syn::Lit::Str(lit_str) => 
        {
            let str_value = lit_str.value();
            syn::parse_str::<syn::Ident>(str_value.as_str()).unwrap()
        },
        _=> panic!(" Packattack only supports string literals as expose name!")
    }
}

fn expr_from_lit(literal : syn::Lit) -> syn::Expr
{
    match literal
    {
        syn::Lit::Str(lit_str) => 
        {
            let str_value = lit_str.value();
            syn::parse_str::<syn::Expr>(str_value.as_str()).unwrap()
        },
        syn::Lit::Int(lit_int) => 
        {
            syn::parse_str::<syn::Expr>(lit_int.to_string().as_str()).unwrap()
        }
        _=> panic!(" Packattack only supports string and int literals : {:#?} is not one!", quote!{ #literal })
    }
}

pub fn ident_from_str(name : &str) -> syn::Ident
{
    match syn::parse_str::<syn::Ident>(name)
    {
        Ok(ident) => ident,
        Err(_e) => panic!("Packattack internal  error: couldn't use the name of datatype {} as an identifier!", name)
    }
}

/*
fn get_slice_name(name : &syn::Ident, count : usize) -> syn::Ident
{
    //push the first slice identifier
    let mut slice_string = name.to_string();
    slice_string.push_str("_slice_");
    slice_string.push_str(count.to_string().as_str());

    match syn::parse_str::<syn::Ident>(slice_string.as_str())
    {
        Ok(slice) => slice,
        Err(_e) => panic!("Packattack internal error: Couldn't parse slice name!")
    }
}*/
