use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Token};

pub struct BitContainerAttr {
    // 必须是 u8, u16, u32, u64,u128
    pub base_ty: syn::Path,
    // 默认不允许重叠
    pub allow_overlap: bool,
    pub export: bool,
}
impl Parse for BitContainerAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input_ast = Punctuated::<syn::Path, Token![,]>::parse_separated_nonempty(input)?;

        let mut base_ty: Option<syn::Path> = None;
        let mut allow_overlap: bool = false;
        let mut export = false;
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
            } else if attr.is_ident("allow_overlap") {
                allow_overlap = true;
            } else if attr.is_ident("export") {
                export = true;
            } else {
                return Err(syn::Error::new(attr.span(), "Unknown attr"));
            }
        }
        if base_ty.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "Container(base) type must be specified",
            ));
        }
        Ok(BitContainerAttr {
            base_ty: base_ty.unwrap(),
            allow_overlap,
            export,
        })
    }
}
