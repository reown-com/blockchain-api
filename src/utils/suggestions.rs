/// Returns suggested words from the dictionary that start with the given
/// prefix.
pub fn dictionary_suggestions(start_with: &str) -> Vec<&str> {
    // The dictionary is a list of words separated by newlines
    let dictionary_contents = include_str!("../../assets/names_dictionary.txt");
    let candidates: Vec<&str> = dictionary_contents
        .lines()
        .filter(|&suggested_name| {
            suggested_name.starts_with(start_with) && suggested_name != start_with
        })
        .collect();
    candidates
}
