const WHITESPACE: &[char] = &['\t', '\n', '\x0C', '\r', ' '];

pub fn preprocess_input(idl_source: &str) -> Result<String, String> {
    for line in idl_source.lines() {
        let mut split_line = line.split(WHITESPACE);

        // let first_token = split_line.peek();

        println!("Split line: {:?}", split_line);

        if let Some("#define") = split_line.next() {
            todo!("#define not yet implemented")
        } else {
        }
    }

    Ok(idl_source
        .lines()
        .filter(|l| {
            if let Some(first_token) = l.split(WHITESPACE).next() {
                !first_token.starts_with("#")
            } else {
                true
            }
        })
        .collect())
}
