use base64;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

/// A struct representing a danfoss api token
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The access token that needs to be sent with every request to the API
    pub access_token: String,
    /// Type of the access token
    pub token_type: String,
    /// Validity duration of the token in seconds.
    pub expires_in: String,
}

/// A struct representing the response for the /devices/ endpoint
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevicesResponse {
    /// A list of all devices connected to your account
    pub result: Vec<Device>,
    /// An identifier
    pub t: i64,
}

// A struct implementing the [device schema](https://developer.danfoss.com/docs/ally/1/types/device)
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    /// Time when last seen online
    pub active_time: i64,
    /// Time when the device was setup
    pub create_time: i64,
    /// Unique identifier of the device
    pub id: String,
    /// User specified name of the device
    pub name: String,
    /// Online status of the device
    pub online: bool,
    /// Current settings for the device
    pub status: Vec<Status>,
    /// Indicates whether this device is controlled by a gateway. True: yes, false: no
    pub sub: bool,
    /// Time Zone
    pub time_zone: String,
    /// Last update of device setting
    pub update_time: i64,
    /// Type of device
    pub device_type: String,
}
/// Values of a device setting
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// Status code
    pub code: String,
    /// Value of the status code
    pub value: Value,
}

/// Struct that holds all information to interact with the Danfoss ally api
#[derive(Debug)]
pub struct API {
    /// List of devices connected to the account
    pub devices: Vec<Device>,
    /// Access token for the API
    pub token: Token,
    /// Time since the last API call. The free API in general has throttling enabled which apply across the API. 
    /// Throttling kicking in can be identified by receiving status code 429 - too many request. 
    /// E.g. the /token endpoint has a maximum of 5 calls per second.
    pub time_since_update: Instant,
    /// Time since the last access token was fetched
    pub time_since_token_renewal: Instant,
    /// How often the run function should poll data. Default: Every 30 seconds
    pub polling_interval: Duration,
    api_key: String,
    api_secret: String,
    reqwest_client: reqwest::Client,
}

/// API client implementation for Danfoss Ally
/// # Examples
///
/// ```
/// use danfoss_ally_rs::API;
///
/// let mut danfoss_api: API = API::new();
/// danfoss_api.get_token();
/// danfoss_api.get_devices();
///
/// ```
impl API {
    /// Create new danfoss ally client
    pub fn new() -> Self {
        let api_key = env::var("DANFOSS_API_KEY").expect("No Danfoss API key provided");

        let api_secret = env::var("DANFOSS_API_SECRET").expect("No Danfoss API secret provided.");

        Self {
            devices: vec![],
            token: Token {
                access_token: String::new(),
                token_type: String::new(),
                expires_in: "0".to_string(),
            },
            api_key,
            api_secret,
            time_since_update: Instant::now(),
            time_since_token_renewal: Instant::now(),
            reqwest_client: reqwest::Client::new(),
            polling_interval: Duration::new(30,0),
        }
    }

    /// Fetch data in a loop indefinitely.
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            sleep(self.polling_interval);
            if self.time_since_token_renewal.elapsed().as_secs()
                >= self.token.expires_in.parse::<u64>()?
            {
                self.get_token()
                    .await
                    .unwrap_or_else(|e| error!("Could not fetch token. {:?}", e));
                self.time_since_token_renewal = Instant::now();
            }
            self.get_devices()
                .await
                .unwrap_or_else(|e| error!("Could not get devices. {:?}", e));
            self.time_since_update = Instant::now();
            self.print_room_temperatures();
        }
    }

    /// Debug print room temperatures
    pub fn print_room_temperatures(&self) {
        for device in &self.devices {
            for status in &device.status {
                if status.code == "va_temperature" || status.code == "temp_current" {
                    debug!("{}: {}", device.name, status.value);
                }
            }
        }
    }
    /// Fetch access token with the provided credentials
    pub async fn get_token(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let basic_auth: String = base64::encode(format!("{}:{}", self.api_key, self.api_secret));
        let authorization_header: String = format!("Basic {}", basic_auth);

        let params = [("grant_type", "client_credentials")];
        let res = self
            .reqwest_client
            .post("https://api.danfoss.com/oauth2/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("accept", "application/json")
            .header("authorization", authorization_header)
            .form(&params)
            .send()
            .await?;

        self.token = serde_json::from_str(res.text().await?.as_str())?;
        Ok(())
    }
    /// Get all devices and their status
    pub async fn get_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = self
            .reqwest_client
            .get("https://api.danfoss.com/ally/devices")
            .header("accept", "application/json")
            .header(
                "authorization",
                format!("Bearer {}", self.token.access_token),
            )
            .send()
            .await?;
        let devices: DevicesResponse = serde_json::from_str(res.text().await?.as_str())?;
        self.devices = devices.result;
        Ok(())
    }
}
