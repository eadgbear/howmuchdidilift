use interface::*;

use base64::Engine as _;
use eyre::{Result, WrapErr};
use gloo_net::http::{Request, RequestBuilder};
use leptos::server_fn::serde;

#[derive(Clone, Copy)]
pub struct UnauthorizedApi {
    url: &'static str,
}

#[derive(Clone)]
pub struct AuthorizedApi {
    pub url: &'static str,
    pub token: String,
}

impl UnauthorizedApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }

    pub async fn register(&self, params: &RegisterParams) -> Result<()> {
        let url = format!("{}/auth/register", self.url);
        let response = Request::post(&url).json(params)?.send().await?;
        response
            .json::<()>()
            .await
            .wrap_err(format!("Registration call failed"))
    }

    pub async fn login(&self, params: &LoginParams) -> Result<AuthorizedApi> {
        let url = format!("{}/auth/login", self.url);
        let response = Request::post(&url).json(params)?.send().await?;
        let login_response: LoginResponse = response.json().await?;
        Ok(AuthorizedApi::new(self.url, login_response.token))
    }

    /// Fetch a share-card PNG (always a fresh random measure) and its matching
    /// share sentence (from the `x-share-text` response header), in one request.
    /// Returns a `data:` URL usable directly as an `<img>` src, plus the text.
    pub async fn fetch_card(
        &self,
        amt: &str,
        unit_lower: &str,
    ) -> Result<(String, Option<String>)> {
        let url = format!("{}/measures/card?amt={amt}&unit={unit_lower}", self.url);
        let response = Request::get(&url).send().await?;
        let share = response
            .headers()
            .get("x-share-text")
            .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok());
        let bytes = response.binary().await?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok((format!("data:image/png;base64,{b64}"), share))
    }
}

impl AuthorizedApi {
    pub fn new(url: &'static str, token: String) -> Self {
        Self { url, token }
    }

    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn with_auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("Authorization", &self.auth_header_value())
    }

    async fn send<T>(&self, mut req: RequestBuilder) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = req
            .header("Authorization", &self.auth_header_value())
            .send()
            .await?;
        response.json::<T>().await.map_err(|e| e.into())
    }

    pub async fn current_user(&self) -> Result<CurrentResponse> {
        let url = format!("{}/user/current", self.url);
        self.with_auth(Request::get(&url))
            .send()
            .await?
            .json::<CurrentResponse>()
            .await
            .wrap_err(format!("Failed to fetch current user"))
    }

    pub async fn add(&self, params: MeasureCreate) -> Result<Measure> {
        let url = format!("{}/measures", self.url);
        self.with_auth(Request::post(&url))
            .json(&params)?
            .send()
            .await?
            .json::<Measure>()
            .await
            .map_err(|e| e.into())
    }

    pub async fn list(&self) -> Result<Vec<Measure>> {
        let url = format!("{}/measures", self.url);
        self.send(Request::get(&url)).await
    }

    /// Search the server-side emoji index (only matches come back).
    pub async fn search_emoji(&self, query: &str) -> Result<Vec<EmojiEntry>> {
        let url = format!(
            "{}/emoji?limit=60&q={}",
            self.url,
            query.replace(' ', "%20")
        );
        self.send(Request::get(&url)).await
    }

    pub async fn get_one(&self, id: i32) -> Result<Measure> {
        let url = format!("{}/measures/{}", self.url, id);
        self.send(Request::get(&url)).await
    }

    pub async fn delete_one(&self, id: i32) -> Result<()> {
        let url = format!("{}/measures/{}", self.url, id);
        self.send(Request::delete(&url)).await
    }

    pub async fn update_one(&self, measure: Measure) -> Result<Measure> {
        let url = format!("{}/measures/{}", self.url, measure.id);
        self.with_auth(Request::post(&url))
            .json(&measure)?
            .send()
            .await?
            .json::<Measure>()
            .await
            .wrap_err(format!("Failed to update measure"))
    }
}
