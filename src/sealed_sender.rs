use crate::address::ProtocolAddress;
use crate::curve::{PrivateKey, PublicKey};
use crate::error::{Result, SignalProtocolError};
use crate::storage::InMemSignalProtocolStore;

use futures::executor::block_on;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::wrap_pyfunction;

use libsignal_protocol::Timestamp;
use rand::rngs::OsRng;

use std::time::SystemTime;
#[pyclass]
#[derive(Debug, Clone)]
pub struct ServerCertificate {
    pub data: libsignal_protocol::ServerCertificate,
}

#[pymethods]
impl ServerCertificate {
    #[staticmethod]
    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(ServerCertificate {
            data: libsignal_protocol::ServerCertificate::deserialize(data)?,
        })
    }

    #[new]
    fn new(key_id: u32, key: PublicKey, trust_root: &PrivateKey) -> PyResult<Self> {
        let mut csprng = OsRng;
        match libsignal_protocol::ServerCertificate::new(
            key_id,
            key.key,
            &trust_root.key,
            &mut csprng,
        ) {
            Ok(data) => Ok(Self { data }),
            Err(err) => Err(SignalProtocolError::new_err(err)),
        }
    }

    fn validate(&self, trust_root: &PublicKey) -> Result<bool> {
        Ok(self.data.validate(&trust_root.key)?)
    }

    fn key_id(&self) -> Result<u32> {
        Ok(self.data.key_id()?)
    }

    fn public_key(&self) -> Result<PublicKey> {
        Ok(PublicKey::new(self.data.public_key()?))
    }

    fn certificate(&self, py: Python) -> Result<PyObject> {
        let result = self.data.certificate()?;
        Ok(PyBytes::new(py, &result).into())
    }

    fn signature(&self, py: Python) -> Result<PyObject> {
        let result = self.data.signature()?;
        Ok(PyBytes::new(py, &result).into())
    }

    fn serialized(&self, py: Python) -> Result<PyObject> {
        let result = self.data.serialized()?;
        Ok(PyBytes::new(py, &result).into())
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SenderCertificate {
    pub data: libsignal_protocol::SenderCertificate,
}

#[pymethods]
impl SenderCertificate {
    #[staticmethod]
    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(SenderCertificate {
            data: libsignal_protocol::SenderCertificate::deserialize(data)?,
        })
    }

    #[new]
    #[pyo3(signature = (sender_uuid, sender_e164, key, sender_device_id, expiration, signer, signer_key))]
    fn new(
        sender_uuid: String,
        sender_e164: Option<String>,
        key: PublicKey,
        sender_device_id: u32,
        expiration: u64,
        signer: ServerCertificate,
        signer_key: &PrivateKey,
    ) -> PyResult<Self> {
        let mut csprng = OsRng;
        match libsignal_protocol::SenderCertificate::new(
            sender_uuid,
            sender_e164,
            key.key,
            sender_device_id.into(),
            Timestamp::from_epoch_millis(expiration),
            signer.data,
            &signer_key.key,
            &mut csprng,
        ) {
            Ok(data) => Ok(Self { data }),
            Err(err) => Err(SignalProtocolError::new_err(err)),
        }
    }

    fn validate(&self, trust_root: &PublicKey, validation_time: u64) -> Result<bool> {
        Ok(self.data.validate(&trust_root.key, Timestamp::from_epoch_millis(validation_time))?)
    }

    fn signer(&self) -> Result<ServerCertificate> {
        Ok(ServerCertificate {
            data: (self.data.signer()?).clone(),
        })
    }

    fn key(&self) -> Result<PublicKey> {
        Ok(PublicKey::new(self.data.key()?))
    }

    fn sender_device_id(&self) -> Result<u32> {
        Ok(self.data.sender_device_id()?.into())
    }

    fn sender_uuid(&self) -> Result<Option<&str>> {
        Ok(Some(self.data.sender_uuid()?))
    }

    fn sender_e164(&self) -> Result<Option<&str>> {
        Ok(self.data.sender_e164()?)
    }

    fn expiration(&self) -> Result<u64> {
        Ok(self.data.expiration()?.epoch_millis())
    }

    fn certificate(&self, py: Python) -> Result<PyObject> {
        let result = self.data.certificate()?;
        Ok(PyBytes::new(py, &result).into())
    }

    fn signature(&self, py: Python) -> Result<PyObject> {
        let result = self.data.signature()?;
        Ok(PyBytes::new(py, &result).into())
    }

    fn serialized(&self, py: Python) -> Result<PyObject> {
        let result = self.data.serialized()?;
        Ok(PyBytes::new(py, &result).into())
    }

}

#[pyclass]
pub struct UnidentifiedSenderMessageContent {
    pub data: libsignal_protocol::UnidentifiedSenderMessageContent,
}

#[pymethods]
impl UnidentifiedSenderMessageContent {
    #[staticmethod]
    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(UnidentifiedSenderMessageContent {
            data: libsignal_protocol::UnidentifiedSenderMessageContent::deserialize(data)?,
        })
    }

    #[new]
    fn new(msg_type_value: u8, sender: SenderCertificate, contents: Vec<u8>, content_hint: u32, group_id: Vec<u8>) -> PyResult<Self> {
        let msg_enum = match msg_type_value {
            2 => libsignal_protocol::CiphertextMessageType::Whisper,
            3 => libsignal_protocol::CiphertextMessageType::PreKey,
            7 => libsignal_protocol::CiphertextMessageType::SenderKey,
            _ => {
                return Err(SignalProtocolError::err_from_str(format!(
                    "unknown message type: {}",
                    msg_type_value
                )))
            }
        };
        match libsignal_protocol::UnidentifiedSenderMessageContent::new(
            msg_enum,
            sender.data,
            contents,
            libsignal_protocol::ContentHint::from(content_hint),
            Some(group_id)
        ) {
            Ok(data) => Ok(Self { data }),
            Err(err) => Err(SignalProtocolError::new_err(err)),
        }
    }

    fn msg_type(&self) -> Result<u8> {
        Ok(self.data.msg_type()? as u8)
    }

    fn sender(&self) -> Result<SenderCertificate> {
        Ok(SenderCertificate {
            data: (self.data.sender()?).clone(),
        })
    }

    fn contents(&self, py: Python) -> Result<PyObject> {
        let result = self.data.contents()?;
        Ok(PyBytes::new(py, &result).into())
    }

    fn serialized(&self, py: Python) -> Result<PyObject> {
        let result = self.data.serialized()?;
        Ok(PyBytes::new(py, &result).into())
    }
}

#[pyclass]
pub struct SealedSenderDecryptionResult {
    pub data: libsignal_protocol::SealedSenderDecryptionResult,
}

#[pymethods]
impl SealedSenderDecryptionResult {
    pub fn sender_uuid(&self) -> String {
        self.data.sender_uuid.clone()
    }

    pub fn sender_e164(&self) -> Option<String> {
        self.data.sender_e164.clone()
    }

    pub fn device_id(&self) -> u32 {
        self.data.device_id.into()
    }

    fn message(&self, py: Python) -> Result<PyObject> {
        Ok(PyBytes::new(py, &self.data.message).into())
    }
}

#[pyfunction]
#[pyo3(signature = (ciphertext, trust_root, timestamp, local_e164, local_uuid, local_device_id, protocol_store))]
pub fn sealed_sender_decrypt(
    ciphertext: &[u8],
    trust_root: &PublicKey,
    timestamp: u64,
    local_e164: Option<String>,
    local_uuid: String,
    local_device_id: u32,
    protocol_store: &mut InMemSignalProtocolStore,
) -> PyResult<SealedSenderDecryptionResult> {
    match block_on(libsignal_protocol::sealed_sender_decrypt(
        ciphertext,
        &trust_root.key,
        Timestamp::from_epoch_millis(timestamp),
        local_e164,
        local_uuid,
        local_device_id.into(),
        &mut protocol_store.store.identity_store,
        &mut protocol_store.store.session_store,
        &mut protocol_store.store.pre_key_store,
        &mut protocol_store.store.signed_pre_key_store,
        &mut protocol_store.store.kyber_pre_key_store
    )) {
        Ok(data) => Ok(SealedSenderDecryptionResult { data }),
        Err(err) => Err(SignalProtocolError::new_err(err)),
    }
}

#[pyfunction]
pub fn sealed_sender_encrypt(
    destination: &ProtocolAddress,
    sender_cert: &SenderCertificate,
    ptext: &[u8],
    protocol_store: &mut InMemSignalProtocolStore,
    py: Python,
) -> Result<PyObject> {
    let mut csprng = OsRng;
    let result = block_on(libsignal_protocol::sealed_sender_encrypt(
        &destination.state,
        &sender_cert.data,
        ptext,
        &mut protocol_store.store.session_store,
        &mut protocol_store.store.identity_store,
        SystemTime::now(),
        &mut csprng,
    ))?;
    Ok(PyBytes::new(py, &result).into())
}

#[pyfunction]
pub fn sealed_sender_decrypt_to_usmc(
    ciphertext: &[u8],
    protocol_store: &mut InMemSignalProtocolStore,
) -> PyResult<UnidentifiedSenderMessageContent> {
    match block_on(libsignal_protocol::sealed_sender_decrypt_to_usmc(
        ciphertext,
        &mut protocol_store.store.identity_store,
    )) {
        Ok(data) => Ok(UnidentifiedSenderMessageContent { data }),
        Err(err) => Err(SignalProtocolError::new_err(err)),
    }
}

pub fn init_submodule(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<SenderCertificate>()?;
    module.add_class::<ServerCertificate>()?;
    module.add_class::<UnidentifiedSenderMessageContent>()?;
    module.add_class::<SealedSenderDecryptionResult>()?;
    module.add_wrapped(wrap_pyfunction!(sealed_sender_decrypt))?;
    module.add_wrapped(wrap_pyfunction!(sealed_sender_decrypt_to_usmc))?;
    module.add_wrapped(wrap_pyfunction!(sealed_sender_encrypt))?;
    Ok(())
}
