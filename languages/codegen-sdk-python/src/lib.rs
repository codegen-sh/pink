#![recursion_limit = "2048"]
#![allow(unused)]
#![allow(non_snake_case)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/python.rs"));
}
pub mod ast {
    use codegen_sdk_ast::References as _;
    include!(concat!(env!("OUT_DIR"), "/python-ast.rs"));
    #[salsa::tracked]
    impl<'db> PythonFile<'db> {
        #[salsa::tracked]
        pub fn get_calls(
            self,
            db: &'db dyn salsa::Database,
            name: String,
        ) -> Vec<crate::cst::Call<'db>> {
            let mut usages = Vec::new();
            if let Some(usage) = self.references(db).calls.get(&name) {
                usages.extend(usage.iter().cloned());
            }
            usages
        }
    }
}
