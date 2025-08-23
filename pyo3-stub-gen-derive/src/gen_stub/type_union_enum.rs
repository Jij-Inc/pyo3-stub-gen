use super::{extract_documents, StubType};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, spanned::Spanned, Error, Fields, ItemEnum, Result, Type};

pub struct TypeUnionEnumInfo {
    pyclass_name: String,
    enum_type: Type,
    variants: Vec<Type>,
    doc: String,
}

impl From<&TypeUnionEnumInfo> for StubType {
    fn from(info: &TypeUnionEnumInfo) -> Self {
        let TypeUnionEnumInfo {
            pyclass_name,
            enum_type,
            ..
        } = info;
        Self {
            ty: enum_type.clone(),
            name: pyclass_name.clone(),
            module: None,
        }
    }
}

impl TryFrom<ItemEnum> for TypeUnionEnumInfo {
    type Error = Error;

    fn try_from(item: ItemEnum) -> Result<Self> {
        let ItemEnum {
            variants,
            attrs,
            ident,
            ..
        } = item;

        let doc = extract_documents(&attrs).join("\n");

        let enum_type = parse_quote!(#ident);
        let pyclass_name = ident.clone().to_string();

        let variants = variants
            .into_iter()
            .map(|var| -> Result<Type> {
                let var_span = var.span();
                let Fields::Unnamed(fields) = var.fields else {
                    return Err(syn::Error::new(
                        var_span,
                        "Enum variant must be 1-tuple".to_owned(),
                    ));
                };

                if fields.unnamed.len() != 1 {
                    return Err(syn::Error::new(
                        var_span,
                        "Enum variant must be 1-tuple".to_owned(),
                    ));
                }

                let variant_type = fields.unnamed[0].ty.clone();
                Ok(variant_type)
            })
            .collect::<Result<Vec<Type>>>()?;

        Ok(Self {
            doc,
            enum_type,
            pyclass_name,
            variants,
        })
    }
}

impl ToTokens for TypeUnionEnumInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            enum_type,
            variants,
            doc,
            ..
        } = self;

        let variants = variants.iter().map(|variant_type| {
            quote! {<#variant_type as pyo3_stub_gen::PyStubType>::type_output}
        });
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::TypeUnionEnumInfo {
                pyclass_name: #pyclass_name,
                enum_id: std::any::TypeId::of::<#enum_type>,
                variants: &[ #( #variants ),* ],
                doc: #doc,
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test() -> Result<()> {
        let input: ItemEnum = parse_str(
            r#"
            #[derive(FromPyObject)]
            pub enum FunctionAgg {
                S(String),
                I(i32),
                F(f32),
            }
            "#,
        )?;
        let out = TypeUnionEnumInfo::try_from(input)?.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::TypeUnionEnumInfo {
            pyclass_name: "FunctionAgg",
            enum_id: std::any::TypeId::of::<FunctionAgg>,
            variants: &[
                <String as pyo3_stub_gen::PyStubType>::type_output,
                <i32 as pyo3_stub_gen::PyStubType>::type_output,
                <f32 as pyo3_stub_gen::PyStubType>::type_output,
            ],
            doc: "",
        }
        "#);
        Ok(())
    }

    fn format_as_value(tt: TokenStream2) -> String {
        let ttt = quote! { const _: () = #tt; };
        let formatted = prettyplease::unparse(&syn::parse_file(&ttt.to_string()).unwrap());
        formatted
            .trim()
            .strip_prefix("const _: () = ")
            .unwrap()
            .strip_suffix(';')
            .unwrap()
            .to_string()
    }
}
