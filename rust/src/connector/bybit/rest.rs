use chrono::Utc;
use serde::Deserialize;

use crate::connector::{
    bybit::msg::{Position, RestResponse},
    util::sign_hmac_sha256,
};

#[derive(Clone)]
pub struct BybitClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
    secret: String,
}

impl BybitClient {
    pub fn new(url: &str, api_key: &str, secret: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
            api_key: api_key.to_string(),
            secret: secret.to_string(),
        }
    }

    async fn get<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        query: &str,
        api_key: &str,
        secret: &str,
    ) -> Result<T, reqwest::Error> {
        let time = Utc::now().timestamp_millis() - 1000;
        let sign_body = format!("{time}{api_key}5000{query}");
        let signature = sign_hmac_sha256(secret, &sign_body);
        let resp = self
            .client
            .get(&format!("{}{}?{}", self.url, path, query))
            .header("Accept", "application/json")
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-API-KEY", api_key)
            .header("X-BAPI-TIMESTAMP", time)
            .header("X-BAPI-RECV-WINDOW", "5000")
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    async fn post<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: String,
        api_key: &str,
        secret: &str,
    ) -> Result<T, reqwest::Error> {
        let time = Utc::now().timestamp_millis() - 1000;
        let sign_body = format!("{time}{api_key}5000{body}");
        let signature = sign_hmac_sha256(secret, &sign_body);
        let resp = self
            .client
            .post(&format!("{}{}", self.url, path))
            .header("Accept", "application/json")
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-API-KEY", api_key)
            .header("X-BAPI-TIMESTAMP", time)
            .header("X-BAPI-RECV-WINDOW", "5000")
            .body(body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn cancel_all_orders(&self) -> Result<(), anyhow::Error> {
        let resp: RestResponse = self
            .post(
                "/v5/order/cancel-all",
                "{\"category\":\"linear\"}".to_string(),
                &self.api_key,
                &self.secret,
            )
            .await?;
        if resp.result.success != "1" {
            return Err(anyhow::Error::msg(resp.ret_msg));
        }
        Ok(())
    }

    pub async fn get_position_information(&self) -> Result<Vec<Position>, anyhow::Error> {
        // Position
        let resp: RestResponse = self
            .get(
                "/v5/position/list",
                "category=linear",
                &self.api_key,
                &self.secret,
            )
            .await?;
        if resp.ret_code != 0 {
            return Err(anyhow::Error::msg(resp.ret_msg));
        } else {
            let position: Vec<Position> = serde_json::from_value(resp.result.list)?;
            Ok(position)
        }
    }
}
