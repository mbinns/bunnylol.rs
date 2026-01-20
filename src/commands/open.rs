/// Open command handler
/// Supports: open (FQDN)
use crate::commands::bunnylol_command::{BunnylolCommand, BunnylolCommandInfo};

pub struct OpenCommand;

impl BunnylolCommand for OpenCommand {
    const BINDINGS: &'static [&'static str] = &["open"];

    fn process_args(args: &str) -> String {
        let fqdn = Self::get_command_args(args).trim();

        if fqdn.is_empty() {
            return "https://".to_string();
        }

        if fqdn.starts_with("http://") || fqdn.starts_with("https://") {
            fqdn.to_string()
        } else {
            format!("https://{}", fqdn)
        }
    }

    fn get_info() -> BunnylolCommandInfo {
        BunnylolCommandInfo {
            bindings: Self::BINDINGS.iter().map(|s| s.to_string()).collect(),
            description: "Open an arbitrary website by FQDN".to_string(),
            example: "open example.com".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_command_fqdn() {
        assert_eq!(
            OpenCommand::process_args("open example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn test_open_command_with_path() {
        assert_eq!(
            OpenCommand::process_args("open example.com/docs"),
            "https://example.com/docs"
        );
    }

    #[test]
    fn test_open_command_https_url() {
        assert_eq!(
            OpenCommand::process_args("open https://example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn test_open_command_http_url() {
        assert_eq!(
            OpenCommand::process_args("open http://example.com"),
            "http://example.com"
        );
    }

    #[test]
    fn test_open_command_no_args() {
        assert_eq!(OpenCommand::process_args("open"), "https://");
    }
}
