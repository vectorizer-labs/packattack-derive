use syn::{Type, GenericArgument, PathArguments, Meta};

pub fn get_meta_attribute_as_ident(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Ident>
{
    let attribute_literal = get_meta_attribute(attrs, token);

    match attribute_literal
    {
        Some(attr_lit) => Some(ident_from_lit(attr_lit)),
        None => None
    }
}

pub fn get_meta_attribute_as_expr(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Expr>
{
    let attribute_literal = get_meta_attribute(attrs, token);

    match attribute_literal
    {
        Some(attr_lit) => Some(expr_from_lit(attr_lit)),
        None => None
    }
}

//Taken from :
//https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn
//TODO: This is probably overkill for our use case so maybe come back and double check the logic
pub fn get_option_inner_type<'a>(derivable : &'a syn::Type) -> Option< &'a syn::Type>
{
    match derivable
    {
        Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => 
        {
            // Get the first segment of the path (there is only one, in fact: "Option"):
            let type_params = &typepath.path.segments.iter().nth(0).unwrap().arguments;
            // It should have only on angle-bracketed param ("<String>"):
            let generic_arg = match type_params {
                PathArguments::AngleBracketed(params) => params.args.iter().nth(0).unwrap(),
                _ => panic!("Packattack: No Type or Bad Type Found Inside Option!")
            };

            // This argument must be a type:
            let ty = match generic_arg {
                GenericArgument::Type(ty) => ty,
                _ => panic!("Packattack: Type Found Inside Option Doesn't Parse!")
            };

            Some(ty)
        },
        _ => return None
    }
}

fn path_is_option(path: &syn::Path) -> bool {
    path.leading_colon.is_none()
    && path.segments.len() == 1
    && path.segments.iter().next().unwrap().ident == "Option"
}

pub fn get_meta_attribute(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Lit>
{
    //for all the attributes
    for attr in attrs.iter()
    {
        if let Ok(meta) = attr.parse_meta() 
        {
            //if the attribute matches the token, return true
            if meta.path().is_ident(token) 
            {
                match meta
                {
                    Meta::NameValue(meta_name_value) => 
                    {
                        return Some(meta_name_value.lit)
                    },
                    Meta::Path(path) => panic!("Packattack requires attributes to be meta name value pairs but a path {} was found!", quote!{ #path } ),
                    Meta::List(meta_list) => panic!("Packattack requires attributes to be meta name value pairs but a meta list {} was found!", quote!{ #meta_list } )
                }
            }
        }
    }

    return None;
}

fn ident_from_lit( literal : syn::Lit) -> syn::Ident
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
        _=> panic!(" Packattack only supports string literals as expose name!")
    }
}
