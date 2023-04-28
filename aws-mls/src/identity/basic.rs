use crate::{identity::CredentialType, identity::SigningIdentity, time::MlsTime};
use alloc::vec;
use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use aws_mls_core::{
    extension::ExtensionList,
    group::RosterUpdate,
    identity::{IdentityProvider, IdentityWarning},
};
use thiserror::Error;

pub use aws_mls_core::identity::BasicCredential;

#[derive(Debug, Error)]
#[error("unsupported credential type found: {0:?}")]
/// Error returned in the event that a non-basic
/// credential is passed to a [`BasicIdentityProvider`].
pub struct BasicIdentityProviderError(CredentialType);

impl From<CredentialType> for BasicIdentityProviderError {
    fn from(value: CredentialType) -> Self {
        BasicIdentityProviderError(value)
    }
}

impl BasicIdentityProviderError {
    pub fn credential_type(&self) -> CredentialType {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
/// An always-valid identity provider that works with [`BasicCredential`].
///
/// # Warning
///
/// This provider always returns `true` for `validate` as long as the
/// [`SigningIdentity`] used contains a [`BasicCredential`]. It is only
/// recommended to use this provider for testing purposes.
pub struct BasicIdentityProvider;

impl BasicIdentityProvider {
    pub fn new() -> Self {
        Self
    }
}

fn resolve_basic_identity(
    signing_id: &SigningIdentity,
) -> Result<&BasicCredential, BasicIdentityProviderError> {
    signing_id
        .credential
        .as_basic()
        .ok_or_else(|| BasicIdentityProviderError(signing_id.credential.credential_type()))
}

#[async_trait]
impl IdentityProvider for BasicIdentityProvider {
    type Error = BasicIdentityProviderError;

    async fn validate_member(
        &self,
        signing_identity: &SigningIdentity,
        _timestamp: Option<MlsTime>,
        _extensions: Option<&ExtensionList>,
    ) -> Result<(), Self::Error> {
        resolve_basic_identity(signing_identity).map(|_| ())
    }

    #[cfg(feature = "external_proposal")]
    async fn validate_external_sender(
        &self,
        signing_identity: &SigningIdentity,
        _timestamp: Option<MlsTime>,
        _extensions: Option<&ExtensionList>,
    ) -> Result<(), Self::Error> {
        resolve_basic_identity(signing_identity).map(|_| ())
    }

    async fn identity(&self, signing_identity: &SigningIdentity) -> Result<Vec<u8>, Self::Error> {
        resolve_basic_identity(signing_identity).map(|b| b.identifier().to_vec())
    }

    async fn valid_successor(
        &self,
        predecessor: &SigningIdentity,
        successor: &SigningIdentity,
    ) -> Result<bool, Self::Error> {
        Ok(resolve_basic_identity(predecessor)? == resolve_basic_identity(successor)?)
    }

    fn supported_types(&self) -> Vec<CredentialType> {
        vec![BasicCredential::credential_type()]
    }

    async fn identity_warnings(
        &self,
        _update: &RosterUpdate,
    ) -> Result<Vec<IdentityWarning>, Self::Error> {
        Ok(vec![])
    }
}
