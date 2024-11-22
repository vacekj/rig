use crate::{
    agent::AgentBuilder,
    embeddings::{self},
    extractor::ExtractorBuilder,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{completion::CompletionModel, embedding::EmbeddingModel, EMBEDDING_V1};

// ================================================================
// xAI Client
// ================================================================
const XAI_BASE_URL: &str = "https://api.x.ai";

#[derive(Clone)]
pub struct Client {
    base_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, XAI_BASE_URL)
    }
    fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::CONTENT_TYPE,
                        "application/json".parse().unwrap(),
                    );
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", api_key)
                            .parse()
                            .expect("Bearer token should parse"),
                    );
                    headers
                })
                .build()
                .expect("xAI reqwest client should build"),
        }
    }

    /// Create a new xAI client from the `XAI_API_KEY` environment variable.
    /// Panics if the environment variable is not set.
    pub fn from_env() -> Self {
        let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY not set");
        Self::new(&api_key)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");

        tracing::debug!("POST {}", url);
        self.http_client.post(url)
    }

    /// Create an embedding model with the given name.
    /// Note: default embedding dimension of 0 will be used if model is not known.
    /// If this is the case, it's better to use function `embedding_model_with_ndims`
    ///
    /// # Example
    /// ```
    /// use rig::providers::xai::{Client, self};
    ///
    /// // Initialize the xAI client
    /// let xai = Client::new("your-xai-api-key");
    ///
    /// let embedding_model = xai.embedding_model(xai::embedding::EMBEDDING_V1);
    /// ```
    pub fn embedding_model(&self, model: &str) -> EmbeddingModel {
        let ndims = match model {
            EMBEDDING_V1 => 3072,
            _ => 0,
        };
        EmbeddingModel::new(self.clone(), model, ndims)
    }

    /// Create an embedding model with the given name and the number of dimensions in the embedding
    ///  generated by the model.
    ///
    /// # Example
    /// ```
    /// use rig::providers::xai::{Client, self};
    ///
    /// // Initialize the xAI client
    /// let xai = Client::new("your-xai-api-key");
    ///
    /// let embedding_model = xai.embedding_model_with_ndims("model-unknown-to-rig", 1024);
    /// ```
    pub fn embedding_model_with_ndims(&self, model: &str, ndims: usize) -> EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, ndims)
    }

    /// Create an embedding builder with the given embedding model.
    ///
    /// # Example
    /// ```
    /// use rig::providers::xai::{Client, self};
    ///
    /// // Initialize the xAI client
    /// let xai = Client::new("your-xai-api-key");
    ///
    /// let embeddings = xai.embeddings(xai::embedding::EMBEDDING_V1)
    ///     .simple_document("doc0", "Hello, world!")
    ///     .simple_document("doc1", "Goodbye, world!")
    ///     .build()
    ///     .await
    ///     .expect("Failed to embed documents");
    /// ```
    pub fn embeddings(&self, model: &str) -> embeddings::EmbeddingsBuilder<EmbeddingModel> {
        embeddings::EmbeddingsBuilder::new(self.embedding_model(model))
    }

    /// Create a completion model with the given name.
    pub fn completion_model(&self, model: &str) -> CompletionModel {
        CompletionModel::new(self.clone(), model)
    }

    /// Create an agent builder with the given completion model.
    /// # Example
    /// ```
    /// use rig::providers::xai::{Client, self};
    ///
    /// // Initialize the xAI client
    /// let xai = Client::new("your-xai-api-key");
    ///
    /// let agent = xai.agent(xai::completion::GROK_BETA)
    ///    .preamble("You are comedian AI with a mission to make people laugh.")
    ///    .temperature(0.0)
    ///    .build();
    /// ```
    pub fn agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    /// Create an extractor builder with the given completion model.
    pub fn extractor<T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync>(
        &self,
        model: &str,
    ) -> ExtractorBuilder<T, CompletionModel> {
        ExtractorBuilder::new(self.completion_model(model))
    }
}

pub mod xai_api_types {
    use serde::Deserialize;

    impl ApiErrorResponse {
        pub fn message(&self) -> String {
            format!("Code `{}`: {}", self.code, self.error)
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct ApiErrorResponse {
        pub error: String,
        pub code: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum ApiResponse<T> {
        Ok(T),
        Error(ApiErrorResponse),
    }
}
