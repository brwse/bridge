use std::path::Path;

use openapiv3::OpenAPI;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenApiError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

pub async fn load_spec(path: &str) -> Result<OpenAPI, OpenApiError> {
    let path = Path::new(path);
    let contents = tokio::fs::read_to_string(path).await?;

    let spec = match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&contents)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)?,
        Some(ext) => return Err(OpenApiError::UnsupportedFormat(ext.to_string())),
        None => {
            // Try JSON first, then YAML
            serde_json::from_str(&contents).or_else(|_| serde_yaml::from_str(&contents))?
        }
    };

    Ok(spec)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn test_load_valid_json_spec() {
        let spec_content = r#"
        {
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {}
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_load_valid_yaml_spec() {
        let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let result = load_spec("/non/existent/file.json").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OpenApiError::FileReadError(_)));
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let spec_content = r#"
        {
            "openapi": "3.0.0",
            "info": {
                "title": "Test API"
                "version": "1.0.0"  // Missing comma - invalid JSON
            },
            "paths": {}
        }
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OpenApiError::JsonParseError(_)));
    }

    #[tokio::test]
    async fn test_load_invalid_yaml() {
        let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OpenApiError::YamlParseError(_)));
    }

    #[tokio::test]
    async fn test_load_unsupported_extension() {
        let spec_content = r#"
        {
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {}
        }
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".xml").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OpenApiError::UnsupportedFormat(_)));
    }

    #[tokio::test]
    async fn test_load_no_extension_try_json_first() {
        let spec_content = r#"
        {
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {}
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Test API");
    }

    #[tokio::test]
    async fn test_load_no_extension_fallback_to_yaml() {
        let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Test API");
    }

    #[tokio::test]
    async fn test_load_incomplete_spec() {
        let spec_content = r#"
        {
            "openapi": "3.0.0"
        }
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        // This should fail because required fields are missing
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_spec_with_complex_structure() {
        let spec_content = r#"
        {
            "openapi": "3.0.0",
            "info": {
                "title": "Complex API",
                "version": "2.0.0",
                "description": "A complex API for testing"
            },
            "servers": [
                {
                    "url": "https://api.example.com/v1",
                    "description": "Production server"
                }
            ],
            "paths": {
                "/users": {
                    "get": {
                        "summary": "List users",
                        "operationId": "listUsers",
                        "responses": {
                            "200": {
                                "description": "Successful response"
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "integer"
                            },
                            "name": {
                                "type": "string"
                            }
                        }
                    }
                }
            }
        }
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{spec_content}").unwrap();

        let result = load_spec(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Complex API");
        assert_eq!(spec.info.version, "2.0.0");
        assert!(!spec.paths.paths.is_empty());
        assert!(spec.components.is_some());
    }
}
