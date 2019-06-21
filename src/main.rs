#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use actix::prelude::*;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;

mod models;

use crate::models::DbExecutor;

fn main() -> std::io::Result<()> {
    dotenv().ok();
    let key = "RUST_LOG";
    match std::env::var(key) {
        Ok(val) => println!("using {}={:?}", key, val),
        Err(_) => {
            std::env::set_var(
                key,
                "armchair_enthusiasts=debug,actix_web=info,actix_server=info",
            );
            println!("setting default RUST_LOG values");
        }
    }
    env_logger::init();
    let sys = actix_rt::System::new("main");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let address: Addr<DbExecutor> = SyncArbiter::start(4, move || DbExecutor(pool.clone()));

    let endpoint = "0.0.0.0:28000";
    info!("Starting server at: {:?}", endpoint);

    HttpServer::new(move || {
        App::new()
            .data(address.clone())
            .wrap(Logger::default())
            .service(web::scope("/").service(web::resource("/auth").route(web::get().to(|| {}))))
    })
    .bind(endpoint)?
    .start();

    sys.run()
}