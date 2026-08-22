use super::ids::ItemId;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
}

pub fn normalize_item_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_name_normalization_ignores_case_and_extra_whitespace() {
        assert_eq!(normalize_item_name("  Rusty   KEY  "), "rusty key");
    }
}
