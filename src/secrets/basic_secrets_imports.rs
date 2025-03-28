
use std::{collections::HashMap, env};
use dotenv::dotenv;
use wasix::mcp::secrets_store::{self, Host, HostSecret, Secret, SecretValue, SecretsError};

use wasmtime::{Result,component::{bindgen, Resource}};

bindgen!({
    world: "wasix:mcp/secrets",
    with: 
    {
        "wasix:mcp/secrets-list": Secrets,
    },
    trappable_imports: true,
});

/// Helper struct for fetching secrets from the environment or `.env` file

struct SecretStore {
    store: HashMap<u32, SecretValue>,
}
impl SecretStore {
    fn new() -> Self {
        SecretStore {
            store: HashMap::new(),
        }
    }

    fn insert(&mut self, handle: u32, value: SecretValue) {
        self.store.insert(handle, value);
    }

    fn get(&self, handle: u32) -> Option<SecretValue> {
        self.store.get(&handle).cloned()
    }
}

struct BasicSecretsImports {
    secret_store: SecretStore,
}

impl BasicSecretsImports {
    fn fetch_secret(key: &str) -> Result<SecretValue, SecretsError> {
        // Load .env file if available
        dotenv().ok();

        // Try fetching the secret from the environment
        match env::var(key) {
            Ok(val) => Ok(SecretValue::String(val)),
            Err(_) => Err(SecretsError::NotFound),
        }
    }
    fn store_secret(&mut self, key: &str) -> Result<u32, SecretsError> {
        let secret_value = BasicSecretsImports::fetch_secret(key)?;

        // Generate a handle for the secret (you can use a unique ID or a counter)
        let handle = self.secret_store.store.len() as u32 + 1;

        // Store the secret in the store
        self.secret_store.insert(handle, secret_value);

        Ok(handle)
    }
}

impl HostSecret for BasicSecretsImports {
    fn drop(&mut self, _rep: Resource<secrets_store::Secret>) -> wasmtime::Result<()> {
        // We are not storing anything, so no action is needed here
        Ok(())
    }
}

impl Host for BasicSecretsImports {
    #[doc = "Gets a single opaque secret value set at the given key if it exists"]
    fn get(
        &mut self,
        key: wasmtime::component::__internal::String,
    ) -> std::result::Result<std::result::Result<Resource<Secret>, SecretsError>, anyhow::Error> {
        let handle = self.store_secret(&key)?; // Store the secret and get its handle

        // Create a new Secret resource with the handle
        Ok(Ok(Resource::<Secret>::new_own(handle)))
    }

    fn reveal(&mut self, s: Resource<Secret>) -> Result<SecretValue> {
        let handle = s.rep(); // Retrieve the handle from the resource

        // Look up the secret value using the handle
        match self.secret_store.get(handle) {
            Some(secret_value) => Ok(secret_value),
            None => Err(SecretsError::NotFound.into()), // Handle not found
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Mock a secret in the environment (simulating a .env file or system env)
    fn setup_mock_env() {
        dotenv().ok();  // Load .env file, if available
        env::set_var("MY_SECRET_KEY", "super_secret_value"); // Set a mock secret
    }

    // Test the fetch_secret function
    #[test]
    fn test_fetch_secret() {
        setup_mock_env(); // Set up the environment variable

        let secret_key = "MY_SECRET_KEY";
        let result = BasicSecretsImports::fetch_secret(secret_key);

        match result {
            Ok(SecretValue::String(val)) => assert_eq!(val, "super_secret_value"), // Check if value matches
            _ => panic!("Expected SecretValue::String, but got something else"),
        }
    }

    // Test the get method to simulate fetching a secret and returning a Resource<Secret>
    #[test]
    fn test_get_secret() {
        setup_mock_env(); // Set up the environment variable

        let secret_key = "MY_SECRET_KEY";
        let mut secrets_imports = BasicSecretsImports {
            secret_store: SecretStore::new(),
        };

        // Store the secret in the SecretStore
        let _ = secrets_imports.store_secret(secret_key).unwrap();

        let result = secrets_imports.get(secret_key.to_string());

        match result {
            Ok(Ok(resource)) => {
                // Check if the resource handle is valid
                let handle = resource.rep();
                assert!(handle != 0, "Resource handle should not be zero");

                // Retrieve and reveal the secret from the resource
                let revealed_value = secrets_imports.reveal(resource);
                match revealed_value {
                    Ok(SecretValue::String(value)) => assert_eq!(value, "super_secret_value"),
                    _ => panic!("Expected SecretValue::String, but got something else"),
                }
            }
            _ => panic!("Expected Ok with resource, but got an error or invalid result"),
        }
    }

    // Test reveal method when secret is valid
    #[test]
    fn test_reveal_valid_secret() {
        setup_mock_env(); // Set up the environment variable

        let secret_key = "MY_SECRET_KEY";
        let mut secrets_imports = BasicSecretsImports {
            secret_store: SecretStore::new(),
        };

        // Store the secret in the SecretStore
        let handle = secrets_imports.store_secret(secret_key).unwrap();

        // Create a Resource<Secret> using the handle (simulate getting the secret)
        let resource = Resource::<Secret>::new_own(handle);

        // Retrieve and reveal the secret from the resource
        let revealed_value = secrets_imports.reveal(resource);
        match revealed_value {
            Ok(SecretValue::String(value)) => assert_eq!(value, "super_secret_value"),
            _ => panic!("Expected SecretValue::String, but got something else"),
        }
    }

    // Test when secret is not found
    #[test]
    fn test_secret_not_found() {
        let secret_key = "NON_EXISTENT_KEY";
        let mut secrets_imports = BasicSecretsImports {
            secret_store: SecretStore::new(),
        };

        let result = secrets_imports.get(secret_key.to_string());

        match result {
            Err(_) => {} // Expected error
            _ => panic!("Expected SecretsError::NotFound, but got something else"),
        }
    }

}
