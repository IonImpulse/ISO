#![allow(non_snake_case)]
use std::cmp::min;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::process::exit;
use std::sync::Arc;
use std::thread;

use actix_cors::*;
use actix_web::rt::spawn;
use actix_web::*;
use actix_web_static_files::ResourceFiles;
use actix_web_middleware_redirect_scheme::RedirectSchemeBuilder;

use lazy_static::__Deref;
use lazy_static::lazy_static;

use log::*;

use openssl::ssl::SslAcceptor;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

mod data;
mod post;
mod user;
mod routes;

use data::*;
use post::*;
use user::*;
use routes::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

lazy_static! {
    pub static ref MEMORY_DATABASE: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::default()));
    pub static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));
}

// Debug vs release address
const ADDRESS: &str = "127.0.0.1:8080";
const ADDRESS_HTTPS: &str = "0.0.0.0:443";

const DB_NAME: &str = "db.json";
const LOGGER_STR: &str = "\nMAKE Log @ %t\nIP: %a (%{r}a)\nRequest: \"%r\"\nAgent: \"%{Referer}i\" \"%{User-Agent}i\"\nResponse: STATUS %s for %b bytes in %D ms";
const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

fn main() {
    std::env::set_var("RUST_LOG", "info, actix_web=trace");
    env_logger::init();

    ctrlc::set_handler(move || {
        info!("Exiting...");
        thread::sleep(Duration::from_secs(2));
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let _ = actix_web::rt::System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .thread_name("main-tokio")
            .build()
            .unwrap()
    })
    .block_on(async_main());
}


async fn async_main() -> std::io::Result<()> {
    // Print startup text
    info!("Starting up...");
    println!("██████████████████████████████████████████████████████████████");
    println!("Version {}", VERSION_STRING);
    println!("██████████████████████████████████████████████████████████████");

    let mut file = OpenOptions::new()
        .read(true)
        .open("config.toml")
        .expect("Failed to open config.toml");
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let data: Config = toml::from_str(&data).expect("Failed to parse api_keys.toml");

    let mut config = CONFIG.lock().await;
    *config = data;
    drop(config);

    // Load all databases
    let data = load_database().unwrap();
    let mut lock = MEMORY_DATABASE.lock().await;
    *lock = data;
    drop(lock);

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let _ = save_database().await;
        }
    });

    info!("Database loaded.");

    if cfg!(debug_assertions) {
        // Create builder without ssl
        return HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_header()
                .allow_any_method()
                .send_wildcard()
                .max_age(3600);

            App::new()
                .wrap(actix_web::middleware::Logger::new(LOGGER_STR))
                .wrap(actix_web::middleware::Compress::default())
                .wrap(cors)
                .service(get_post_page)
                .service(get_user_info)
                .service(new_post)
                .service(start_verification)
                .service(check_verification)
                .service(claim_post)
                .service(get_individual_post)
                .service(ResourceFiles::new("/", generate()))

        })
        .bind(ADDRESS)?
        .run()
        .await;
    } else {

        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(
                "/etc/letsencrypt/live/isoapp.dev/privkey.pem",
                SslFiletype::PEM,
            )
            .unwrap();
        builder
            .set_certificate_chain_file("/etc/letsencrypt/live/isoapp.dev/fullchain.pem")
            .unwrap();

        // Create builder without ssl
        return HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_header()
                .allow_any_method()
                .send_wildcard()
                .max_age(3600);

            App::new()
                .wrap(actix_web::middleware::Logger::new(LOGGER_STR))
                .wrap(actix_web::middleware::Compress::default())
                .wrap(cors)
                .service(get_post_page)
                .service(get_user_info)
                .service(new_post)
                .service(start_verification)
                .service(check_verification)
                .service(claim_post)
                .service(get_individual_post)
                .service(ResourceFiles::new("/", generate()))


        })
        .bind_openssl(ADDRESS_HTTPS, builder)?
        .run()
        .await;
    }
}
