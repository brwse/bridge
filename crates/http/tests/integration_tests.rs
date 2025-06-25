use std::{io::Write, sync::Arc};

use brwse_bridge_http::{mcp::HttpMcpService, openapi};
use insta::assert_json_snapshot;
use serde_json::json;
use tempfile::NamedTempFile;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path, query_param},
};

#[tokio::test]
async fn test_complete_openapi_to_mcp_workflow() {
    // Step 1: Create a comprehensive OpenAPI spec
    let openapi_spec = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0",
            "description": "A comprehensive test API"
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
                    "operationId": "listUsers",
                    "summary": "List all users",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "schema": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 10
                            }
                        },
                        {
                            "name": "offset",
                            "in": "query",
                            "schema": {
                                "type": "integer",
                                "minimum": 0,
                                "default": 0
                            }
                        },
                        {
                            "name": "Authorization",
                            "in": "header",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of users"
                        }
                    }
                },
                "post": {
                    "operationId": "createUser",
                    "summary": "Create a new user",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "name": {
                                            "type": "string",
                                            "minLength": 1
                                        },
                                        "email": {
                                            "type": "string",
                                            "format": "email"
                                        },
                                        "age": {
                                            "type": "integer",
                                            "minimum": 0,
                                            "maximum": 150
                                        }
                                    },
                                    "required": ["name", "email"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "User created successfully"
                        }
                    }
                }
            },
            "/users/{id}": {
                "get": {
                    "operationId": "getUserById",
                    "summary": "Get user by ID",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "integer",
                                "minimum": 1
                            }
                        },
                        {
                            "name": "include",
                            "in": "query",
                            "schema": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": ["profile", "settings", "preferences"]
                                }
                            },
                            "style": "form",
                            "explode": false
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "User details"
                        }
                    }
                }
            }
        }
    }
    "#;

    // Step 2: Save OpenAPI spec to temp file and load it
    let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
    write!(temp_file, "{openapi_spec}").unwrap();
    let spec = openapi::load_spec(temp_file.path().to_str().unwrap()).await.unwrap();

    // Step 3: Set up mock server to simulate the actual API
    let mock_server = MockServer::start().await;

    // Mock for GET /users
    Mock::given(method("GET"))
        .and(path("/users"))
        .and(header("Authorization", "Bearer test-token"))
        .and(query_param("limit", "5"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com"},
                {"id": 2, "name": "Bob", "email": "bob@example.com"}
            ],
            "total": 2,
            "limit": 5,
            "offset": 0
        })))
        .mount(&mock_server)
        .await;

    // Mock for POST /users
    Mock::given(method("POST"))
        .and(path("/users"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 3,
            "name": "Charlie",
            "email": "charlie@example.com",
            "age": 30
        })))
        .mount(&mock_server)
        .await;

    // Mock for GET /users/{id}
    Mock::given(method("GET"))
        .and(path("/users/1"))
        .and(query_param("include", "profile,settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com",
            "profile": {"bio": "Software engineer"},
            "settings": {"theme": "dark", "notifications": true}
        })))
        .mount(&mock_server)
        .await;

    // Step 4: Create HTTP MCP server
    let client = Arc::new(reqwest::Client::new());
    let spec = Arc::new(spec);
    let service = HttpMcpService::new(Arc::clone(&spec), mock_server.uri(), Arc::clone(&client));

    // Step 5: Test tool listing
    let tools = service.tools(None).collect::<Vec<_>>();

    assert_json_snapshot!(tools, @r###"
    [
      {
        "name": "listUsers",
        "description": "List all users",
        "inputSchema": {
          "properties": {
            "headers": {
              "properties": {
                "Authorization": {
                  "type": "string"
                }
              },
              "required": [
                "Authorization"
              ],
              "type": "object"
            },
            "limit": {
              "default": 10,
              "maximum": 100,
              "minimum": 1,
              "type": "integer"
            },
            "offset": {
              "default": 0,
              "minimum": 0,
              "type": "integer"
            }
          },
          "required": [
            "headers"
          ],
          "type": "object"
        }
      },
      {
        "name": "createUser",
        "description": "Create a new user",
        "inputSchema": {
          "properties": {
            "body": {
              "properties": {
                "age": {
                  "maximum": 150,
                  "minimum": 0,
                  "type": "integer"
                },
                "email": {
                  "format": "email",
                  "type": "string"
                },
                "name": {
                  "minLength": 1,
                  "type": "string"
                }
              },
              "required": [
                "name",
                "email"
              ],
              "type": "object"
            }
          },
          "required": [
            "body"
          ],
          "type": "object"
        }
      },
      {
        "name": "getUserById",
        "description": "Get user by ID",
        "inputSchema": {
          "properties": {
            "id": {
              "minimum": 1,
              "type": "integer"
            },
            "include": {
              "items": {
                "type": "string"
              },
              "type": "array"
            }
          },
          "required": [
            "id"
          ],
          "type": "object"
        }
      }
    ]
    "###);

    // Step 6: Test listUsers tool execution
    let list_result = service
        .execute_tool(
            "listUsers",
            json!({
                "limit": 5,
                "offset": 0,
                "headers": {
                    "Authorization": "Bearer test-token"
                }
            }),
        )
        .await
        .unwrap();

    assert_json_snapshot!(list_result, @r###"
    {
      "content": [
        {
          "type": "text",
          "text": "{\"limit\":5,\"offset\":0,\"total\":2,\"users\":[{\"email\":\"alice@example.com\",\"id\":1,\"name\":\"Alice\"},{\"email\":\"bob@example.com\",\"id\":2,\"name\":\"Bob\"}]}"
        }
      ],
      "isError": false
    }
    "###);
}
