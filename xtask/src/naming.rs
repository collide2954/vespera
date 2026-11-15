pub fn is_snake_case(name: &str) -> bool {
    name.split('_').all(|word| {
        !word.is_empty()
            && word
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;

    #[test]
    fn lowercase_words_with_single_underscores_are_snake_case() {
        for name in ["walk", "player_male", "tree_oak_large_01"] {
            assert!(is_snake_case(name), "{name}");
        }
    }

    #[test]
    fn other_names_are_not_snake_case() {
        for name in ["Walk", "walk-fast", "walk__fast", "_walk", "walk_", ""] {
            assert!(!is_snake_case(name), "{name:?}");
        }
    }
}
