/// Kagi Search command handler
/// Supports: kagi [search terms], kg [search terms]
use crate::commands::bunnylol_command::{BunnylolCommand, BunnylolCommandInfo};
use crate::utils::url_encoding::build_search_url;

pub struct KagiCommand;

impl BunnylolCommand for KagiCommand {
    const BINDINGS: &'static [&'static str] = &["kagi", "kg"];

    fn process_args(args: &str) -> String {
        let query = Self::get_command_args(args);
        build_search_url("https://kagi.com/search", "q", query)
    }

    fn get_info() -> BunnylolCommandInfo {
        BunnylolCommandInfo {
            bindings: Self::BINDINGS.iter().map(|s| s.to_string()).collect(),
            description: "Search Kagi".to_string(),
            example: "kagi rust programming".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kagi_command_simple() {
        assert_eq!(
            KagiCommand::process_args("privacy search"),
            "https://kagi.com/search?q=privacy%20search"
        );
    }

    #[test]
    fn test_kagi_command_with_prefix() {
        assert_eq!(
            KagiCommand::process_args("kagi rust programming"),
            "https://kagi.com/search?q=rust%20programming"
        );
    }

    #[test]
    fn test_kagi_command_with_short_prefix() {
        assert_eq!(
            KagiCommand::process_args("kg test"),
            "https://kagi.com/search?q=test"
        );
    }

    #[test]
    fn test_kagi_command_empty() {
        assert_eq!(
            KagiCommand::process_args("kagi"),
            "https://kagi.com/search?q="
        );
    }
}
