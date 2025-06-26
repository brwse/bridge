use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use derive_builder::Builder;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tonic::transport::Channel;

use crate::protobuf::registry::v1::{
    RefreshTokenRequest, RegisterBridgeRequest,
    bridge_registry_service_client::BridgeRegistryServiceClient,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status: {0}")]
    Status(#[from] Box<tonic::Status>),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("No token available")]
    NoToken,
    #[error("Invalid token format from server")]
    InvalidTokenFormat,
    #[error("Builder error: {0}")]
    Builder(String),
}

impl From<derive_builder::UninitializedFieldError> for Error {
    fn from(e: derive_builder::UninitializedFieldError) -> Self {
        Error::Builder(e.to_string())
    }
}

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error::Status(Box::new(status))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Builder, Clone)]
#[builder(build_fn(name = "build_internal", private, error = "self::Error"), pattern = "owned")]
pub struct Client {
    #[builder(setter(skip), default = "Client::default_grpc_client()")]
    grpc_client: BridgeRegistryServiceClient<Channel>,

    #[builder(setter(into))]
    endpoint: String,

    #[builder(setter(name = "decoding_key_arc"))]
    decoding_key: Arc<DecodingKey>,

    #[builder(default = "Duration::seconds(30)")]
    refresh_leeway: Duration,

    #[builder(setter(skip), default = "Arc::new(RwLock::new(None))")]
    token: Arc<RwLock<Option<Token>>>,
}

impl Client {
    fn default_grpc_client() -> BridgeRegistryServiceClient<Channel> {
        // This is a dummy client that will be replaced in the builder's build function.
        // It's needed to satisfy `derive_builder`'s requirement for a default value.
        let channel = Channel::balance_list(
            [tonic::transport::Endpoint::from_static("http://[::1]:50051")].into_iter(),
        );
        BridgeRegistryServiceClient::new(channel)
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub async fn register(&self, br_token: &str) -> Result<()> {
        let request = tonic::Request::new(RegisterBridgeRequest { br_token: br_token.to_owned() });
        let mut client = self.grpc_client.clone();
        let response = client.register_bridge(request).await?.into_inner();

        let expires_at = response
            .expires_at
            .and_then(|ts| DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
            .ok_or(Error::InvalidTokenFormat)?;

        let token = Token {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at,
        };
        *self.token.write().unwrap() = Some(token);
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        let current_token = self.token.read().unwrap().clone().ok_or(Error::NoToken)?;

        let request = tonic::Request::new(RefreshTokenRequest {
            access_token: current_token.access_token.clone(),
            refresh_token: current_token.refresh_token.clone(),
        });

        let mut client = self.grpc_client.clone();
        let response = client.refresh_token(request).await?.into_inner();

        let expires_at = response
            .expires_at
            .and_then(|ts| DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
            .ok_or(Error::InvalidTokenFormat)?;

        let new_token = Token {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at,
        };
        *self.token.write().unwrap() = Some(new_token);

        Ok(())
    }

    pub fn validate_token(&self, token_str: &str) -> Result<Claims> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token_str, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    pub fn get_token(&self) -> Option<Token> {
        self.token.read().unwrap().clone()
    }

    pub fn spawn_refresh_task(&self) -> JoinHandle<()> {
        let client = self.clone();
        tokio::spawn(async move {
            loop {
                let (should_refresh, expires_at) = if let Some(token) = client.get_token() {
                    (token.expires_at <= Utc::now() + client.refresh_leeway, token.expires_at)
                } else {
                    (false, Utc::now())
                };

                if should_refresh {
                    if let Err(e) = client.refresh().await {
                        tracing::error!("Failed to refresh token: {}", e);
                    }
                }

                let sleep_duration = if expires_at > Utc::now() {
                    let refresh_check_point = expires_at - client.refresh_leeway;
                    let now = Utc::now();
                    if refresh_check_point > now {
                        (refresh_check_point - now)
                            .to_std()
                            .unwrap_or(std::time::Duration::from_secs(60))
                    } else {
                        std::time::Duration::from_secs(10) // expired or close to expiry, check sooner
                    }
                } else {
                    // No token or expired, check every minute
                    std::time::Duration::from_secs(60)
                };

                tokio::time::sleep(sleep_duration).await;
            }
        })
    }
}

impl ClientBuilder {
    pub fn decoding_key(mut self, value: DecodingKey) -> Self {
        self.decoding_key = Some(Arc::new(value));
        self
    }

    pub async fn build(self) -> Result<Client> {
        let mut client = self.build_internal()?;
        let grpc_client = BridgeRegistryServiceClient::connect(client.endpoint.clone()).await?;
        client.grpc_client = grpc_client;
        Ok(client)
    }
}
