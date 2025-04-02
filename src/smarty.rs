use std::cell::RefCell;
use color_eyre::eyre::{bail, eyre};
use serde::{Deserialize, Serialize};
use smarty_rust_sdk::sdk::authentication::SecretKeyCredential;
use smarty_rust_sdk::sdk::batch::Batch;
use smarty_rust_sdk::sdk::options::{Options, OptionsBuilder};
use smarty_rust_sdk::us_street_api::client::USStreetAddressClient;
use smarty_rust_sdk::us_street_api::lookup::{Lookup, MatchStrategy};
use crate::atmb::model::Address;
use crate::utils::retry_wrapper;

/// A free trial account is limited to 1000 lookups per month.
/// So we use multiple accounts to avoid the limitation.
///
/// As there are ~1700 atmb location currently, we need at least 2 accounts.
pub struct SmartyClientProxy {
    clients: Vec<SmartyClient>,
    state: RefCell<Vec<ClientState>>,
}

impl SmartyClientProxy {
    pub fn new() -> color_eyre::Result<Self> {
        let credentials = Self::credentials();
        let clients = credentials.into_iter()
            .map(|(id, secret)| SmartyClient::new(id, secret))
            .collect::<Result<Vec<_>, _>>()?;
        let state = clients.iter().map(|_| ClientState::default()).collect();
        Ok(
            Self {
                clients,
                state: RefCell::new(state),
            }
        )
    }

    pub async fn inquire_address(&self, address: Address) -> color_eyre::Result<AdditionalInfo> {
        let client = self.next_client();
        client.inquire_address(address).await
    }

    fn next_client(&self) -> &SmartyClient {
        let idx = self.get_client_id();
        self.update_state(idx);
        &self.clients[idx]
    }

    /// get the index of a client that is not exceeded
    fn get_client_id(&self) -> usize {
        self.state.borrow().iter().enumerate().find(|(_, state)| !state.is_exceeded())
            .map(|(id, _)| id)
            .expect("all clients are exceeded")
    }

    fn update_state(&self, idx: usize) {
        let mut state = self.state.borrow_mut();
        state[idx].lookups += 1;
    }

    /// load authentication credentials from environment variables
    ///
    /// CREDENTIALS=`ID1`=`SECRET1`[,`ID2`=`SECRET2`]*
    fn credentials() -> Vec<(String, String)> {
        std::env::var("CREDENTIALS")
            .map(|credentials| {
                credentials.split(',')
                    .map(|pair| {
                        let mut iter = pair.split('=');
                        (iter.next().unwrap().to_string(), iter.next().unwrap().to_string())
                    })
                    .collect()
            })
            .expect("`CREDENTIALS` environment variable must be set")
    }
}

#[derive(Default)]
struct ClientState {
    lookups: u32,
}

impl ClientState {
    fn is_exceeded(&self) -> bool {
        self.lookups > 1000
    }
}

struct SmartyClient {
    client: USStreetAddressClient,
}

impl SmartyClient {
    fn new(auth_id: impl Into<String>, auth_token: impl Into<String>) -> color_eyre::Result<Self> {
        Ok(
            Self {
                client: USStreetAddressClient::new(Self::options(auth_id, auth_token))?,
            }
        )
    }

    async fn inquire_address(&self, address: Address) -> color_eyre::Result<AdditionalInfo> {
        retry_wrapper(3, || async {
            self._inquire_address(address.clone()).await
        }).await
    }

    async fn _inquire_address(&self, address: Address) -> color_eyre::Result<AdditionalInfo> {
        let mut batch = Batch::default();
        batch.push(Lookup::from(address))?;
        self.client.send(&mut batch).await?;
        let resp = batch.records().into_iter().next()
            .ok_or_else(|| eyre!("no response from Smarty"))?;
        resp.clone().try_into()
    }

    fn authentication(auth_id: impl Into<String>, auth_token: impl Into<String>) -> Box<SecretKeyCredential> {
        SecretKeyCredential::new(
            auth_id.into(),
            auth_token.into(),
        )
    }

    fn options(auth_id: impl Into<String>, auth_token: impl Into<String>) -> Options {
        OptionsBuilder::new(Some(Self::authentication(auth_id, auth_token)))
            .with_license("us-core-cloud")
            .with_retries(3)
            .build()
    }
}

impl From<Address> for Lookup {
    fn from(address: Address) -> Self {
        Self {
            zipcode: address.full_zip(),
            street: address.line1,
            city: address.city,
            state: address.state,
            match_strategy: MatchStrategy::Enhanced,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct AdditionalInfo {
    pub cmra: YesOrNo,
    pub rdi: Rdi,
}

#[derive(Debug, PartialEq, Eq, Serialize, Ord, PartialOrd, Clone)]
#[serde(rename_all = "PascalCase")]
#[repr(u8)]
pub enum Rdi {
    Residential,
    Commercial,
    Unknown,
}

impl TryFrom<String> for Rdi {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "residential" => Ok(Rdi::Residential),
            "commercial" => Ok(Rdi::Commercial),
            "" => Ok(Rdi::Unknown),
            _ => Err(value),
        }
    }
}

impl AdditionalInfo {
    pub fn is_cmra(&self) -> bool {
        self.cmra == YesOrNo::Y
    }
}

impl TryFrom<Lookup> for AdditionalInfo {
    type Error = color_eyre::eyre::Error;

    fn try_from(lookup: Lookup) -> Result<Self, Self::Error> {
        if lookup.results.is_empty() {
            bail!("no results found: {:?}", lookup);
        }
        let candidate = lookup.results
            .into_iter()
            .next()
            .unwrap();

        Ok(
            Self {
                cmra: YesOrNo::try_from(candidate.analysis.dpv_cmra)
                    .map_err(|e| eyre!("failed to parse CMRA: {}", e))?,
                rdi: Rdi::try_from(candidate.metadata.rdi)
                    .map_err(|e| eyre!("failed to parse RDI: {}", e))?,
            }
        )
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum YesOrNo {
    N,
    Y,
}

impl TryFrom<String> for YesOrNo {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "y" => Ok(YesOrNo::Y),
            "n" => Ok(YesOrNo::N),
            _ => Err(value),
        }
    }
}
