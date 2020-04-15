use std::time::Duration;

use actix_web::{web, App, HttpServer, Responder};
use clap::Clap;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use tokio::time::delay_for;

use freebox::free_client;

async fn wifi_status(data: web::Data<AppState>) -> impl Responder {
    format!(
        "wifi enabled: {}!",
        &data.free_client.get_wifi_status().await.unwrap().enabled
    )
}

async fn restart_wifi(data: web::Data<AppState>) -> impl Responder {
    &data.free_client.update_wifi_status(false).await.unwrap();
    delay_for(Duration::from_secs(5)).await;
    &data.free_client.update_wifi_status(true).await.unwrap();
    String::from("done")
}

#[derive(Clap)]
#[clap(version = "1.0", author = "François")]
struct Opts {
    /// conf path
    #[clap(short = "c", long = "config", default_value = "free.conf")]
    config: String,
}

#[derive(Debug, Deserialize)]
struct ConfigurationOnlyApp {
    app_id: String,
}

struct AppState {
    free_client: free_client::FreeClient,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    let conf: Option<ConfigurationOnlyApp> = hocon::HoconLoader::new()
        .load_file(&opts.config)
        .and_then(|hc| hc.resolve())
        .ok();

    let app_id = conf
        .map(|conf| conf.app_id)
        .unwrap_or_else(|| thread_rng().sample_iter(&Alphanumeric).take(15).collect());

    let free_client = free_client::FreeClient::new(&app_id, &opts.config)
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                free_client: free_client.clone(),
            })
            .route("/wifi", web::get().to(wifi_status))
            .route("/restart_wifi", web::post().to(restart_wifi))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
