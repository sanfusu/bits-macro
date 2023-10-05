use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments,
    PathArguments::AngleBracketed, Token,
};

pub struct BitStructAttr {
    // 必须是 u8, u16, u32, u64,u128
    pub base_ty: syn::Path,
    pub export: syn::Visibility,
}
impl Parse for BitStructAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input_ast = Punctuated::<syn::Path, Token![,]>::parse_separated_nonempty(input)?;

        let mut base_ty: Option<syn::Path> = None;
        let mut export = syn::Visibility::Inherited;
        for ref attr in input_ast {
            if attr.is_ident("u8")
                || attr.is_ident("u16")
                || attr.is_ident("u32")
                || attr.is_ident("u64")
                || attr.is_ident("u128")
            {
                if base_ty.is_some() {
                    return Err(syn::Error::new(attr.span(), "Duplicated container type"));
                }
                base_ty = Some(attr.clone());
            } else if let Some(seg) = attr.segments.first() {
                if seg.ident.to_string() != "export" {
                    return Err(syn::Error::new(attr.span(), "Unknow attribute"));
                }
                let path_segment = attr.segments.first().unwrap();
                if path_segment.arguments.is_empty() {
                    let public = syn::parse2::<Token![pub]>(quote! {pub}).unwrap();
                    export = syn::Visibility::Public(public);
                } else if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                    &path_segment.arguments
                {
                    if args.len() != 1 {
                        return Err(syn::Error::new(
                            args.span(),
                            "Only one path can be in export",
                        ));
                    }
                    let args = args.first().unwrap();
                    if let syn::GenericArgument::Type(syn::Type::Path(path)) = args {
                        export = syn::parse2::<syn::Visibility>(quote! {pub(in #path)}).unwrap();
                    } else {
                        return Err(syn::Error::new(
                            args.span(),
                            "export's argument must be a path",
                        ));
                    }
                }
            }
        }
        Ok(BitStructAttr {
            base_ty: base_ty.ok_or(syn::Error::new(
                input.span(),
                "Container(base) type must be specified",
            ))?,
            export,
        })
    }
}
