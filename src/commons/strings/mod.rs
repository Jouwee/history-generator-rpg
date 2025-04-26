pub(crate) struct Strings {}

impl Strings {
    pub(crate) fn capitalize(string: &str) -> String {
        let mut capitalized = String::new();
        for (i, char) in string.chars().enumerate() {
            if i == 0 {
                capitalized.push(char.to_ascii_uppercase());
            } else {
                capitalized.push(char.to_ascii_lowercase());
            }
        }
        return capitalized;
    }
}