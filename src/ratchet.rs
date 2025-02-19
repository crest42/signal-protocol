use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use rand::rngs::OsRng;

use crate::curve::{KeyPair, PublicKey};
use crate::error::Result;
use crate::identity_key::{IdentityKey, IdentityKeyPair};
use crate::state::SessionRecord;

#[pyclass]
pub struct AliceSignalProtocolParameters {
    inner: libsignal_protocol::AliceSignalProtocolParameters,
}

#[pymethods]
impl AliceSignalProtocolParameters {
    #[new]
    pub fn new(
        our_identity_key_pair: IdentityKeyPair,
        our_base_key_pair: KeyPair,
        their_identity_key: IdentityKey,
        their_signed_pre_key: PublicKey,
        their_one_time_pre_key: PublicKey,
    ) -> Self {
        Self {
            inner: libsignal_protocol::AliceSignalProtocolParameters::new(
                our_identity_key_pair.key,
                our_base_key_pair.key,
                their_identity_key.key,
                their_signed_pre_key.key,
                their_one_time_pre_key.key,
            ),
        }
    }

    pub fn our_identity_key_pair(&self) -> Result<IdentityKeyPair> {
        Ok(IdentityKeyPair {
            key: *self.inner.our_identity_key_pair(),
        })
    }

    pub fn our_base_key_pair(&self) -> Result<KeyPair> {
        Ok(KeyPair {
            key: *self.inner.our_base_key_pair(),
        })
    }

    pub fn their_identity_key(&self) -> Result<IdentityKey> {
        Ok(IdentityKey {
            key: *self.inner.their_identity_key(),
        })
    }

    pub fn their_signed_pre_key(&self) -> Result<PublicKey> {
        Ok(PublicKey {
            key: *self.inner.their_signed_pre_key(),
        })
    }

    pub fn their_one_time_pre_key(&self) -> Result<Option<PublicKey>> {
        let key = match self.inner.their_one_time_pre_key() {
            None => return Ok(None),
            Some(key) => key,
        };

        Ok(Some(PublicKey { key: *key }))
    }

    pub fn their_ratchet_key(&self) -> Result<PublicKey> {
        Ok(PublicKey {
            key: *self.inner.their_ratchet_key(),
        })
    }
}

#[pyfunction]
pub fn initialize_alice_session(
    parameters: &AliceSignalProtocolParameters,
) -> Result<SessionRecord> {
    let mut csprng = OsRng;
    let state =
        libsignal_protocol::initialize_alice_session_record(&parameters.inner, &mut csprng)?;
    Ok(SessionRecord { state })
}

#[pyclass]
pub struct BobSignalProtocolParameters {
    inner: libsignal_protocol::BobSignalProtocolParameters<'static>,
}

#[pymethods]
impl BobSignalProtocolParameters {
    #[new]
    #[pyo3(signature = (our_identity_key_pair, our_signed_pre_key_pair, our_one_time_pre_key_pair, our_ratchet_key_pair, their_identity_key, their_base_key))]
    pub fn new(
        our_identity_key_pair: IdentityKeyPair,
        our_signed_pre_key_pair: KeyPair,
        our_one_time_pre_key_pair: Option<KeyPair>,
        our_ratchet_key_pair: KeyPair,
        their_identity_key: IdentityKey,
        their_base_key: PublicKey,
    ) -> Self {
        let upstream_our_one_time_pre_key_pair = match our_one_time_pre_key_pair {
            None => None,
            Some(x) => Some(x.key),
        };

        Self {
            inner: libsignal_protocol::BobSignalProtocolParameters::new(
                our_identity_key_pair.key,
                our_signed_pre_key_pair.key,
                upstream_our_one_time_pre_key_pair,
                our_ratchet_key_pair.key,
                None,
                their_identity_key.key,
                their_base_key.key,
                None,
            ),
        }
    }

    pub fn our_identity_key_pair(&self) -> Result<IdentityKeyPair> {
        Ok(IdentityKeyPair {
            key: *self.inner.our_identity_key_pair(),
        })
    }

    pub fn our_signed_pre_key_pair(&self) -> Result<KeyPair> {
        Ok(KeyPair {
            key: *self.inner.our_signed_pre_key_pair(),
        })
    }

    pub fn our_one_time_pre_key_pair(&self) -> Result<Option<KeyPair>> {
        let keypair = match self.inner.our_one_time_pre_key_pair() {
            None => return Ok(None),
            Some(keypair) => keypair,
        };

        Ok(Some(KeyPair { key: *keypair }))
    }

    pub fn our_ratchet_key_pair(&self) -> Result<KeyPair> {
        Ok(KeyPair {
            key: *self.inner.our_ratchet_key_pair(),
        })
    }

    pub fn their_identity_key(&self) -> Result<IdentityKey> {
        Ok(IdentityKey {
            key: *self.inner.their_identity_key(),
        })
    }

    pub fn their_base_key(&self) -> Result<PublicKey> {
        Ok(PublicKey {
            key: *self.inner.their_base_key(),
        })
    }
}

#[pyfunction]
pub fn initialize_bob_session(parameters: &BobSignalProtocolParameters) -> Result<SessionRecord> {
    let state = libsignal_protocol::initialize_bob_session_record(&parameters.inner)?;
    Ok(SessionRecord { state })
}

/// fn are_we_alice, ChainKey, RootKey, MessageKey are not exposed as part of the Python API.
pub fn init_submodule(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<AliceSignalProtocolParameters>()?;
    module.add_wrapped(wrap_pyfunction!(initialize_alice_session))?;
    module.add_class::<BobSignalProtocolParameters>()?;
    module.add_wrapped(wrap_pyfunction!(initialize_bob_session))?;
    Ok(())
}
