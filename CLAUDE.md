# CLAUDE.md - Developer Guide for AI Assistants

This guide provides context about the bunnylol.rs repository structure and patterns to help work efficiently on this codebase.

## Project Overview

**bunnylol.rs** is a smart bookmark server written in Rust that lets you create URL shortcuts accessible from your browser's search bar. It's a modern Rust implementation of [bunny1](https://github.com/ccheever/bunny1).

**Tech Stack:**
- **Language:** Rust (2024 edition)
- **Web Framework:** Rocket 0.5 (async)
- **Frontend:** Leptos 0.6 (SSR for bindings page)
- **CLI:** clap 4.5 with subcommands
- **Deployment:** Native services (systemd/launchd/Windows Service) or Docker (compose v2)

**Key Features:**
- Smart URL routing with command patterns (e.g., `gh username/repo` → GitHub)
- Configurable default search engine fallback (`google`, `ddg`, `bing`, `kagi`)
- Multiple aliases per command (e.g., `ig`/`instagram`, `tw`/`twitter`)
- Subcommand support (e.g., `meta pay`, `ig reels`)
- User-defined command aliases from config
- Web portal to view all command bindings and aliases
- Alias management from the web UI (add/delete with flash notices)
- Unified CLI with command execution and server management

## Repository Structure

```
bunnylol.rs/
├── src/
│   ├── main.rs                          # CLI entry point and dispatcher
│   ├── lib.rs                           # Library exports
│   ├── config.rs                        # Configuration (server, aliases, history)
│   ├── bunnylol_command_registry.rs     # Command registry and command metadata cache
│   ├── history.rs                       # Command history persistence
│   ├── server/
│   │   ├── mod.rs                       # Rocket server setup, routes, alias mutations
│   │   ├── service.rs                   # Service install/start/stop/log helpers
│   │   └── web.rs                       # SSR landing page and bindings/aliases UI
│   ├── commands/
│   │   ├── mod.rs                       # Module exports
│   │   ├── github.rs                    # Example: gh command
│   │   ├── instagram.rs                 # Example: ig command with subcommands
│   │   ├── meta.rs                      # Example: meta command with subcommands
│   │   └── [40+ other command files]
│   ├── utils/
│   │   ├── mod.rs                       # Query-string helpers
│   │   └── url_encoding.rs              # URL building helpers
│   └── tests/                           # Integration tests live at repository root
├── Cargo.toml
├── docker-compose.yml
├── Dockerfile
├── README.md
└── CLAUDE.md (this file)
```

## Architecture Patterns

### 1. BunnylolCommand Trait

All commands implement the `BunnylolCommand` trait defined in `src/commands/bunnylol_command.rs`:

```rust
pub trait BunnylolCommand {
    const BINDINGS: &'static [&'static str];  // Command aliases
    fn process_args(args: &str) -> String;     // Returns URL
    fn get_info() -> BunnylolCommandInfo;      // For documentation
}
```

### 2. Command Registration

Commands are registered in two places:

1. **`src/commands/mod.rs`** - Module exports:
   ```rust
   pub use self::github::GitHubCommand;
   pub use self::instagram::InstagramCommand;
   // ... etc
   ```

2. **`src/bunnylol_command_registry.rs`** - In `BunnylolCommandRegistry`:
   - `register_commands!` macro expands the lookup table and command listing
   - `process_command_with_config()` handles command routing plus configurable fallback search
   - `get_all_commands()` powers both CLI listing and the server landing page

### 3. URL Building Helpers

Located in `src/utils/url_encoding.rs`:
- `build_search_url(base, param, query)` - Constructs search URLs with encoded params
- `build_path_url(base, path)` - Appends path to base URL

## How to Add New Commands

### Adding a Brand New Command

1. **Create command file** in `src/commands/your_command.rs`:
   ```rust
   use crate::commands::bunnylol_command::{BunnylolCommand, BunnylolCommandInfo};

   pub struct YourCommand;

   impl BunnylolCommand for YourCommand {
       const BINDINGS: &'static [&'static str] = &["alias1", "alias2"];

       fn process_args(args: &str) -> String {
           let query = Self::get_command_args(args);
           // Return URL based on query
           "https://example.com".to_string()
       }

       fn get_info() -> BunnylolCommandInfo {
           BunnylolCommandInfo::new(
               Self::BINDINGS,
               "Description here",
               "alias1 example",
           )
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_your_command() {
           assert_eq!(YourCommand::process_args("alias1"), "https://example.com");
       }
   }
   ```

2. **Export in `src/commands/mod.rs`**:
   ```rust
   pub mod your_command;
   pub use self::your_command::YourCommand;
   ```

3. **Register in `src/bunnylol_command_registry.rs`** - Add to the `register_commands!` macro:
   ```rust
   register_commands! {
       crate::commands::BindingsCommand,
       // ... other commands ...
       crate::commands::YourCommand,  // ADD YOUR COMMAND HERE
   }
   ```

   **IMPORTANT:** The `register_commands!` macro automatically generates both:
   - `initialize_command_lookup()` - Maps aliases to handlers
   - `get_all_commands_impl()` - Lists all commands for /bindings page

   You only need to add your command once to the macro, and it will be registered everywhere.

### Adding Subcommands to Existing Commands

**Much simpler!** Just edit the existing command file:

1. **Update the `process_args` method** with a match statement:
   ```rust
   fn process_args(args: &str) -> String {
       let query = Self::get_command_args(args);
       match query {
           "subcommand1" => "https://example.com/sub1".to_string(),
           "sub2" | "alias2" => "https://example.com/sub2".to_string(),  // Multiple aliases
           _ => "https://example.com".to_string(),  // Default
       }
   }
   ```

2. **Add tests** for the new subcommands

3. **Update doc comment** at top of file

**No registration needed** - the command is already hooked up!

**Example:** See `src/commands/instagram.rs` for `reels`, `messages`, `msg`, `chat` subcommands, or `src/commands/meta.rs` for `pay`, `accounts`, `ai` subcommands.

## Testing

### Running Tests

```bash
# Run all tests
cargo test --features server

# Run tests for specific command
cargo test instagram
cargo test meta

# Run with output
cargo test -- --nocapture
```

### Test Patterns

All commands include unit tests in `#[cfg(test)]` modules:
- Test base command (no args)
- Test each alias
- Test subcommands
- Test search/dynamic behavior
- Test edge cases

**Example test:**
```rust
#[test]
fn test_instagram_command_reels() {
    assert_eq!(
        InstagramCommand::process_args("ig reels"),
        "https://www.instagram.com/reels/"
    );
}
```

## Building and Running

```bash
# Development
cargo run -- serve             # Starts server on localhost:8000
cargo run -- gh facebook/react # Execute a command
cargo build                   # Build without running
cargo check                   # Fast syntax check

# Docker
docker compose up -d          # Run on port 8000
BUNNYLOL_PORT=9000 docker compose up  # Custom port

# Testing
cargo test                    # Run all tests
cargo test --test ''          # (Don't use - this errors)

# Service management
cargo install --path .
bunnylol service install
bunnylol service status
bunnylol service logs
```

## Key Implementation Details

### Command Resolution Flow

1. User types: `http://localhost:8000/?cmd=ig reels`
2. Rocket routes to main handler
3. Server resolves config aliases first (e.g. `work` → `gh myorg/repo`)
4. `BunnylolCommandRegistry::process_command_with_config()` extracts command: `"ig"`
5. Registry matches `"ig"` to `InstagramCommand`
6. `InstagramCommand::process_args("ig reels")` is called
7. `get_command_args()` strips `"ig"` prefix → `"reels"`
8. Command returns `"https://www.instagram.com/reels/"`
9. Server sends a redirect

If no command matches, the configured default search engine is used.

### Server State Pattern

- The Rocket server stores `BunnylolConfig` inside `AppState { config: RwLock<BunnylolConfig> }`
- The landing page reads config from that shared state on each request
- Alias add/delete routes mutate the in-memory config and then persist it with `BunnylolConfig::save()`
- UI status messages use Rocket flash messages instead of query params

### Landing Page Behavior

- The main page is rendered in `src/server/web.rs` with Leptos SSR
- It has two tabs: built-in commands and configured aliases
- The aliases tab includes:
  - a form to add/update aliases
  - per-alias delete buttons
  - temporary flash notices that auto-dismiss on the client
- The setup/help content is behind a small `?` button in the top-left

### Search Engine Notes

- Unknown commands fall back to the configured search engine in `BunnylolConfig::get_search_url()`
- Supported config fallback values are `google`, `ddg`/`duckduckgo`, `bing`, and `kagi`
- Kagi is also available as an explicit command via `kagi` or `kg`

### Multiple Alias Pattern

```rust
const BINDINGS: &'static [&'static str] = &["alias1", "alias2"];
```

The `matches_command()` trait method automatically checks all bindings.

### Subcommand Pattern with Match

```rust
match query {
    "sub1" => "url1",
    "sub2" | "sub2_alias" => "url2",  // Multiple aliases for one subcommand
    "" => "default_url",               // No args
    _ => {                             // Fallback (search, etc.)
        // Handle dynamic args
    }
}
```

### Special Patterns

- **Prefix commands:** Dollar sign (`$AAPL`) handled specially in `process_prefix_commands()`
- **Default search:** Any unmatched command falls through to Google search
- **Profile syntax:** `@username` pattern (see Twitter, Instagram, Threads commands)
- **Subreddit syntax:** `r/subreddit` pattern (see Reddit command)

## Common Tasks Reference

### View all available commands
Navigate to `http://localhost:8000/?cmd=bindings` (or use aliases: `commands`, `list`)

### Add a simple redirect
Edit existing command or create new one with static URL return

### Add search functionality
Use `build_search_url()` helper from `url_encoding.rs`

### Add profile lookup
Parse args for `@` prefix (see `instagram.rs`, `twitter.rs`, `threads.rs`)

### Support special syntax
Add parsing logic in `process_args()` (see `reddit.rs` for `r/` pattern)

## Tips for Efficient Development

1. **Use the Explore agent** when you need to understand existing patterns or find similar commands
2. **Read existing commands** for patterns before creating new ones (Instagram, Meta, YouTube are good examples)
3. **Always add tests** - the project has comprehensive test coverage
4. **Follow the existing patterns** - consistency is valued over creativity here
5. **Don't modify registration** when adding subcommands to existing commands
6. **Use parallel tool calls** when reading multiple command files for context
7. **Check `url_encoding.rs`** before writing custom URL builders

## Recent Changes

- 3-12-26: **Major refactor** - Merged binaries, added cross-platform service installation
  - Unified `bunnylol-server` and `bunnylol-cli` into single `bunnylol` binary
  - Server now runs with `bunnylol serve` subcommand
  - Added cross-platform service installation (systemd/launchd/Windows Service)
  - New service management commands: `install-server`, `server start/stop/status/logs`, etc.
  - Moved server code to `src/server/` module
  - Added `ServerConfig` to centralize server configuration
- 3-12-26: Added `meta pay`, `ig reels`, `ig messages/msg/chat` subcommands
- See git log for full history: `git log --oneline`

## Troubleshooting

**Tests failing?**
- Check URL formatting (trailing slashes, query params)
- Verify match arm order (specific before general)
- Ensure test name doesn't conflict with existing tests

**Command not working?**
- Verify registration in `process_command()` match statement
- Check command is exported in `mod.rs`
- Ensure BINDINGS array is correct
- Test with `cargo test your_command`

**Build errors?**
- Run `cargo check` for fast feedback
- Check imports at top of file
- Verify trait implementation is complete

## Reference Commands

**Best examples to study:**
- `src/commands/instagram.rs` - Profile lookup, search, subcommands
- `src/commands/meta.rs` - Multiple subcommands, special binding behavior
- `src/commands/youtube.rs` - Complex subcommand routing
- `src/commands/github.rs` - Path parsing for usernames/repos
- `src/commands/reddit.rs` - Subreddit syntax parsing

---

*This guide is intended for AI assistants working on this codebase. Last updated: 3-12-26*
