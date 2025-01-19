pub mod config;
pub mod dns_utils;
use anyhow::{anyhow, Context, Result};
use clap::{App, Arg};
use config::Config;
use dns_utils::{create_dns_query, parse_dns_answer};
use odoh_rs::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use reqwest::{
    header::{HeaderMap, ACCEPT, CACHE_CONTROL, CONTENT_TYPE},
    Client, Response, StatusCode,
};
use std::env;
use url::Url;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const QUERY_PATH: &str = "/dns-query";
const WELL_KNOWN: &str = "/.well-known/odohconfigs";

#[derive(Clone, Debug)]
struct ClientSession {
    pub client: Client,
    pub target: Url,
    pub proxy: Option<Url>,
    pub client_secret: Option<[u8; 16]>,
    pub target_config: ObliviousDoHConfigContents,
    pub query: Option<ObliviousDoHMessagePlaintext>,
}

impl ClientSession {
    /// Create a new ClientSession
    pub async fn new(config: Config) -> Result<Self> {
        let mut target = Url::parse(&config.server.target)?;
        target.set_path(QUERY_PATH);
        let proxy = if let Some(p) = &config.server.proxy {
            Url::parse(p).ok()
        } else {
            None
        };

        // fetch `odohconfigs` by querying well known endpoint using GET request
        let mut odohconfigs = reqwest::get(&format!("{}{}", config.server.target, WELL_KNOWN))
            .await?
            .bytes()
            .await?;
        let configs: ObliviousDoHConfigs = parse(&mut odohconfigs).context("invalid configs")?;
        let target_config = configs
            .into_iter()
            .next()
            .context("no available config")?
            .into();

        Ok(Self {
            client: Client::new(),
            target,
            proxy,
            client_secret: None,
            target_config,
            query: None,
        })
    }

    /// Create an oblivious query from a domain and query type
    pub fn create_request(&mut self, domain: &str, qtype: &str) -> Result<Vec<u8>> {
        // create a DNS message
        let dns_msg = create_dns_query(domain, qtype)?;
        let query = ObliviousDoHMessagePlaintext::new(&dns_msg, 1);
        self.query = Some(query.clone());
        let mut rng = StdRng::from_entropy();
        let (oblivious_query, client_secret) = encrypt_query(&query, &self.target_config, &mut rng)
            .context("failed to encrypt query")?;
        let query_body = compose(&oblivious_query)
            .context("failed to compose query body")?
            .freeze();
        self.client_secret = Some(client_secret);
        Ok(query_body.to_vec())
    }

    /// Set headers and build an HTTP request to send the oblivious query to the proxy/target.
    /// If a proxy is specified, the request will be sent to the proxy. However, if a proxy is absent,
    /// it will be sent directly to the target. Note that not specifying a proxy effectively nullifies
    /// the entire purpose of using ODoH.
    pub async fn send_request(&mut self, request: &[u8]) -> Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, ODOH_HTTP_HEADER.parse()?);
        headers.insert(ACCEPT, ODOH_HTTP_HEADER.parse()?);
        headers.insert(CACHE_CONTROL, "no-cache, no-store".parse()?);
        let query = [
            (
                "targethost",
                self.target
                    .host_str()
                    .context("Target host is not a valid host string")?,
            ),
            ("targetpath", QUERY_PATH),
        ];
        let builder = if let Some(p) = &self.proxy {
            self.client.post(p.clone()).headers(headers).query(&query)
        } else {
            self.client.post(self.target.clone()).headers(headers)
        };
        let resp = builder.body(request.to_vec()).send().await?;
        Ok(resp)
    }

    /// Parse the received response from the resolver and print the answer.
    pub async fn parse_response(&self, resp: Response) -> Result<()> {
        if resp.status() != StatusCode::OK {
            return Err(anyhow!(
                "query failed with response status code {}",
                resp.status().as_u16()
            ));
        }
        let mut data = resp.bytes().await?;
        let response_body = parse(&mut data).context("failed to parse response body")?;
        let response = decrypt_response(
            &self.query.clone().unwrap(),
            &response_body,
            self.client_secret.clone().unwrap(),
        )
        .context("failed to decrypt response")?;
        parse_dns_answer(&response.into_msg())?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(PKG_NAME)
        .version(PKG_VERSION)
        .author(PKG_AUTHORS)
        .about(PKG_DESCRIPTION)
        .arg(
            Arg::with_name("domain")
                .help("Domain to query")
                .required(true)
                .index(1), // positional argument, must be './odoh-client <domain> ...'
        )
        .arg(
            Arg::with_name("type")
                .help("Query type")
                .default_value("A")
                .index(2), // positional argument, must be './odoh-client <domain> <type>'
        )
        .arg(
            Arg::with_name("config_file")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to the config.toml config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .value_name("URL")
                .help("Target URL for the ODoH server")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("proxy")
                .short("p")
                .long("proxy")
                .value_name("URL")
                .help("Proxy URL for the ODoH server")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches
        .value_of("config_file")
        .unwrap_or("tests/config.toml");
    let mut config = Config::from_path(config_file)?;
    // if target and proxy are specified in the command line, override the config file
    if let Some(target) = matches.value_of("target") {
        config.server.target = target.to_string();
    }
    if let Some(proxy) = matches.value_of("proxy") {
        config.server.proxy = Some(proxy.to_string());
    }
    let domain = matches.value_of("domain").unwrap();
    let qtype = matches.value_of("type").unwrap();
    let mut session = ClientSession::new(config.clone()).await?;
    let request = session.create_request(domain, qtype)?;
    let response = session.send_request(&request).await?;
    session.parse_response(response).await?;
    Ok(())
}
