pub fn generate_casings_from_kebab<T: AsRef<str>>(
    kebab_values: &[T],
) -> (Vec<String>, Vec<String>) {
    let mut pascal = Vec::with_capacity(kebab_values.len());
    let mut snake = Vec::with_capacity(kebab_values.len());
    for name in kebab_values {
        let n = name.as_ref();
        pascal.push(kebab_to_pascal(n));
        snake.push(kebab_to_snake(n));
    }
    (pascal, snake)
}

pub fn is_kebab(s: &str) -> bool {
    s.chars()
        .all(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn kebab_to_pascal(kebab: &str) -> String {
    let mut pascal = String::with_capacity(kebab.len() + 1);
    let first = kebab.chars().next();
    // add underscore, since non alphabetic can't start an identifier
    if first.is_some_and(|c| !c.is_alphabetic()) {
        pascal.push('_');
    }
    for part in kebab.split('-') {
        let Some(c) = part.chars().next() else {
            continue;
        };
        pascal.push(c.to_ascii_uppercase());
        pascal.push_str(&part[c.len_utf8()..]);
    }
    if pascal.contains('+') {
        // c++, hehh
        return pascal.replace('+', "_plus");
    }
    pascal
}

pub fn kebab_to_snake(kebab: &str) -> String {
    kebab.replace('-', "_")
}
