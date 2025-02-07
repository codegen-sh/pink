pub fn format_cst(cst: &str) -> String {
    let parsed = syn::parse_str::<syn::File>(cst)
        .map_err(|e| {
            println!("{:#?}", e);
            e
        })
        .unwrap();

    prettyplease::unparse(&parsed)
}
