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

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevicesResponse {
    pub result: Vec<Device>,
    pub t: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub active_time: i64,
    pub create_time: i64,
    pub id: String,
    pub name: String,
    pub online: bool,
    pub status: Vec<Status>,
    pub sub: bool,
    pub time_zone: String,
    pub update_time: i64,
    pub device_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub code: String,
    pub value: Value,
}

#[derive(Debug)]
pub struct API {
    pub devices: Vec<Device>,
    pub token: Token,
    pub time_since_update: Instant,
    pub time_since_token_renewal: Instant,
    pub polling_interval: Duration,
    api_key: String,
    api_secret: String,
    reqwest_client: reqwest::Client,
}


impl API {
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

    pub fn print_room_temperatures(&self) {
        for device in &self.devices {
            for status in &device.status {
                if status.code == "va_temperature" || status.code == "temp_current" {
                    debug!("{}: {}", device.name, status.value);
                }
            }
        }
    }

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
