use std::time::{SystemTime, UNIX_EPOCH};

use clap::Clap;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;

mod free_client;
mod unauthorized_client;

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

fn ts_nano() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    (since_the_epoch.as_secs() as u128 * 1_000 + since_the_epoch.subsec_millis() as u128)
        * 1_000_000
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let conf: Option<ConfigurationOnlyApp> = hocon::HoconLoader::new()
        .load_file(&opts.config)
        .and_then(|hc| hc.resolve())
        .ok();

    let app_id = conf
        .map(|conf| conf.app_id)
        .unwrap_or_else(|| thread_rng().sample_iter(&Alphanumeric).take(15).collect());

    let client = free_client::FreeClient::new(&app_id, &opts.config).await?;

    let ts = ts_nano();

    let connection_status = client.get_connection_status().await?;
    let xdsl_status = client.get_xdsl_connection_status().await?;
    // dbg!(client.get_lan_interfaces()?);
    let lan_hosts = client.get_hosts_on_lan("pub").await?;

    println!(
        "connection_status,api_domain={} type={:?},media={:?},state={:?} {}",
        client.api_domain,
        connection_status.ty,
        connection_status.media,
        connection_status.state,
        ts
    );
    println!(
        "connection_status,api_domain={} rate_down={}i,rate_up={}i {}",
        client.api_domain, connection_status.rate_down, connection_status.rate_up, ts
    );
    println!(
        "connection_status,api_domain={} bytes_down={}i,bytes_up={}i {}",
        client.api_domain, connection_status.bytes_down, connection_status.bytes_up, ts
    );
    println!(
        "connection_status,api_domain={} bandwidth_down={}i,bandwidth_up={}i {}",
        client.api_domain, connection_status.bandwidth_down, connection_status.bandwidth_up, ts
    );
    println!(
        "connection_status,api_domain={} ipv4={},ipv6={} {}",
        client.api_domain, connection_status.ipv4, connection_status.ipv6, ts
    );

    println!(
        "xdsl_status,api_domain={} status={:?},protocol={:?},modulation={:?},uptime={}i {}",
        client.api_domain,
        xdsl_status.status.status,
        xdsl_status.status.protocol,
        xdsl_status.status.modulation,
        xdsl_status.status.uptime,
        ts
    );
    println!(
        "xdsl_stats,api_domain={},direction=up maxrate={}i,rate={}i {}",
        client.api_domain, xdsl_status.up.maxrate, xdsl_status.up.rate, ts
    );
    println!(
        "xdsl_stats,api_domain={},direction=down maxrate={}i,rate={}i {}",
        client.api_domain, xdsl_status.down.maxrate, xdsl_status.down.rate, ts
    );

    for host in lan_hosts {
        println!(
            "lan_hosts,api_domain={},primary_name={},l2ident={} reachable={},active={} {}",
            client.api_domain,
            if host.primary_name == "" {
                String::from("null")
            } else {
                host.primary_name.replace(' ', "_")
            },
            host.l2ident.id,
            host.reachable,
            host.active,
            ts
        );
    }

    Ok(())
}
