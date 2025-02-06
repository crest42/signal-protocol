use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::wrap_pyfunction;

use futures::executor::block_on;
use rand::rngs::OsRng;

use crate::error::{Result, SignalProtocolError};
use crate::protocol::SenderKeyDistributionMessage;
use crate::address::ProtocolAddress;
use crate::storage::InMemSignalProtocolStore;
use crate::uuid::MyUuid;

#[pyfunction]
pub fn group_encrypt(
    py: Python,
    protocol_store: &mut InMemSignalProtocolStore,
    sender: &ProtocolAddress,
    distribution_id: MyUuid,
    plaintext: &[u8],
) -> Result<PyObject> {
    let mut csprng = OsRng;
    let ciphertext = block_on(libsignal_protocol::group_encrypt(
        &mut protocol_store.store.sender_key_store,
        &sender.state,
        distribution_id.uuid,
        plaintext,
        &mut csprng,
    ))?;
    Ok(PyBytes::new(py, &ciphertext.serialized()).into())
}

#[pyfunction]
pub fn group_decrypt(
    py: Python,
    skm_bytes: &[u8],
    protocol_store: &mut InMemSignalProtocolStore,
    protocol_address: &ProtocolAddress,
) -> Result<PyObject> {
    let plaintext = block_on(libsignal_protocol::group_decrypt(
        skm_bytes,
        &mut protocol_store.store.sender_key_store,
        &protocol_address.state,
    ))?;
    Ok(PyBytes::new(py, &plaintext).into())
}

#[pyfunction]
pub fn process_sender_key_distribution_message(
    protocol_address: &ProtocolAddress,
    skdm: &SenderKeyDistributionMessage,
    protocol_store: &mut InMemSignalProtocolStore,
) -> Result<()> {
    Ok(block_on(
        libsignal_protocol::process_sender_key_distribution_message(
            &protocol_address.state,
            &skdm.data,
            &mut protocol_store.store.sender_key_store,
        ),
    )?)
}

#[pyfunction]
pub fn create_sender_key_distribution_message(
    sender: &ProtocolAddress,
    distribution_id: MyUuid,
    protocol_store: &mut InMemSignalProtocolStore,
) -> PyResult<SenderKeyDistributionMessage> {
    let mut csprng = OsRng;
    let upstream_data = match block_on(
        libsignal_protocol::create_sender_key_distribution_message(
            &sender.state,
            distribution_id.uuid,
            &mut protocol_store.store.sender_key_store,
            &mut csprng,
        ),
    ) {
        Ok(data) => data,
        Err(err) => return Err(SignalProtocolError::new_err(err)),
    };
    Ok(SenderKeyDistributionMessage {
        data: upstream_data.clone(),
    })
}

pub fn init_submodule(module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(group_encrypt))?;
    module.add_wrapped(wrap_pyfunction!(group_decrypt))?;
    module.add_wrapped(wrap_pyfunction!(process_sender_key_distribution_message))?;
    module.add_wrapped(wrap_pyfunction!(create_sender_key_distribution_message))?;
    Ok(())
}
