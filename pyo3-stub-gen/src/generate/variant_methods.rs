use crate::generate::{Arg, MethodDef, MethodType};
use crate::type_info::{PyComplexEnumInfo, VariantForm, VariantInfo};
use crate::TypeInfo;
use std::collections::HashSet;

pub(super) fn get_variant_methods(
    enum_info: &PyComplexEnumInfo,
    info: &VariantInfo,
) -> Vec<MethodDef> {
    let full_class_name = format!("{}.{}", enum_info.pyclass_name, info.pyclass_name);

    let mut methods = Vec::new();

    methods.push(MethodDef {
        name: "__new__",
        args: info.constr_args.iter().map(|a| a.into()).collect(),
        r#return: TypeInfo {
            name: full_class_name,
            import: HashSet::new(),
        },
        doc: "",
        r#type: MethodType::New,
    });

    if let VariantForm::Tuple = info.form {
        methods.push(MethodDef {
            name: "__len__",
            args: Vec::new(),
            r#return: TypeInfo::builtin("int"),
            doc: "",
            r#type: MethodType::Instance,
        });

        methods.push(MethodDef {
            name: "__getitem__",
            args: vec![Arg {
                name: "key",
                r#type: TypeInfo::builtin("int"),
                signature: None,
            }],
            r#return: TypeInfo::any(),
            doc: "",
            r#type: MethodType::Instance,
        });
    }

    methods
}
