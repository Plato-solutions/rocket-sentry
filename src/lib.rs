#![allow(clippy::needless_doctest_main)]
//! **rocket-sentry** is a simple add-on for the **Rocket** web framework to simplify
//! integration with the **Sentry** application monitoring system.
//!
//! Or maybe...
//!
//! > "The Rocket Sentry is a static rocket-firing gun platform that is based on a
//! > Personality Construct and used in the Aperture Science Enrichment Center."
//! >
//! > -- [Half-Life wiki](https://half-life.fandom.com/wiki/Rocket_Sentry)
//!
//! Example usage
//! =============
//!
//! ```no_run
//! # #[macro_use]
//! # extern crate rocket;
//! use rocket_sentry::RocketSentry;
//!
//! # fn main() {
//! #[launch]
//! fn rocket() -> _ {
//!     rocket::build()
//!         .attach(RocketSentry::fairing())
//!         // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   add this line
//! }
//! # }
//! ```
//!
//! Then, the Sentry integration can be enabled by adding a `sentry_dsn=` value to
//! the `Rocket.toml` file, for example:
//!
//! ```toml
//! [debug]
//! sentry_dsn = ""  # Disabled
//! [release]
//! sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
//! ```
//!
#[macro_use]
extern crate log;

use std::sync::Mutex;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::serde::Deserialize;
use rocket::{fairing, Build, Rocket};
use logging::ClientInitGuard;
use logging::sentry::{Span, TransactionOrSpan};

pub struct RocketSentry {
    guard: Mutex<Option<ClientInitGuard>>,
}

#[derive(Deserialize)]
struct Config {
    sentry_dsn: String,
}

impl RocketSentry {
    pub fn fairing(sentry:ClientInitGuard) -> impl Fairing {
        RocketSentry {
            guard: Mutex::new(Some(sentry)),
        }
    }

    // fn init(&self) {
    //     let guard = logging::init();
    //
    //     if guard.is_enabled() {
    //         // Tuck the ClientInitGuard in the fairing, so it lives as long as the server.
    //         let mut self_guard = self.guard.lock().unwrap();
    //         *self_guard = Some(guard);
    //
    //         info!("Sentry enabled.");
    //     } else {
    //         error!("Sentry did not initialize.");
    //     }
    // }
}

#[rocket::async_trait]
impl Fairing for RocketSentry {
    fn info(&self) -> Info {
        Info {
            name: "rocket-sentry",
            kind: Kind::Ignite | Kind::Singleton,
        }
    }
    //
    // async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
    //     let figment = rocket.figment();
    //
    //     let config: figment::error::Result<Config> = figment.extract();
    //     match config {
    //         Ok(config) => {
    //             if config.sentry_dsn.is_empty() {
    //                 info!("Sentry disabled.");
    //             } else {
    //                 self.init(&config.sentry_dsn);
    //             }
    //         }
    //         Err(err) => error!("Sentry not configured: {}", err),
    //     }
    //     Ok(rocket)
    // }

    async fn on_request(&self, req: &mut rocket::Request<'_>, data: &mut rocket::Data<'_>) {
        let mut headers = vec![];
        if let Some(sentry_trace) = req.headers().get_one("sentry-trace") {
            headers.push(("sentry-trace",sentry_trace)); }

        let transaction_ctx = logging::sentry::TransactionContext::continue_from_headers(
            "ApiGateway",
            "on_request",
            headers,
        );
        let transaction =  logging::sentry::start_transaction(transaction_ctx);
        logging::sentry::configure_scope(|scope| scope.set_span(Some( transaction.clone().into())));

        //add data to scope?
        //force to beginning of scope?
    }
}
