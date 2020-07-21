use crate::util::{ get_meta_attribute, ident_from_meta_attribute, expr_from_meta_attribute };

use syn::{ Type, PathArguments, GenericArgument,  };

#[derive(Clone)]
pub enum Derivable
{
    InsideOption(syn::Type, syn::Expr),
    Naked(syn::Type)
}

impl Derivable
{
    pub fn get_size_in_bits(&self) -> syn::Expr
    {
        let var = match self
        {
            Derivable::InsideOption(ty, _flag) => ty,
            Derivable::Naked(ty) => ty
        };

        let input : proc_macro::TokenStream = quote!{ <#var>::SIZE_IN_BITS }.into();

        let path : syn::Expr = syn::parse(input).unwrap();

        path
    }

    pub fn get_inner_type(&self) -> syn::Type
    {
        match self
        {
            Derivable::InsideOption(ty, _flag) => ty.clone(),
            Derivable::Naked(ty) => ty.clone()
        }
    }

    pub fn get_raw(&self ) -> proc_macro2::TokenStream
    {
        match self
        {
            Derivable::InsideOption(ty, _flag) => quote!{ Option<#ty> },
            Derivable::Naked(ty) => quote!{ #ty }
        }
    }
}

//Taken from :
//https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn
//TODO: change to from_field and move inside Derivable impl
pub fn get_derivable(field : &syn::Field ) -> Derivable
{
    match field.ty.clone()
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
                _ => panic!("Packattack: Token Inside Option isn't type!")
            };
            
            let flag_ident : syn::Expr = match get_meta_attribute(&field.attrs, "flag")
            {
                Some(meta) => expr_from_meta_attribute(meta),
                None => panic!("Packattack: Type is an Option<> but there was no 'flag' attribute!")
            };

            Derivable::InsideOption(ty.clone(), flag_ident)
        },
        _ => 
        {
            //check if someone accidentally put a flag
            match get_meta_attribute(&field.attrs, "flag")
            {
                Some(meta) => panic!("Packattack: Attribute 'flag' was set but marked type wasn't an 'Option' type!"),
                //No flag just llike we expect return Derivable Naked
                None => Derivable::Naked(field.ty.clone())
            }
        }
    }
}

fn path_is_option(path: &syn::Path) -> bool {
    path.leading_colon.is_none()
    && path.segments.len() == 1
    && path.segments.iter().next().unwrap().ident == "Option"
}