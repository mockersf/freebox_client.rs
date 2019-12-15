mod free_client;
mod unauthorized_client;

use std::time::{SystemTime, UNIX_EPOCH};

fn ts_nano() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    (since_the_epoch.as_secs() as u128 * 1_000 + since_the_epoch.subsec_millis() as u128)
        * 1_000_000
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_id = String::from("gjdjvkhgf");

    let client = free_client::FreeClient::new(&app_id)?;

    let ts = ts_nano();

    let connection_status = client.get_connection_status()?;
    let xdsl_status = client.get_xdsl_connection_status()?;
    // dbg!(client.get_lan_interfaces()?);
    let lan_hosts = client.get_hosts_on_lan("pub")?;

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
