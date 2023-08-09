use derive_builder::Builder;
use log::{info, warn};
use reqwest::{Client, Method, Request, Response};

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionMode {
    User(String),
    Group(String),
}

impl InteractionMode {
    pub fn prefix(&self) -> String {
        match self {
            Self::User(id) => format!("users/{id}/").to_owned(),
            Self::Group(id) => format!("groups/{id}/").to_owned(),
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
    const HOST: &str = "https://api.zotero.org";

    fn default_endpoint(&self) -> url::Url {
        Self::HOST.parse().expect("BASE_URL is valid")
    }
}

impl ZoteroAPI {
    fn insert_headers(&self, req: &mut Request) -> Result<(), Box<dyn std::error::Error>> {
        let map = req.headers_mut();
        map.reserve(2);
        map.insert(
            "Zotero-API-Version",
            "3".parse().expect("3 is a valid HeaderValue"),
        );
        map.insert(
            "Authorization",
            format!("Bearer {}", &self.api_key)
                .parse()
                .expect("API key is valid"),
        );
        Ok(())
    }

    fn set_url(
        &self,
        req: &mut Request,
        path: &str,
        args: Option<&[(&str, &str)]>,
        query: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = req.url_mut();

        #[cfg(test)]
        assert_eq!(
            url.as_str(),
            self.endpoint.as_str(),
            "Initial URL {} must be `self.endpoint()`",
            url.as_str()
        );

        let usergroup_prefix = self.mode.as_ref().expect("Mode is defined").prefix();
        url.set_path(&usergroup_prefix);
        *url = url.join(path)?;
        if let Some(args) = args {
            *url = reqwest::Url::parse_with_params(url.as_str(), args)?;
        };
        if let Some(query) = query {
            url.set_query(Some(query));
        };

        Ok(())
    }

    // TODO: API requests macro for common methods <09-08-23>
    // TODO: Collections getter with the possibility of nested <09-08-23>
    // TODO: Model feature with methods returning to model structs <09-08-23>

    async fn execute(&self, req: Request) -> Result<Response, reqwest::Error> {
        if let Some(client) = &self.client {
            client.execute(req).await
        } else {
            warn!("Not suplying a client may impact performance");
            info!("Creating a one-off client");

            Client::execute(&Client::new(), req).await
        }
    }

    pub async fn fetch_userid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.execute(self.get_key_info()).await?;
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

    fn test_api_with_mode() -> ZoteroAPI {
        ZoteroAPIBuilder::default()
            .api_key(API_KEY)
            .mode(InteractionMode::User(USER_ID.to_owned()))
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

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn set_nonblank_url() {
        let api = test_api();
        api.set_url(&mut api.get_key_info(), "items", None, None);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn set_nomode_url() {
        let api = test_api();
        let mut req = Request::new(Method::GET, api.endpoint.clone());

        api.set_url(&mut req, "items", None, None);
    }

    #[test]
    fn set_url() {
        let api = test_api_with_mode();
        let mut req = Request::new(Method::GET, api.endpoint.clone());
        let res = api.set_url(&mut req, "items", None, None);

        assert!(res.is_ok());

        assert_eq!(
            req.url().as_str(),
            format!("https://api.zotero.org/users/{}/items", USER_ID)
        );
    }
}
