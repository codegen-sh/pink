use tree_sitter::TextProvider;

pub fn get_text_from_node(node: &tree_sitter::Node, source: &str) -> String {
    String::from_utf8(source.as_bytes().text(*node).next().unwrap().to_vec()).unwrap()
}
