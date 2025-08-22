// IO
pub fn file_to_chars<T: AsRef<str>>(path: T) -> std::io::Result<Vec<char>> {
    let p = path.as_ref();
    let data = std::fs::read_to_string(p)?;
    let code = data.chars().collect::<Vec<_>>();
    Ok(code)
}
