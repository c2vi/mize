use chrono::{DateTime, Utc};
use mize::{mize_err, mize_part, Mize, MizeError, MizePart, MizeResult};
use reqwest::{blocking::Client, blocking::RequestBuilder, blocking::Response, header, Method};
use serde_json::{json, Value};
use std::thread::sleep;
use std::time::Duration;

#[mize_part]
#[derive(Default)]
pub struct Habitica {
    mize: Mize,
    client: Client,
}

pub fn habitica(mize: &mut Mize) -> MizeResult<()> {
    let client = Client::new();
    mize.register_part(Box::new(Habitica {
        mize: mize.clone(),
        client,
    }))
}

impl MizePart for Habitica {}

impl Habitica {
    fn api_request(&mut self, method: Method, path: String, data: Value) -> MizeResult<Value> {
        let api_url = self.mize.get_config("habitica.api_url")?.to_string();
        let user_id = self.mize.get_config("habitica.user_id")?.to_string();
        let api_token = self.mize.get_config("habitica.api_token")?.to_string();
        let client_name = self.mize.get_config("habitica.client_name")?.to_string();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert("x-api-user", header::HeaderValue::from_str(&user_id)?);
        headers.insert("x-api-key", header::HeaderValue::from_str(&api_token)?);
        headers.insert("x-client", header::HeaderValue::from_str(&client_name)?);

        let url = format!("{}/{}", api_url, path.trim_start_matches('/'));

        let mut request_builder = self.client.request(method, &url).headers(headers);

        if data != json!({}) {
            request_builder = request_builder.json(&data);
        }

        let response = request_builder.send()?;

        handle_rate_limit(&response);

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            println!("request_url: {}", url);
            return Err(mize_err!(
                "Habitica API error: {} {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                text
            ));
        }

        let json_response: Value = response.json()?;
        Ok(json_response.get("data").cloned().unwrap_or(Value::Null))
    }
}

fn handle_rate_limit(response: &Response) {
    let headers = response.headers();
    let limit = headers
        .get("X-RateLimit-Limit")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("NONE");
    let remaining = headers
        .get("X-RateLimit-Remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(10);
    let reset = headers
        .get("X-RateLimit-Reset")
        .and_then(|v| v.to_str().ok());

    println!(
        "RateLimit: {} | Remaining: {} | Reset: {}",
        limit,
        remaining,
        reset.unwrap_or("N/A")
    );

    if remaining < 2 {
        if let Some(reset_str) = reset {
            if let Ok(reset_date) = DateTime::parse_from_rfc2822(reset_str) {
                let now = Utc::now();
                let wait_duration = reset_date.signed_duration_since(now).to_std();
                if let Ok(wait_duration) = wait_duration {
                    let wait_ms = wait_duration.as_millis() as u64 + 1000;
                    println!(
                        "Waiting {} secs for next rate limit window...",
                        (wait_ms / 1000)
                    );
                    sleep(Duration::from_millis(wait_ms));
                }
            }
        }
    }
}
