use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, Expr, ExprPath, ExprRange, Ident,
    Token, Visibility,
};

// 一般会默认定义一个 unint 结构体，该结构体的类型名为字段名。
// 该类型名可能会和 target_ty 冲突，因此需要放在一个单独的 mod 空间中。
pub struct BitStructFieldInfo {
    // 目标类型是可选的，如果为 none，则默认为 BitContainerInfo::base_ty
    pub target_ty: syn::Type,
    pub name: Ident,
    pub attr: BitFieldAttr,
    pub vis: Visibility,
    pub doc: Vec<syn::Attribute>,
}

#[derive(PartialEq, Eq)]
pub enum BitFieldPerm {
    R,
    W,
    RW,
}
impl Default for BitFieldPerm {
    fn default() -> Self {
        BitFieldPerm::R
    }
}

pub struct BitFieldAttr {
    // 是否实现 TryReadableField，默认不实现
    pub need_try: bool,
    // 默认不可读，且不可写，需要显式说明。
    pub perm: BitFieldPerm,
    pub expr_range: ExprRange,
}
impl TryFrom<syn::Field> for BitStructFieldInfo {
    type Error = syn::Error;

    fn try_from(field: syn::Field) -> Result<Self, Self::Error> {
        let disallowed_attr = field.attrs.iter().find(|&attr| {
            attr.path().is_ident("field") == false && attr.path().is_ident("doc") == false
        });
        if let Some(attr) = disallowed_attr {
            return Err(syn::Error::new(
                attr.span(),
                "Only \"field\" attr or doc is allowed",
            ));
        }
        let mut clean_attr = field
            .attrs
            .iter()
            .filter(|&attr| attr.path().is_ident("field"));
        if clean_attr.clone().count() != 1 {
            let mut span = clean_attr.next().unwrap().span();
            for attr in clean_attr {
                span = span.join(attr.span()).unwrap();
            }
            return Err(syn::Error::new(span, "Only one \"field\" attr is allowed"));
        }
        let field_attr = field
            .attrs
            .iter()
            .find(|&x| x.path().is_ident("field"))
            .ok_or(syn::Error::new(
                field.span(),
                "Attribute \"field\" must be presented",
            ))?;
        let doc_attr = field
            .attrs
            .clone()
            .into_iter()
            .filter(|x| x.path().is_ident("doc"))
            .collect();
        Ok(BitStructFieldInfo {
            target_ty: field.ty.to_owned(),
            name: field.ident.to_owned().ok_or(syn::Error::new(
                field.span().to_owned(),
                "The field must be named",
            ))?,
            attr: field_attr.parse_args()?,
            vis: field.vis,
            doc: doc_attr,
        })
    }
}
impl Parse for BitFieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr_token = Punctuated::<Expr, Token![,]>::parse_separated_nonempty(input)?;
        if attr_token.is_empty() {
            return Err(syn::Error::new(
                attr_token.span(),
                "Attribute \"field\" cannot be empty",
            ));
        }
        let mut perms = attr_token.iter().filter(|&x| {
            if let Expr::Path(ExprPath { path, .. }) = x {
                if path.is_ident("R") || path.is_ident("W") || path.is_ident("RW") {
                    return true;
                }
            }
            return false;
        });
        if perms.clone().count() > 1 {
            let mut span = perms.next().span();
            for perm in perms {
                span = span.join(perm.span()).unwrap();
            }
            return Err(syn::Error::new(
                span,
                "Only a single perm token is allowed, one of (R, W, RW) ",
            ));
        }
        let mut ranges = attr_token.iter().filter(|&x| {
            if let Expr::Range(_) = x {
                return true;
            }
            return false;
        });
        if ranges.clone().count() > 1 {
            let mut span = ranges.next().span();
            for range in ranges {
                span = span.join(range.span()).unwrap();
            }
            return Err(syn::Error::new(
                span,
                "Only a single range token is allowed. Impl virtul bit field instead, if you need a cross bit field",
            ));
        }
        let mut perm = BitFieldPerm::R;
        let mut need_try = false;
        let mut expr_range: Option<ExprRange> = None;
        for ref item in attr_token {
            if let Expr::Path(ExprPath { path, .. }) = item {
                if path.is_ident("R") {
                    perm = BitFieldPerm::R;
                } else if path.is_ident("W") {
                    perm = BitFieldPerm::W;
                } else if path.is_ident("RW") {
                    perm = BitFieldPerm::RW;
                } else if path.is_ident("Try") {
                    need_try = true;
                } else {
                    let msg = format!(
                        "Unknown arg: \"{}\" in attribute \"field\"",
                        path.get_ident().unwrap().to_string()
                    );
                    return Err(syn::Error::new(item.span(), msg));
                }
            } else if let Expr::Range(range) = item {
                let value: ExprRange = range.to_owned();
                expr_range = Some(value);
            } else {
                return Err(syn::Error::new(item.span(), "Cannot parse as field attr"));
            }
        }
        return Ok(Self {
            need_try,
            perm,
            expr_range: expr_range.unwrap(),
        });
    }
}
