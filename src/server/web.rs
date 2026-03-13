/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use leptos::*;
use rocket::request::FlashMessage;
use serde::{Deserialize, Serialize};

use crate::{BunnylolCommandInfo, BunnylolCommandRegistry, BunnylolConfig};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct LandingPageState {
    pub active_tab: String,
    pub alias_notice: Option<AliasNotice>,
}

impl LandingPageState {
    pub fn new(tab: Option<&str>, flash: Option<FlashMessage<'_>>) -> Self {
        let alias_notice = flash.map(|flash| AliasNotice {
            kind: match flash.kind() {
                "error" => AliasNoticeKind::Error,
                "deleted" => AliasNoticeKind::Deleted,
                _ => AliasNoticeKind::Success,
            },
            message: flash.message().to_string(),
        });
        let active_tab = match tab {
            Some("aliases") => "aliases".to_string(),
            _ if alias_notice.is_some() => "aliases".to_string(),
            _ => "commands".to_string(),
        };

        Self {
            active_tab,
            alias_notice,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AliasNotice {
    pub kind: AliasNoticeKind,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq)]
pub enum AliasNoticeKind {
    Success,
    Deleted,
    Error,
}

/// Render the landing page HTML with the given config
pub fn render_landing_page_html(config: &BunnylolConfig, page_state: &LandingPageState) -> String {
    let display_url = config.server.get_display_url();
    let aliases = config.aliases.clone();
    let initial_tab = page_state.active_tab.clone();
    let page_state = page_state.clone();
    let body_content = leptos::ssr::render_to_string(move || {
        view! {
            <LandingPage
                server_display_url=display_url.clone()
                aliases=aliases.clone()
                page_state=page_state.clone()
            />
        }
    })
    .to_string();

    // Wrap in proper HTML document with favicon
    format!(
        r#"<!DOCTYPE html>
                    <html lang="en">
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <title>bunnylol</title>
                        <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🐰</text></svg>">
                        <link rel="preconnect" href="https://fonts.googleapis.com">
                        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                        <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
                        <style>
                            :root {{
                                --gradient-start: #87CEEB;
                                --gradient-end: #6D28D9;
                                --accent-blue: #008ECD;
                                --accent-purple: #532ED1;
                                --text-gray: #9CA3AF;
                                --text-dark: #333;
                                --text-medium: #666;
                                --text-light: #888;
                                --bg-white: white;
                                --bg-light-gray: #f5f7fa;
                                --bg-gradient-gray: #c3cfe2;
                                --border-light: #e0e0e0;
                            }}
                            * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                            body {{
                                font-family: 'JetBrains Mono', monospace;
                                background: linear-gradient(135deg, var(--gradient-start) 0%, var(--gradient-end) 100%);
                                background-attachment: fixed;
                                min-height: 100vh;
                                padding: 20px;
                            }}
                            .binding-card {{
                                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
                                cursor: pointer;
                            }}
                            .binding-card:hover {{
                                transform: translateY(-5px);
                                box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
                            }}
                            .tab-button {{
                                border: 1px solid var(--border-light);
                                background: var(--bg-white);
                                color: var(--text-medium);
                                border-radius: 999px;
                                padding: 10px 16px;
                                font-family: 'JetBrains Mono', monospace;
                                font-size: 0.95rem;
                                cursor: pointer;
                                transition: all 0.2s ease;
                            }}
                            .tab-button.active {{
                                background: linear-gradient(135deg, var(--accent-blue) 0%, var(--accent-purple) 100%);
                                color: white;
                                border-color: transparent;
                                box-shadow: 0 10px 20px rgba(83, 46, 209, 0.18);
                            }}
                            .tab-panel[hidden] {{
                                display: none !important;
                            }}
                            .help-button {{
                                width: 40px;
                                height: 40px;
                                border-radius: 999px;
                                border: 1px solid var(--border-light);
                                background: var(--bg-white);
                                color: var(--text-medium);
                                font-family: 'JetBrains Mono', monospace;
                                font-size: 1.2rem;
                                font-weight: 700;
                                cursor: pointer;
                                box-shadow: 0 10px 24px rgba(0, 0, 0, 0.08);
                                transition: all 0.2s ease;
                            }}
                            .help-button.active {{
                                border-color: transparent;
                                background: linear-gradient(135deg, var(--accent-blue) 0%, var(--accent-purple) 100%);
                                color: white;
                                box-shadow: 0 10px 24px rgba(83, 46, 209, 0.22);
                            }}
                            .help-panel[hidden] {{
                                display: none !important;
                            }}
                        </style>
                    </head>
                    <body data-initial-tab="{}">
                        {}
                        <script>
                            (() => {{
                                const initialTab = document.body.dataset.initialTab || 'commands';
                                const buttons = Array.from(document.querySelectorAll('[data-tab-button]'));
                                const panels = Array.from(document.querySelectorAll('[data-tab-panel]'));
                                const showTab = (tabName) => {{
                                    buttons.forEach((button) => {{
                                        const isActive = button.dataset.tabButton === tabName;
                                        button.classList.toggle('active', isActive);
                                        button.setAttribute('aria-selected', isActive ? 'true' : 'false');
                                    }});
                                    panels.forEach((panel) => {{
                                        panel.hidden = panel.dataset.tabPanel !== tabName;
                                    }});
                                }};
                                buttons.forEach((button) => {{
                                    button.addEventListener('click', () => showTab(button.dataset.tabButton));
                                }});
                                showTab(initialTab);

                                const helpButton = document.querySelector('[data-help-button]');
                                const helpPanel = document.querySelector('[data-help-panel]');
                                if (helpButton && helpPanel) {{
                                    helpButton.addEventListener('click', () => {{
                                        const isOpen = helpButton.getAttribute('aria-expanded') === 'true';
                                        const nextState = isOpen ? 'false' : 'true';
                                        helpButton.setAttribute('aria-expanded', nextState);
                                        helpButton.classList.toggle('active', !isOpen);
                                        helpPanel.hidden = isOpen;
                                    }});
                                }}

                                const aliasNotice = document.querySelector('[data-alias-notice]');
                                if (aliasNotice) {{
                                    window.setTimeout(() => {{
                                        aliasNotice.remove();
                                    }}, 5000);
                                }}
                            }})();
                        </script>
                    </body>
                </html>"#,
        initial_tab, body_content
    )
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingData {
    pub command: String,
    pub description: String,
    pub example: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AliasData {
    pub alias: String,
    pub target: String,
}

impl From<BunnylolCommandInfo> for BindingData {
    fn from(info: BunnylolCommandInfo) -> Self {
        Self {
            command: info
                .bindings
                .first()
                .unwrap_or(&"(default)".to_string())
                .clone(),
            description: info.description,
            example: info.example,
        }
    }
}

#[component]
fn BindingCard(binding: BindingData) -> impl IntoView {
    view! {
        <div
            class="binding-card"
            style:background="linear-gradient(135deg, var(--bg-light-gray) 0%, var(--bg-gradient-gray) 100%)"
            style:border-radius="8px"
            style:padding="20px"
            style:transition="transform 0.2s, box-shadow 0.2s"
            style:border="2px solid var(--border-light)"
        >
            <div
                style:font-family="'JetBrains Mono', monospace"
                style:font-size="1.4em"
                style:font-weight="700"
                style:color="var(--accent-blue)"
                style:margin-bottom="10px"
                style:background="var(--bg-white)"
                style:padding="8px 12px"
                style:border-radius="4px"
                style:display="inline-block"
            >
                {binding.command}
            </div>
            <div
                style:color="var(--text-dark)"
                style:margin-bottom="15px"
                style:line-height="1.5"
            >
                {binding.description}
            </div>
            <div
                style:background="var(--bg-white)"
                style:padding="10px"
                style:border-radius="4px"
                style:border-left="3px solid var(--accent-blue)"
            >
                <div
                    style:font-size="0.85em"
                    style:color="var(--text-medium)"
                    style:margin-bottom="5px"
                    style:font-weight="600"
                >
                    "Example:"
                </div>
                <div
                    style:font-family="'JetBrains Mono', monospace"
                    style:color="var(--accent-purple)"
                    style:font-weight="500"
                >
                    {binding.example}
                </div>
            </div>
        </div>
    }
}

#[component]
fn AliasCard(alias: AliasData) -> impl IntoView {
    view! {
        <div
            class="binding-card"
            style:background="linear-gradient(135deg, #fff9e8 0%, #ffe8cc 100%)"
            style:border-radius="8px"
            style:padding="20px"
            style:transition="transform 0.2s, box-shadow 0.2s"
            style:border="2px solid #ffd8a8"
            style:position="relative"
        >
            <form
                action="/aliases/delete"
                method="post"
                style:position="absolute"
                style:top="12px"
                style:right="12px"
            >
                <input type="hidden" name="alias" value=alias.alias.clone() />
                <button
                    type="submit"
                    aria-label={format!("Delete alias {}", alias.alias)}
                    title="Delete alias"
                    style:width="34px"
                    style:height="34px"
                    style:border-radius="999px"
                    style:border="1px solid #f1c3bb"
                    style:background="rgba(255, 255, 255, 0.92)"
                    style:color="#b44c39"
                    style:display="flex"
                    style:align-items="center"
                    style:justify-content="center"
                    style:cursor="pointer"
                    style:box-shadow="0 6px 14px rgba(180, 76, 57, 0.10)"
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <path d="M3 6h18"></path>
                        <path d="M8 6V4h8v2"></path>
                        <path d="M19 6l-1 14H6L5 6"></path>
                        <path d="M10 11v6"></path>
                        <path d="M14 11v6"></path>
                    </svg>
                </button>
            </form>
            <div
                style:font-family="'JetBrains Mono', monospace"
                style:font-size="1.3em"
                style:font-weight="700"
                style:color="var(--accent-purple)"
                style:margin-bottom="12px"
                style:padding-right="40px"
            >
                {alias.alias}
            </div>
            <div
                style:font-size="0.85em"
                style:color="var(--text-medium)"
                style:margin-bottom="8px"
                style:font-weight="600"
            >
                "Resolves to"
            </div>
            <div
                style:font-family="'JetBrains Mono', monospace"
                style:background="var(--bg-white)"
                style:padding="12px"
                style:border-radius="4px"
                style:color="var(--text-dark)"
                style:border="1px solid #ffd8a8"
                style:line-height="1.5"
                style:word-break="break-word"
            >
                {alias.target}
            </div>
        </div>
    }
}

#[component]
pub fn LandingPage(
    server_display_url: String,
    aliases: std::collections::HashMap<String, String>,
    page_state: LandingPageState,
) -> impl IntoView {
    let mut bindings: Vec<BindingData> = BunnylolCommandRegistry::get_all_commands()
        .iter()
        .map(|cmd| (*cmd).clone().into())
        .collect();
    let mut alias_entries: Vec<AliasData> = aliases
        .into_iter()
        .map(|(alias, target)| AliasData { alias, target })
        .collect();

    // Sort bindings alphabetically by command name
    bindings.sort_by(|a, b| a.command.to_lowercase().cmp(&b.command.to_lowercase()));
    alias_entries.sort_by(|a, b| a.alias.to_lowercase().cmp(&b.alias.to_lowercase()));
    let binding_count = bindings.len();
    let alias_count = alias_entries.len();
    let has_aliases = alias_count > 0;
    let alias_entries = store_value(alias_entries);
    let active_tab = page_state.active_tab.clone();
    let alias_notice = page_state.alias_notice.clone();

    // Clone server_display_url for use in the view
    let example_url = format!("{}/?cmd=gh facebook/bunnylol.rs", server_display_url);

    view! {
        <div
            style:max-width="1200px"
            style:margin="0 auto 10px auto"
            style:background="var(--bg-white)"
            style:border-radius="12px"
            style:padding="20px 30px 30px 30px"
            style:box-shadow="0 20px 60px rgba(0, 0, 0, 0.3)"
            style:font-family="'JetBrains Mono', monospace"
            style:position="relative"
        >
            <div
                style:position="absolute"
                style:top="18px"
                style:left="18px"
                style:z-index="2"
            >
                <button
                    class="help-button"
                    type="button"
                    data-help-button
                    aria-expanded="false"
                    aria-label="Show setup help"
                    title="Show setup help"
                >
                    "?"
                </button>
            </div>
            <h1
                style:color="var(--text-dark)"
                style:text-align="center"
                style:margin-bottom="2px"
                style:margin-top="5px"
                style:font-size="3em"
                style:font-weight="700"
            >
                "bunnylol"
            </h1>
            <div
                style:text-align="center"
                style:margin-bottom="20px"
            >
                <a
                    href="https://github.com/facebook/bunnylol.rs"
                    target="_blank"
                    rel="noopener noreferrer"
                    style:color="var(--accent-blue)"
                    style:text-decoration="none"
                    style:font-size="0.95em"
                    style:font-weight="500"
                    style:display="inline-flex"
                    style:align-items="center"
                    style:gap="6px"
                    style:transition="all 0.2s"
                >
                    // GitHub icon SVG
                    <svg
                        width="20"
                        height="20"
                        viewBox="0 0 16 16"
                        fill="currentColor"
                        style:display="inline-block"
                    >
                        <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>
                    </svg>
                    <span>
                        <span style:color="var(--accent-purple)" style:font-weight="600">"facebook"</span>
                        <span style:color="var(--text-dark)" style:padding-left="2px" style:padding-right="2px">"/"</span>
                        <span style:color="var(--accent-blue)" style:font-weight="600">"bunnylol.rs"</span>
                    </span>
                </a>
            </div>

            <div
                data-help-panel
                class="help-panel"
                hidden=true
                style:background="var(--bg-light-gray)"
                style:padding="20px"
                style:border-radius="6px"
                style:margin-bottom="20px"
                style:border="1px solid var(--border-light)"
            >
                <div style:max-width="700px" style:margin="0 auto" style:color="var(--text-medium)" style:line-height="1.6" style:text-align="center">
                    <p style:margin-bottom="10px">
                        "This server is available at "
                        <code
                            style:font-family="'JetBrains Mono', monospace"
                            style:background="var(--bg-white)"
                            style:padding="4px 8px"
                            style:border-radius="4px"
                            style:color="var(--text-dark)"
                            style:border="1px solid var(--border-light)"
                            style:font-size="0.9em"
                        >
                            {server_display_url.clone()}
                        </code>
                        ", so try:"
                    </p>
                    <a
                        href={example_url.clone()}
                        target="_blank"
                        rel="noopener noreferrer"
                        style:font-family="'JetBrains Mono', monospace"
                        style:background="var(--bg-white)"
                        style:padding="12px 16px"
                        style:border-radius="4px"
                        style:display="inline-block"
                        style:color="var(--accent-blue)"
                        style:border="1px solid var(--accent-blue)"
                        style:text-decoration="none"
                        style:transition="all 0.2s"
                        style:font-size="0.9em"
                    >{example_url.clone()}</a>

                    // Setup guides within web usage section
                    <div style:margin-top="15px">
                        <div style:font-weight="600" style:margin-bottom="15px" style:color="var(--text-dark)" style:font-size="1em" style:text-align="center">
                            "Set bunnylol as your default search engine!"
                        </div>
                        <p style:margin-bottom="15px" style:text-align="center" style:color="var(--text-medium)" style:line-height="1.8" style:max-width="800" style:margin-left="auto" style:margin-right="auto">
                            "Once configured, just enter "
                            <code
                                style:font-family="'JetBrains Mono', monospace"
                                style:background="var(--bg-white)"
                                style:padding="4px 8px"
                                style:border-radius="4px"
                                style:color="var(--text-dark)"
                                style:border="1px solid var(--border-light)"
                                style:font-size="0.9em"
                                style:white-space="nowrap"
                            >
                                "gh facebook/bunnylol.rs"
                            </code>
                            " in your address bar to get the same result."
                        </p>
                        <p style:margin-bottom="15px" style:text-align="center" style:color="var(--text-medium)" style:line-height="1.8" style:max-width="800" style:margin-left="auto" style:margin-right="auto">
                            "Use this URL as your search engine: "
                            <code
                                style:font-family="'JetBrains Mono', monospace"
                                style:background="var(--bg-white)"
                                style:padding="4px 8px"
                                style:border-radius="4px"
                                style:color="var(--text-dark)"
                                style:border="1px solid var(--border-light)"
                                style:font-size="0.9em"
                                style:white-space="nowrap"
                            >
                                {format!("{}/?cmd=%s", server_display_url)}
                            </code>
                        </p>
                        <div style:color="var(--text-medium)" style:line-height="1.8" style:max-width="600px" style:margin="0 auto">
                            <div style:display="grid" style:grid-template-columns="repeat(auto-fit, minmax(200px, 1fr))" style:gap="10px" style:margin-bottom="15px">
                                <div style:text-align="center">
                                    "🖥️ "
                                    <a
                                        href="https://support.google.com/chrome/answer/95426?hl=en&co=GENIE.Platform%3DDesktop"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        style:color="var(--accent-blue)"
                                        style:text-decoration="none"
                                        style:font-weight="500"
                                    >
                                        "Desktop Chrome"
                                    </a>
                                </div>
                                <div style:text-align="center">
                                    "🦊 "
                                    <a
                                        href="https://support.mozilla.org/en-US/kb/add-custom-search-engine-firefox"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        style:color="var(--accent-blue)"
                                        style:text-decoration="none"
                                        style:font-weight="500"
                                    >
                                        "Desktop Firefox"
                                    </a>
                                </div>
                                <div style:text-align="center">
                                    "📱 "
                                    <a
                                        href="https://support.mozilla.org/en-US/kb/change-your-default-search-engine-firefox-ios"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        style:color="var(--accent-blue)"
                                        style:text-decoration="none"
                                        style:font-weight="500"
                                    >
                                        "iOS Firefox"
                                    </a>
                                </div>
                                <div style:text-align="center">
                                    "📱 "
                                    <a
                                        href="https://support.mozilla.org/en-US/kb/manage-my-default-search-engines-firefox-android"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        style:color="var(--accent-blue)"
                                        style:text-decoration="none"
                                        style:font-weight="500"
                                    >
                                        "Android Firefox"
                                    </a>
                                </div>
                            </div>
                            <p style:font-size="0.85em" style:margin-top="10px" style:color="var(--text-light)" style:font-style="italic" style:text-align="center">
                                "Note: iOS Safari does not support custom search engines."
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <div
                style:text-align="center"
                style:color="var(--text-medium)"
                style:margin-bottom="20px"
                style:font-size="1.1em"
                style:font-weight="600"
            >
                "Available Shortcuts"
            </div>

            <div
                style:display="flex"
                style:justify-content="center"
                style:gap="12px"
                style:margin-bottom="24px"
                style:flex-wrap="wrap"
            >
                <button
                    class=if active_tab == "commands" { "tab-button active" } else { "tab-button" }
                    type="button"
                    data-tab-button="commands"
                    aria-selected=if active_tab == "commands" { "true" } else { "false" }
                >
                    {format!("Commands ({})", binding_count)}
                </button>
                <button
                    class=if active_tab == "aliases" { "tab-button active" } else { "tab-button" }
                    type="button"
                    data-tab-button="aliases"
                    aria-selected=if active_tab == "aliases" { "true" } else { "false" }
                >
                    {format!("Aliases ({})", alias_count)}
                </button>
            </div>

            <div
                data-tab-panel="commands"
                class="tab-panel"
                style:display="grid"
                style:grid-template-columns="repeat(auto-fill, minmax(350px, 1fr))"
                style:gap="20px"
                style:margin-top="30px"
            >
                <For
                    each=move || bindings.clone()
                    key=|binding| binding.command.clone()
                    children=|binding| view! { <BindingCard binding=binding /> }
                />
            </div>

            <div
                data-tab-panel="aliases"
                class="tab-panel"
                hidden=active_tab != "aliases"
            >
                {alias_notice.map(|notice| {
                    let (background, border, title) = match notice.kind {
                        AliasNoticeKind::Success => ("#eefbf3", "#8bd8a8", "Saved"),
                        AliasNoticeKind::Deleted => ("#fff3f1", "#f2b3a8", "Deleted"),
                        AliasNoticeKind::Error => ("#fff3f1", "#f2b3a8", "Could not save"),
                    };

                    view! {
                        <div
                            data-alias-notice
                            style:background=background
                            style:border={format!("1px solid {}", border)}
                            style:border-radius="10px"
                            style:padding="16px 18px"
                            style:margin-bottom="20px"
                            style:color="var(--text-dark)"
                        >
                            <div
                                style:font-size="0.9em"
                                style:font-weight="700"
                                style:margin-bottom="6px"
                            >
                                {title}
                            </div>
                            <div style:line-height="1.6">
                                {notice.message}
                            </div>
                        </div>
                    }
                })}

                <div
                    style:background="linear-gradient(135deg, #fff9e8 0%, #fff4d6 100%)"
                    style:border="1px solid #ffd8a8"
                    style:border-radius="10px"
                    style:padding="22px"
                    style:margin-bottom="22px"
                >
                    <div
                        style:font-size="1.05em"
                        style:font-weight="700"
                        style:color="var(--text-dark)"
                        style:margin-bottom="8px"
                    >
                        "Add an alias"
                    </div>
                    <div
                        style:color="var(--text-medium)"
                        style:line-height="1.7"
                        style:margin-bottom="16px"
                    >
                        "Aliases become available immediately in the running server and are also written to your config file."
                    </div>
                    <form
                        action="/aliases"
                        method="post"
                        style:display="grid"
                        style:grid-template-columns="repeat(auto-fit, minmax(220px, 1fr))"
                        style:gap="14px"
                        style:align-items="end"
                    >
                        <label style:display="block">
                            <div
                                style:font-size="0.85em"
                                style:font-weight="600"
                                style:color="var(--text-medium)"
                                style:margin-bottom="6px"
                            >
                                "Alias"
                            </div>
                            <input
                                type="text"
                                name="alias"
                                required
                                autocomplete="off"
                                placeholder="work"
                                style:width="100%"
                                style:padding="12px"
                                style:border="1px solid #e2c790"
                                style:border-radius="8px"
                                style:font-family="'JetBrains Mono', monospace"
                                style:background="var(--bg-white)"
                            />
                        </label>
                        <label style:display="block">
                            <div
                                style:font-size="0.85em"
                                style:font-weight="600"
                                style:color="var(--text-medium)"
                                style:margin-bottom="6px"
                            >
                                "Resolves to"
                            </div>
                            <input
                                type="text"
                                name="target"
                                required
                                autocomplete="off"
                                placeholder="gh facebook/react"
                                style:width="100%"
                                style:padding="12px"
                                style:border="1px solid #e2c790"
                                style:border-radius="8px"
                                style:font-family="'JetBrains Mono', monospace"
                                style:background="var(--bg-white)"
                            />
                        </label>
                        <button
                            type="submit"
                            style:border="none"
                            style:border-radius="999px"
                            style:padding="12px 18px"
                            style:font-family="'JetBrains Mono', monospace"
                            style:font-weight="700"
                            style:cursor="pointer"
                            style:background="linear-gradient(135deg, var(--accent-blue) 0%, var(--accent-purple) 100%)"
                            style:color="white"
                            style:box-shadow="0 10px 20px rgba(83, 46, 209, 0.18)"
                        >
                            "Save alias"
                        </button>
                    </form>
                </div>

                <Show
                    when=move || has_aliases
                    fallback=|| view! {
                        <div
                            style:background="linear-gradient(135deg, #fff9e8 0%, #fff4d6 100%)"
                            style:border="1px solid #ffd8a8"
                            style:border-radius="10px"
                            style:padding="28px"
                            style:text-align="center"
                            style:color="var(--text-medium)"
                            style:line-height="1.7"
                        >
                            <div
                                style:font-size="1.1em"
                                style:font-weight="700"
                                style:color="var(--text-dark)"
                                style:margin-bottom="8px"
                            >
                                "No aliases configured"
                            </div>
                            <div>
                                "Use the form above to add your first alias."
                            </div>
                        </div>
                    }
                >
                    <div
                        style:display="grid"
                        style:grid-template-columns="repeat(auto-fill, minmax(320px, 1fr))"
                        style:gap="20px"
                    >
                        <For
                            each=move || alias_entries.get_value()
                            key=|alias| alias.alias.clone()
                            children=|alias| view! { <AliasCard alias=alias /> }
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_landing_page_includes_aliases() {
        let mut config = BunnylolConfig::default();
        config
            .aliases
            .insert("work".to_string(), "gh mycompany/repo".to_string());

        let html = render_landing_page_html(
            &config,
            &LandingPageState {
                active_tab: "aliases".to_string(),
                alias_notice: Some(AliasNotice {
                    kind: AliasNoticeKind::Success,
                    message: "Alias saved.".to_string(),
                }),
            },
        );

        assert!(html.contains("Aliases (1)"));
        assert!(html.contains("work"));
        assert!(html.contains("gh mycompany"));
        assert!(html.contains("Save alias"));
        assert!(html.contains("/aliases/delete"));
        assert!(html.contains("Alias saved."));
    }
}
