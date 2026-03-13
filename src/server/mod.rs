/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

// Server runtime (routes, web UI) - only needed for server feature
#[cfg(feature = "server")]
pub mod web;

// Service management - only needed for CLI feature
#[cfg(feature = "cli")]
pub mod service;

// Server runtime code below only compiled with server feature
#[cfg(feature = "server")]
use rocket::State;
#[cfg(feature = "server")]
use rocket::form::{Form, FromForm};
#[cfg(feature = "server")]
use rocket::request::FlashMessage;
#[cfg(feature = "server")]
use rocket::request::{self, FromRequest, Request};
#[cfg(feature = "server")]
use rocket::response::{Flash, Redirect};
#[cfg(feature = "server")]
use std::sync::RwLock;

#[cfg(feature = "server")]
use crate::{BunnylolCommandRegistry, BunnylolConfig, History, utils};

#[cfg(feature = "server")]
mod server_impl {
    use super::*;

    pub(super) struct AppState {
        pub config: RwLock<BunnylolConfig>,
    }

    #[derive(FromForm)]
    pub(super) struct AliasForm {
        pub alias: String,
        pub target: String,
    }

    #[derive(FromForm)]
    pub(super) struct DeleteAliasForm {
        pub alias: String,
    }

    fn alias_redirect(status: &str, message: &str) -> Flash<Redirect> {
        Flash::new(Redirect::to("/"), status, message)
    }

    // Request guard to extract client IP address
    pub(super) struct ClientIP(pub String);

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for ClientIP {
        type Error = ();

        async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            let ip = req
                .client_ip()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            request::Outcome::Success(ClientIP(ip))
        }
    }

    // http://localhost:8000/?cmd=gh
    #[rocket::get("/?<cmd>&<tab>")]
    pub(super) fn search(
        cmd: Option<&str>,
        tab: Option<&str>,
        flash: Option<FlashMessage<'_>>,
        state: &State<AppState>,
        client_ip: ClientIP,
    ) -> Result<Redirect, rocket::response::content::RawHtml<String>> {
        let config = state
            .config
            .read()
            .expect("config state should not be poisoned")
            .clone();

        match cmd {
            Some(cmd_str) => {
                println!("bunnylol command: {}", cmd_str);
                let resolved = config.resolve_command(cmd_str);
                let command = utils::get_command_from_query_string(&resolved);
                let redirect_url = BunnylolCommandRegistry::process_command(command, &resolved);
                println!("redirecting to: {}", redirect_url);

                // Track command in history if enabled
                if config.history.enabled
                    && let Some(history) = History::new(&config)
                    && let Err(e) = history.add(cmd_str, &client_ip.0)
                {
                    eprintln!("Warning: Failed to save command to history: {}", e);
                }

                Ok(Redirect::to(redirect_url))
            }
            None => {
                let page_state = web::LandingPageState::new(tab, flash);
                Err(rocket::response::content::RawHtml(
                    web::render_landing_page_html(&config, &page_state),
                ))
            }
        }
    }

    #[rocket::post("/aliases", data = "<form>")]
    pub(super) fn add_alias(form: Form<AliasForm>, state: &State<AppState>) -> Flash<Redirect> {
        let alias = form.alias.trim();
        let target = form.target.trim();

        if alias.is_empty() {
            return alias_redirect("error", "Alias name is required.");
        }
        if alias.chars().any(char::is_whitespace) {
            return alias_redirect("error", "Alias names cannot contain spaces.");
        }
        if target.is_empty() {
            return alias_redirect("error", "Alias target is required.");
        }

        let mut config = state
            .config
            .write()
            .expect("config state should not be poisoned");
        let previous_value = config.aliases.insert(alias.to_string(), target.to_string());
        let status = if previous_value.is_some() {
            "updated"
        } else {
            "saved"
        };

        if let Err(error) = config.save() {
            match previous_value {
                Some(value) => {
                    config.aliases.insert(alias.to_string(), value);
                }
                None => {
                    config.aliases.remove(alias);
                }
            }
            return alias_redirect("error", &format!("Could not save alias: {error}"));
        }

        alias_redirect(status, &format!("Alias '{alias}' saved."))
    }

    #[rocket::post("/aliases/delete", data = "<form>")]
    pub(super) fn delete_alias(
        form: Form<DeleteAliasForm>,
        state: &State<AppState>,
    ) -> Flash<Redirect> {
        let alias = form.alias.trim();

        if alias.is_empty() {
            return alias_redirect("error", "Alias name is required.");
        }

        let mut config = state
            .config
            .write()
            .expect("config state should not be poisoned");
        let removed_value = match config.aliases.remove(alias) {
            Some(value) => value,
            None => return alias_redirect("error", &format!("Alias '{alias}' was not found.")),
        };

        if let Err(error) = config.save() {
            config.aliases.insert(alias.to_string(), removed_value);
            return alias_redirect("error", &format!("Could not delete alias: {error}"));
        }

        alias_redirect("deleted", &format!("Alias '{alias}' deleted."))
    }

    // Health check endpoint for Docker healthcheck (no verbose logging)
    #[rocket::get("/health")]
    pub(super) fn health() -> &'static str {
        "ok"
    }

    // Catch 404 errors and show landing page
    #[rocket::catch(404)]
    pub(super) fn not_found(req: &rocket::Request) -> rocket::response::content::RawHtml<String> {
        // Get config from request state
        if let Some(state) = req.rocket().state::<AppState>() {
            let config = state
                .config
                .read()
                .expect("config state should not be poisoned")
                .clone();
            rocket::response::content::RawHtml(web::render_landing_page_html(
                &config,
                &web::LandingPageState::default(),
            ))
        } else {
            // Fallback if config is not available (shouldn't happen)
            rocket::response::content::RawHtml(
                "<html><body><h1>404 Not Found</h1></body></html>".to_string(),
            )
        }
    }
}

#[cfg(feature = "server")]
use server_impl::*;

/// Launch the Bunnylol web server with the given configuration
#[cfg(feature = "server")]
pub async fn launch(config: BunnylolConfig) -> Result<(), Box<rocket::Error>> {
    println!(
        "Bunnylol server starting with default search: {}",
        config.default_search
    );
    println!(
        "Server listening on {}:{}",
        config.server.address, config.server.port
    );

    let figment = rocket::Config::figment()
        .merge(("address", config.server.address.clone()))
        .merge(("port", config.server.port))
        .merge(("log_level", config.server.log_level.clone()))
        .merge(("ident", format!("Bunnylol/{}", env!("CARGO_PKG_VERSION"))));
    let state = AppState {
        config: RwLock::new(config),
    };

    let _rocket = rocket::custom(figment)
        .manage(state)
        .mount(
            "/",
            rocket::routes![search, add_alias, delete_alias, health],
        )
        .register("/", rocket::catchers![not_found])
        .launch()
        .await?;
    Ok(())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use std::collections::HashMap;

    use rocket::http::Status;
    use rocket::local::blocking::Client;

    use super::*;

    #[test]
    fn test_search_resolves_aliases() {
        let mut config = BunnylolConfig::default();
        config.history.enabled = false;
        config.aliases = HashMap::from([("work".to_string(), "gh @octocat".to_string())]);

        let state = AppState {
            config: RwLock::new(config),
        };
        let rocket = rocket::build()
            .manage(state)
            .mount("/", rocket::routes![search]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/?cmd=work").dispatch();

        assert_eq!(response.status(), Status::SeeOther);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://github.com/octocat")
        );
    }
}
