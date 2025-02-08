use futures::executor::block_on;
use pyo3::prelude::*;

use uuid::Uuid;

use crate::address::ProtocolAddress;
use crate::error::{Result, SignalProtocolError};
use crate::identity_key::{IdentityKey, IdentityKeyPair};
use crate::sender_keys::SenderKeyRecord;
use crate::state::{PreKeyId, PreKeyRecord, SessionRecord, SignedPreKeyId, SignedPreKeyRecord};

// traits
use libsignal_protocol::{
    IdentityKeyStore, PreKeyStore, SenderKeyStore, SessionStore, SignedPreKeyStore
};

#[pyclass]
#[derive(Clone)]
pub struct InMemSignalProtocolStore {
    pub store: libsignal_protocol::InMemSignalProtocolStore,
}

#[pymethods]
impl InMemSignalProtocolStore {
    #[new]
    fn new(key_pair: &IdentityKeyPair, registration_id: u32) -> PyResult<InMemSignalProtocolStore> {
        match libsignal_protocol::InMemSignalProtocolStore::new(key_pair.key, registration_id)
        {
            Ok(store) => Ok(Self { store }),
            Err(err) => Err(SignalProtocolError::new_err(err)),
        }
    }

    /// libsignal_protocol::IdentityKeyStore
    /// is_trusted_identity is not implemented (it requries traits::Direction as arg)
    fn get_identity_key_pair(&self) -> Result<IdentityKeyPair> {
        let key = block_on(self.store.identity_store.get_identity_key_pair())?;
        Ok(IdentityKeyPair { key })
    }

    fn get_local_registration_id(&self) -> Result<u32> {
        Ok(block_on(
            self.store.identity_store.get_local_registration_id(),
        )?)
    }

    fn save_identity(&mut self, address: &ProtocolAddress, identity: &IdentityKey) -> Result<bool> {
        Ok(block_on(self.store.identity_store.save_identity(
            &address.state,
            &identity.key,
        ))?)
    }

    fn get_identity(&self, address: &ProtocolAddress) -> Result<Option<IdentityKey>> {
        let key = block_on(self.store.identity_store.get_identity(&address.state))?;

        match key {
            Some(key) => Ok(Some(IdentityKey { key })),
            None => Ok(None),
        }
    }

    /// libsignal_protocol::SessionStore
    pub fn load_session(&self, address: &ProtocolAddress) -> Result<Option<SessionRecord>> {
        let session = block_on(self.store.load_session(&address.state))?;

        match session {
            None => Ok(None),
            Some(state) => Ok(Some(SessionRecord { state })),
        }
    }

    fn store_session(&mut self, address: &ProtocolAddress, record: &SessionRecord) -> Result<()> {
        block_on(
            self.store
                .store_session(&address.state, &record.state),
        )?;
        Ok(())
    }

    /// libsignal_protocol::PreKeyStore
    fn get_pre_key(&self, id: PreKeyId) -> Result<PreKeyRecord> {
        let state = block_on(self.store.pre_key_store.get_pre_key(id.into()))?;
        Ok(PreKeyRecord { state })
    }

    fn save_pre_key(&mut self, id: PreKeyId, record: &PreKeyRecord) -> Result<()> {
        block_on(
            self.store
                .pre_key_store
                .save_pre_key(id.into(), &record.state),
        )?;
        Ok(())
    }

    fn remove_pre_key(&mut self, id: PreKeyId) -> Result<()> {
        block_on(self.store.pre_key_store.remove_pre_key(id.into()))?;
        Ok(())
    }


    /// libsignal_protocol::SignedPreKeyStore
    fn get_signed_pre_key(&self, id: SignedPreKeyId) -> Result<SignedPreKeyRecord> {
        let state = block_on(self.store.get_signed_pre_key(id.into()))?;
        Ok(SignedPreKeyRecord { state })
    }

    fn save_signed_pre_key(
        &mut self,
        id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
    ) -> Result<()> {
        block_on(
            self.store
                .save_signed_pre_key(id.into(), &record.state.to_owned()),
        )?;
        Ok(())
    }

    /// libsignal_protocol::SenderKeyStore
    fn store_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: String,
        record: &SenderKeyRecord,
    ) -> Result<()> {
        Ok(block_on(self.store.store_sender_key(
            &sender.state,
            Uuid::parse_str(&distribution_id).unwrap(),
            &record.state,
        ))?)
    }

    fn load_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: String,
    ) -> Result<Option<SenderKeyRecord>> {
        match block_on(self.store.load_sender_key(&sender.state, Uuid::parse_str(&distribution_id).unwrap()))? {
            Some(state) => Ok(Some(SenderKeyRecord { state })),
            None => Ok(None),
        }
    }
}

/// The storage traits are not exposed as part of the API (this is not supported by Pyo3)
///
/// Python classes for InMemSenderKeyStore, InMemSessionStore, InMemIdentityKeyStore, InMemPreKeyStore
/// or InMemSignedPreKeyStore are not exposed.
/// One will need to operate on the InMemSignalProtocolStore instead.
pub fn init_submodule(module: &PyModule) -> PyResult<()> {
    module.add_class::<InMemSignalProtocolStore>()?;
    Ok(())
}
