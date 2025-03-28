
use std::env;
use std::fmt::{Formatter,Debug};

use exports::wasix::mcp::secrets_list;
use wasix::mcp::secrets_store::{self, Host, HostSecret, Secret, SecretsError};
use wasmtime::{Result,component::{bindgen, Resource}};

bindgen!({
    world: "wasix:mcp/secrets",
    with: 
    {
        "wasix:mcp/secrets-list": SecretsListImpl,
    },
    trappable_imports: true,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretsDescription {
    pub name: String,
    pub description: String,
    pub required: bool,
}

impl SecretsListImpl {
    // This method is a stub for the `list-secrets` function in the WIT interface.
    pub fn list_secrets() -> Vec<SecretsDescription> {
        // For demonstration purposes, we'll return a mock list of secrets
        vec![
            SecretsDescription {
                name: "API_KEY".to_string(),
                description: "API Key for accessing the service".to_string(),
                required: true,
            },
            SecretsDescription {
                name: "DB_PASSWORD".to_string(),
                description: "Database password for connections".to_string(),
                required: true,
            },
            SecretsDescription {
                name: "DEBUG_FLAG".to_string(),
                description: "Flag for enabling debug mode".to_string(),
                required: false,
            },
        ]
    }
}

impl secrets_list for SecretsListImpl {
    fn list_secrets(&self) -> Result<Vec<SecretsDescription>> {
        Ok(Self::list_secrets()) // Return the mock list
    }
}

pub struct SecretsListImpl;


/* 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret {
    key: String,
}
*/

impl Debug for SecretValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SecretValue::String(_) => write!(f, "string(redacted)"),
            SecretValue::Bytes(_) => write!(f, "bytes(redacted)"),
        }
    }
}

impl SecretValue {
    pub fn new_string(value: String) -> Self {
        SecretValue::String(value)
    }
    pub fn new_bytes(value: Vec<u8>) -> Self {
        SecretValue::Bytes(value)
    }
}
/*
pub trait SecretsTrait {
    /// Handle `wasmcloud:secrets/store.get`
    fn get(
        &self,
        key: &str,
    ) -> anyhow::Result<Result<Secret, SecretsError>>;

    /// Handle `wasmcloud:secrets/reveal.reveal`
    fn reveal(
        &self,
        secret: Secret,
    ) -> anyhow::Result<SecretValue>;
}
 */

struct BasicSecretsImports;

impl BasicSecretsImports {

    fn fetch_secret(secret: Secret) -> Result<SecretValue, SecretsError> {
        match env::var(secret.key) {
            Ok(val) => Ok( SecretValue::new_string(val)),
            Err(_) => Err(SecretsError::NotFound),
        }
    }
}
 
impl HostSecret for BasicSecretsImports {
    fn drop(&mut self,rep:wasmtime::component::Resource<secrets_store::Secret>) -> wasmtime::Result<()> {
        // Ignore because we are not storing anything
        Ok(())
    }
}



impl Host for BasicSecretsImports {
    #[doc = " Gets a single opaque secrets value set at the given key if it exists"]
    fn get(&mut self,key:wasmtime::component::__internal::String,) -> std::result::Result<std::result::Result<wasmtime::component::Resource<Secret>, SecretsError>, anyhow::Error> {
        match BasicSecretsImports::fetch_secret(&key) {
            Ok(secret) => Ok(Secret {
                value: Some(secret.value),
            }),
            Err(SecretsError::NotFound) => {
                Err(SecretsError::NotFound.into())
            }
            Err(_) => Err(SecretsError::Io("Error".into()).into()),
        }
    }
    
    fn reveal(&mut self,s:Resource<Secret>,) -> Result<SecretValue> {
        todo!()
    }

    
}
