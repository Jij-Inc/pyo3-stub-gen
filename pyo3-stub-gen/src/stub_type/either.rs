use std::collections::HashSet;

use super::{PyStubType, TypeInfo};

impl<L: PyStubType, R: PyStubType> PyStubType for either::Either<L, R> {
    fn type_input() -> TypeInfo {
        let TypeInfo {
            name: name_l,
            quote: quote_l,
            import: import_l,
        } = L::type_input();
        let TypeInfo {
            name: name_r,
            quote: quote_r,
            import: import_r,
        } = R::type_input();

        let mut import: HashSet<_> = import_l.into_iter().chain(import_r).collect();

        import.insert("typing".into());

        TypeInfo {
            name: format!("typing.Union[{name_l}, {name_r}]"),
            quote: quote_l || quote_r,
            import,
        }
    }
    fn type_output() -> TypeInfo {
        let TypeInfo {
            name: name_l,
            quote: quote_l,
            import: import_l,
        } = L::type_output();
        let TypeInfo {
            name: name_r,
            quote: quote_r,
            import: import_r,
        } = R::type_output();

        let mut import: HashSet<_> = import_l.into_iter().chain(import_r).collect();

        import.insert("typing".into());

        TypeInfo {
            name: format!("typing.Union[{name_l}, {name_r}]"),
            quote: quote_l || quote_r,
            import,
        }
    }
}
