use serde::{Deserialize, Serialize};
use crate::core::clients::client::Client;

// TODO: implement it

#[allow(dead_code)]
pub enum Version {
    V1_16_5,
    V1_12_2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomClient {
    pub client: Client,
    pub filename: String,
}

#[allow(dead_code)]
impl CustomClient {
    pub fn new(client: Client, filename: String) -> Self {
        CustomClient { client, filename }
    }
}
