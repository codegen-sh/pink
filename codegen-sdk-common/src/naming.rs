use std::debug_assert;

use convert_case::{Case, Casing};
use phf::phf_map;

static MAPPINGS: phf::Map<char, &'static str> = phf_map! {
    '-' => "Minus",
    '.' => "Dot",
    '(' => "OpenParen",
    ')' => "CloseParen",
    '[' => "OpenBracket",
    ']' => "CloseBracket",
    '{' => "OpenBrace",
    '}' => "CloseBrace",
    '=' => "Equals",
    '!' => "Bang",
    '<' => "LessThan",
    '>' => "GreaterThan",
    '?' => "QuestionMark",
    ':' => "Colon",
    ',' => "Comma",
    '/' => "Slash",
    '\\' => "Backslash",
    '@' => "At",
    '#' => "Hash",
    '$' => "Dollar",
    '%' => "Percent",
    '^' => "Caret",
    '&' => "Ampersand",
    '*' => "Asterisk",
    '+' => "Plus",
    '_' => "Underscore",
    '~' => "Tilde",
    '`' => "Backtick",
    '\'' => "SingleQuote",
    '"' => "DoubleQuote",
    '|' => "Pipe",
    ';' => "Semicolon",
    '\0' => "Null",
};
pub fn field_name_getter(field_name: &str) -> String {
    if field_name == "source" {
        return "source_node".to_string();
    }
    field_name.to_string()
}

pub fn normalize_field_name(field_name: &str) -> String {
    if field_name == "type" {
        return "r#type".to_string();
    }
    if field_name == "macro" {
        return "r#macro".to_string();
    }
    if field_name == "else" {
        return "r#else".to_string();
    }
    if field_name == "trait" {
        return "r#trait".to_string();
    }
    field_name.to_string()
}
fn get_char_mapping(c: char) -> String {
    if let Some(mapping) = MAPPINGS.get(&c) {
        return mapping.to_string();
    }
    c.to_string()
}
fn escape_char(c: char) -> String {
    if c == '\'' {
        return "\\'".to_string();
    }
    if c == '"' {
        return "\\\"".to_string();
    }
    if c == '\\' {
        return "\\\\".to_string();
    }
    c.to_string()
}
pub fn normalize_string(string: &str) -> String {
    let escaped = String::from_iter(string.chars().map(escape_char));
    escaped
}
pub fn normalize_type_name(type_name: &str, named: bool) -> String {
    if type_name == "self" {
        return "SelfNode".to_string();
    }
    let mut cased = type_name.to_string();
    if type_name.chars().any(|c| c.is_ascii_alphabetic()) {
        cased = cased.to_case(Case::Pascal);
    }
    let escaped = String::from_iter(cased.chars().map(get_char_mapping));
    debug_assert!(
        escaped
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit()),
        "Type name '{}' contains invalid characters",
        type_name
    );
    if named {
        escaped
    } else {
        format!("Anonymous{}", escaped)
    }
}
