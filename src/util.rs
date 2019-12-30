use syn::{ Meta, Attribute };

pub fn get_attribute_value(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Lit>
{
    for attr in attrs.iter()
    {
        match attr.parse_meta()
        {
            Ok(meta_attribute) =>
            {
                match meta_attribute
                {
                    Meta::NameValue(meta_name_value) => 
                    {
                        let path_to_print = &meta_name_value.path;

                        match &*quote!{#path_to_print}.to_string() == token
                        {
                            true => return Some(meta_name_value.lit),
                            false => return None
                        }
                        
                    },
                    Meta::Path(_path) => {},
                    Meta::List(_meta_list) => {}
                }
            },
            _ => return None
        }
    }

    return None
}

pub fn get_attribute_path(attrs : &Vec<syn::Attribute>, token : &str) -> Option<syn::Lit>
{
    for attr in attrs.iter()
    {
        match attr.parse_meta()
        {
            Ok(meta_attribute) =>
            {
                match meta_attribute
                {
                    Meta::NameValue(meta_name_value) => 
                    {
                        let path_to_print = &meta_name_value.path;

                        match &*quote!{#path_to_print}.to_string() == token
                        {
                            true => return Some(meta_name_value.lit),
                            false => return None
                        }
                        
                    },
                    Meta::Path(_path) => {},
                    Meta::List(_meta_list) => {}
                }
            },
            _ => return None
        }
    }

    return None
}


pub fn has_meta_path(attrs : &Vec<Attribute>, token : &str) -> bool
{
    //for all the attributes
    for attr in attrs.iter()
    {
        if let Ok(meta) = attr.parse_meta() 
        {
            //if the attribute matches the token, return true
            if meta.path().is_ident(token) 
            {
                return true;
            }
        }
    }

    return false;
}
