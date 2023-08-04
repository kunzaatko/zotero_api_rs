use derive_builder::Builder;
use log::{info, warn};
use reqwest::{header, Client, Method, Request, Response};

// TODO: When implementing async, this should be split into structs that hold the data for sending
// the requests and the structs that hold the necessary request data. Then the process of creating
// requests can be separate <01-08-23>

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionMode {
    User(String),
    Group(String),
}

impl InteractionMode {
    pub fn prefix(&self) -> String {
        match self {
            Self::User(id) => format!("users/{id}").to_owned(),
            Self::Group(id) => format!("groups/{id}").to_owned(),
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct ZoteroAPI {
    #[builder(try_setter, setter(into), default = "self.default_endpoint()")]
    endpoint: url::Url,
    #[builder(setter(into), default = "3")]
    version: u8,
    #[builder(setter(into))]
    api_key: String,
    #[builder(setter(strip_option), default = "None")]
    mode: Option<InteractionMode>,
    #[builder(setter(strip_option), default = "Some(reqwest::Client::new())")]
    client: Option<reqwest::Client>,
}

impl ZoteroAPIBuilder {
    const BASE_URL: &str = "https://api.zotero.org";

    fn default_endpoint(&self) -> url::Url {
        Self::BASE_URL.parse().expect("BASE_URL is valid")
    }
}

impl ZoteroAPI {
    fn headers(&self) -> header::HeaderMap {
        todo!()
    }

    async fn execute_request(&self, req: Request) -> Result<Response, reqwest::Error> {
        if let Some(client) = &self.client {
            client.execute(req).await
        } else {
            warn!("Not suplying a client may impact performance");
            info!("Creating a one-off client");

            Client::execute(&Client::new(), req).await
        }
    }

    pub async fn fetch_userid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.execute_request(self.get_key_info()).await?;
        let user_id = &response.json::<serde_json::Value>().await?["userID"];
        self.mode = Some(InteractionMode::User(user_id.to_string()));
        Ok(())
    }

    /// Request info about the API key. The response includes the `userID` field, which is the ID associated with the
    /// API key. See also [`Self.fetch_userid`]
    /// **WARNING** exposes the API key in the URL, so OAuth should be preferred where possible.
    pub fn get_key_info(&self) -> Request {
        Request::new(
            Method::GET,
            self.endpoint
                .join(&format!("keys/{}", self.api_key))
                .expect("Valid URL join"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zotero_api_builder() {
        let mut builder = ZoteroAPIBuilder::default();

        // NOTE:  API key field is required
        assert!(builder.build().is_err());

        // NOTE: API key should be the only required field
        assert!(ZoteroAPIBuilder::default().api_key("12345").build().is_ok());

        assert!(builder.try_endpoint("invalid url address").is_err());
        assert!(builder.try_endpoint("https://api.test_zotero.org").is_ok());
        builder.api_key("12345");

        let client = builder.build();
        assert!(client.is_ok());

        let client = client.unwrap();

        assert_eq!(client.endpoint.as_str(), "https://api.test_zotero.org/");
    }

    const API_KEY: &str = std::env!("ZOTERO_API_KEY_TEST");
    const USER_ID: &str = std::env!("ZOTERO_USER_ID_TEST");

    fn test_api() -> ZoteroAPI {
        ZoteroAPIBuilder::default()
            .api_key(API_KEY)
            .build()
            .unwrap()
    }

    #[test]
    fn get_key_info() {
        let api = test_api();

        let keys_request = api.get_key_info();
        assert_eq!(
            keys_request.url().as_str(),
            format!("https://api.zotero.org/keys/{}", API_KEY),
        );
    }

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn fetch_userid() {
        let mut api = test_api();

        aw!(api.fetch_userid()).unwrap();

        assert!(api.mode.is_some());
        assert_eq!(api.mode, Some(InteractionMode::User(USER_ID.to_owned())));
    }
}
