use crate::generate::{Arg, MethodDef, MethodType};
use crate::type_info::{PyComplexEnumInfo, VariantForm, VariantInfo};
use crate::TypeInfo;
use std::collections::{HashMap, HashSet};

pub(super) fn get_variant_methods(
    enum_info: &PyComplexEnumInfo,
    info: &VariantInfo,
) -> HashMap<String, Vec<MethodDef>> {
    let full_class_name = format!("{}.{}", enum_info.pyclass_name, info.pyclass_name);

    let mut methods: HashMap<String, Vec<MethodDef>> = HashMap::new();

    methods
        .entry("__new__".to_string())
        .or_default()
        .push(MethodDef {
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
        let len_name = "__len__";
        methods
            .entry(len_name.to_string())
            .or_default()
            .push(MethodDef {
                name: len_name,
                args: Vec::new(),
                r#return: TypeInfo::builtin("int"),
                doc: "",
                r#type: MethodType::Instance,
            });

        let getitem_name = "__getitem__";
        methods
            .entry(getitem_name.to_string())
            .or_default()
            .push(MethodDef {
                name: getitem_name,
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
