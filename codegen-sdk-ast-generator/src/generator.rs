use codegen_sdk_common::language::Language;
pub fn generate_ast(language: &Language) -> anyhow::Result<String> {
    let content = format!(
        "
    #[derive(Debug, Clone)]
    #[salsa::tracked]
    pub struct {language_struct_name}File {{
        node: {language_name}::{root_node_name},
        path: PathBuf,
    }}
    impl File for {language_struct_name}File {{
        fn path(&self) -> &PathBuf {{
            &self.path
        }}
        fn parse(path: &PathBuf) -> Result<Self, ParseError> {{
            log::debug!(\"Parsing {language_name} file: {{}}\", path.display());
            let ast = {language_name}::{language_struct_name}::parse_file(path)?;
            Ok({language_struct_name}File {{ node: ast, path: path.clone() }})
        }}
    }}
    impl {language_struct_name}File {{
        #[salsa::tracked]
        pub fn definitions(&self) -> Definitions {{
            let mut definitions = Definitions::default();
            self.node.drive(&mut definitions);
            definitions
        }}
        #[salsa::tracked]
        pub fn references(&self) -> References {{
            let mut references = References::default();
            self.node.drive(&mut references);
            references
        }}
    }}
    impl HasNode for {language_struct_name}File {{
        type Node = {language_name}::{root_node_name};
        fn node(&self) -> &Self::Node {{
            &self.node
        }}
    }}
    #[salsa::jar]
    pub struct Jar (
        Definitions,
        References,
        {language_struct_name}File,
    );
    ",
        language_struct_name = language.struct_name,
        language_name = language.name(),
        root_node_name = language.root_node(),
    );
    // for (name, query) in language.definitions() {
    //     content.push_str(&format!("
    //     impl {language_struct_name}File {{
    //         pub fn {name}(&self) -> {language_struct_name}File {{
    //             {language_struct_name}File {{
    //                 node: self.node.children().find(|node| node.type_name == \"{name}\").unwrap(),
    //                 path: self.path.clone()
    //             }}
    //         }}
    //     "));
    // }
    Ok(content)
}
