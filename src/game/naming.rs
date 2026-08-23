pub fn normalize_name(name: &str) -> String {
    let mut normalized = String::new();

    for character in name.chars() {
        if character.is_alphanumeric() {
            normalized.extend(character.to_lowercase());
        } else if matches!(character, '\'' | '’' | '‘' | 'ʼ' | '"' | '“' | '”') {
            // Quotes and apostrophes do not create a word boundary. This makes
            // "founder's" and "founders" resolve to the same canonical name.
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_ignores_case_and_extra_whitespace() {
        assert_eq!(normalize_name("  Rusty   KEY  "), "rusty key");
    }

    #[test]
    fn normalization_ignores_quotes_and_apostrophes() {
        assert_eq!(normalize_name("\"Founder’s Plaque\""), "founders plaque");
        assert_eq!(normalize_name("Founder's Plaque"), "founders plaque");
    }

    #[test]
    fn normalization_treats_other_punctuation_as_word_boundaries() {
        assert_eq!(normalize_name("iron-bound/chest"), "iron bound chest");
    }
}
