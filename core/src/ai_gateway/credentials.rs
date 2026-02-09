use std::env;

use async_trait::async_trait;

use crate::ai_gateway::{
    error::{GatewayError, GatewayErrorKind, invalid_request},
    types::{BackendProfile, CredentialRef, ResolvedCredential},
};

#[async_trait]
pub trait CredentialProvider: Send + Sync {
    async fn resolve(
        &self,
        reference: &CredentialRef,
        backend: &BackendProfile,
    ) -> Result<ResolvedCredential, GatewayError>;
}

#[derive(Default)]
pub struct EnvCredentialProvider;

#[async_trait]
impl CredentialProvider for EnvCredentialProvider {
    async fn resolve(
        &self,
        reference: &CredentialRef,
        backend: &BackendProfile,
    ) -> Result<ResolvedCredential, GatewayError> {
        match reference {
            CredentialRef::Env { var } => {
                let token = env::var(var).map_err(|_| {
                    GatewayError::new(
                        GatewayErrorKind::Authentication,
                        format!(
                            "missing credential environment variable {} for backend {}",
                            var, backend.id
                        ),
                    )
                    .with_retryable(false)
                    .with_backend_id(backend.id.clone())
                })?;

                Ok(ResolvedCredential {
                    auth_header: Some(format!("Bearer {}", token)),
                    extra_headers: Vec::new(),
                    opaque: Default::default(),
                })
            }
            CredentialRef::InlineToken { token } => {
                if token.trim().is_empty() {
                    return Err(invalid_request("inline credential token cannot be empty"));
                }
                Ok(ResolvedCredential {
                    auth_header: Some(format!("Bearer {}", token)),
                    extra_headers: Vec::new(),
                    opaque: Default::default(),
                })
            }
            CredentialRef::None => Ok(ResolvedCredential::none()),
        }
    }
}
