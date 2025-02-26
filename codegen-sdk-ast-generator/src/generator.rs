use codegen_sdk_common::language::Language;
pub fn generate_ast(language: &Language) -> anyhow::Result<String> {
    let content = format!(
        "
    #[derive(Debug, Clone)]
    pub struct {language_struct_name}File {{
        node: {language_name}::{root_node_name},
        path: PathBuf,
        pub references: References,
        pub definitions: Definitions
    }}
    impl File for {language_struct_name}File {{
        fn path(&self) -> &PathBuf {{
            &self.path
        }}
        fn parse(path: &PathBuf) -> Result<Self, ParseError> {{
            log::debug!(\"Parsing {language_name} file: {{}}\", path.display());
            let ast = {language_name}::{language_struct_name}::parse_file(path)?;
            let mut references = References::default();
            let mut definitions = Definitions::default();
            ast.drive(&mut definitions);
            ast.drive(&mut references);
            Ok({language_struct_name}File {{ node: ast, path: path.clone(), references, definitions }})
        }}
    }}
    impl HasNode for {language_struct_name}File {{
        type Node = {language_name}::{root_node_name};
        fn node(&self) -> &Self::Node {{
            &self.node
        }}
    }}
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
