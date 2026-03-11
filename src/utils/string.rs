pub fn string_fixed_length(input: &str, length: usize) -> String {
    if input.len() < length {
        return format!("{:<width$}", input, width = length)
    } 

    return input.chars().take(length).collect()
}