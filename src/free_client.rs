use serde::{Deserialize, Serialize};

pub struct FreeClient {
    http_client: reqwest::Client,
    base_url: String,
    pub api_domain: String,
    app_id: String,
    app_token: String,
}

#[derive(Deserialize, Serialize)]
struct Configuration {
    app_id: String,
    app_token: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Response<T> {
    success: bool,
    pub result: T,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    GoingUp,
    Up,
    GoingDown,
    Down,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Ethernet,
    Rfc2684,
    Pppoatm,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMedia {
    Ftth,
    Xdsl,
}

#[derive(Deserialize, Debug)]
pub struct ConnectionStatus {
    #[serde(rename = "type")]
    pub ty: ConnectionType,
    pub media: ConnectionMedia,
    pub state: ConnectionState,
    pub rate_down: u32,
    pub rate_up: u32,
    pub bytes_down: u64,
    pub bytes_up: u64,
    pub bandwidth_down: u32,
    pub bandwidth_up: u32,
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum XDSLStatusStatus {
    Down,
    Training,
    Started,
    ChanAnalysis,
    MsgExchange,
    Showtime,
    Disabled,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum XDSLStatusProtocol {
    T1413,
    Adsl1A,
    Adsl2A,
    Adsl2plusA,
    Readsl2,
    Adsl2M,
    Adsl2plusM,
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum XDSLStatusModulation {
    Adsl,
    Vdsl,
}

#[derive(Deserialize, Debug)]
pub struct XDSLStatus {
    pub status: XDSLStatusStatus,
    pub protocol: XDSLStatusProtocol,
    pub modulation: XDSLStatusModulation,
    pub uptime: u32,
}

#[derive(Deserialize, Debug)]
pub struct XDSLStats {
    pub maxrate: u32,
    pub rate: u32,
    pub snr: u32,
    pub attn: u32,
    pub snr_10: u32,
    pub attn_10: u32,
    pub fec: u32,
    pub crc: u32,
    pub hec: u32,
    pub es: u32,
    pub ses: u32,
    pub phyr: bool,
    pub ginp: bool,
    pub nitro: bool,
    pub rxmt: Option<u32>,
    pub rxmt_corr: Option<u32>,
    pub rxmt_uncorr: Option<u32>,
    pub rtx_tx: Option<u32>,
    pub rtx_c: Option<u32>,
    pub rtx_uc: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct XDSLConnectionStatus {
    pub status: XDSLStatus,
    pub up: XDSLStats,
    pub down: XDSLStats,
}

#[derive(Deserialize, Debug)]
pub struct LanInterface {
    pub name: String,
    pub host_count: u8,
}

#[derive(Deserialize, Debug)]
pub struct LanHostName {
    pub name: String,
    pub source: String,
}

#[derive(Deserialize, Debug)]
pub struct LanHostL2Ident {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Deserialize, Debug)]
pub struct LanHostL3Connectivity {
    pub addr: String,
    pub af: String,
    pub active: bool,
    pub reachable: bool,
    pub last_activity: u64,
    pub last_time_reachable: u64,
}

#[derive(Deserialize, Debug)]
pub struct LanHost {
    pub id: String,
    pub primary_name: String,
    pub host_type: String,
    pub primary_name_manual: bool,
    pub l2ident: LanHostL2Ident,
    pub vendor_name: String,
    pub persistent: bool,
    pub reachable: bool,
    pub last_time_reachable: u64,
    pub active: bool,
    pub last_activity: u64,
    pub names: Option<Vec<LanHostName>>,
    pub l3connectivities: Option<Vec<LanHostL3Connectivity>>,
}

impl FreeClient {
    pub fn new(app_id: &str) -> anyhow::Result<FreeClient> {
        let http_client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()?;

        let unauth = crate::unauthorized_client::UnauthorizedClient::new(&http_client, app_id);
        let base_url = unauth.get_base_url()?;
        let api_domain = unauth.get_api_domain()?;

        let conf: Option<Configuration> = hocon::HoconLoader::new()
            .load_file("free.conf")
            .and_then(|hc| hc.resolve())
            .ok();
        let app_token = match conf.and_then(|c| c.app_token) {
            Some(app_token) => app_token,
            None => {
                let app_token = unauth.authorize(&base_url)?;
                let new_config = Configuration {
                    app_id: String::from(app_id),
                    app_token: Some(app_token.clone()),
                };
                let f = std::fs::File::create("free.conf")?;
                serde_json::to_writer(f, &new_config)?;
                app_token
            }
        };

        Ok(FreeClient {
            http_client,
            base_url,
            api_domain,
            app_id: String::from(app_id),
            app_token: app_token,
        })
    }

    fn get<T>(&self, url: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let session =
            crate::unauthorized_client::UnauthorizedClient::new(&self.http_client, &self.app_id)
                .get_session(&self.base_url, &self.app_token)?;

        Ok(self
            .http_client
            .get(url)
            .header("X-Fbx-App-Auth", session)
            .send()?
            .json::<Response<T>>()?
            .result)
    }

    #[allow(unused)]
    fn get_text(&self, url: &str) -> anyhow::Result<String> {
        let session =
            crate::unauthorized_client::UnauthorizedClient::new(&self.http_client, &self.app_id)
                .get_session(&self.base_url, &self.app_token)?;

        Ok(self
            .http_client
            .get(url)
            .header("X-Fbx-App-Auth", session)
            .send()?
            .text()?)
    }

    pub fn get_connection_status(&self) -> anyhow::Result<ConnectionStatus> {
        self.get(&format!("{}{}", self.base_url, "connection/"))
    }

    pub fn get_xdsl_connection_status(&self) -> anyhow::Result<XDSLConnectionStatus> {
        self.get(&format!("{}{}", self.base_url, "connection/xdsl/"))
    }

    pub fn get_lan_interfaces(&self) -> anyhow::Result<Vec<LanInterface>> {
        self.get(&format!("{}{}", self.base_url, "lan/browser/interfaces/"))
    }

    pub fn get_hosts_on_lan(&self, interface: &str) -> anyhow::Result<Vec<LanHost>> {
        self.get(&format!(
            "{}{}/{}/",
            self.base_url, "lan/browser", interface
        ))
    }
}
