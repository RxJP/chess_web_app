#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, Options};
use rocket::Route;
use rocket_dyn_templates::Template;
use std::env::args;
use std::fs::create_dir;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;

use crate::models::{SessionDB, UserDB};

mod routing;
mod models;
mod utils;
mod stockfish_interface;

//todo add saving the database to filesystem
#[launch]
fn rocket() -> _ {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Enter the folder path.\nUsage: {} [path to working directory]", args[0]);
        exit(0);
    }
    std::env::set_current_dir(&args[1]).expect("Unable to set working directory");

    if !Path::new("./data").exists() {
        create_dir("./data").expect("Error creating directory");
    }
    if !Path::new(models::SESSION_DB_PATH).exists() {
        File::create(models::SESSION_DB_PATH).expect("Error creating token db file");
    }
    if !Path::new(models::USER_DB_PATH).exists() {
        File::create(models::USER_DB_PATH).expect("Error creating token db file");
    }


    rocket::build()
        .manage(SessionDB::initialize())
        .manage(UserDB::initialize())
        .manage(Arc::new(AtomicU8::new(0)))
        .mount("/", routes![
            routing::index,
            routing::auth,
            routing::login,
            routing::register,
            routing::logout,
            routing::is_username_taken,
            routing::bot_play,
            routing::ws_bot_play,
        ])
        .mount("/", debug_only_route())
        .mount("/", FileServer::new("./static", Options::None))
        .register("/", catchers![routing::not_found, routing::internal_error])
        .attach(Template::custom(|_engines| {
            // routing::customize(&mut engines.tera);
        }))
}

#[cfg(debug_assertions)]
fn debug_only_route() -> Vec<Route> {
    routes![routing::debug_internal_state]
}

#[cfg(not(debug_assertions))]
fn debug_only_route() -> Vec<Route> {
    // Return an empty list in release mode or just omit the route
    routes![]
}