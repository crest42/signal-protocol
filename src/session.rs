use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use futures::executor::block_on;
use rand::rngs::OsRng;
use std::time::SystemTime;

use crate::address::ProtocolAddress;
use crate::error::Result;
use crate::state::PreKeyBundle;
use crate::storage::InMemSignalProtocolStore;

// #[pyfunction]
// pub fn process_prekey(
//     message: &PreKeySignalMessage,
//     remote_address: &ProtocolAddress,
//     session_record: &mut SessionRecord,
//     protocol_store: &mut InMemSignalProtocolStore,
// ) -> Result<Option<PreKeyId>> {
//     let result = block_on(libsignal_protocol::process_prekey(
//         &message.data,
//         &remote_address.state,
//         &mut session_record.state,
//         &mut protocol_store.store.identity_store,
//         &mut protocol_store.store.pre_key_store,
//         &mut protocol_store.store.signed_pre_key_store,
//         &mut protocol_store.store.kyber_pre_key_store,
//     ))?;
//     Ok(Some(result.pre_key_id.expect("").0))
// }

#[pyfunction]
pub fn process_prekey_bundle(
    remote_address: ProtocolAddress,
    protocol_store: &mut InMemSignalProtocolStore,
    bundle: PreKeyBundle,
) -> Result<()> {
    let mut csprng = OsRng;
    block_on(libsignal_protocol::process_prekey_bundle(
        &remote_address.state,
        &mut protocol_store.store.session_store,
        &mut protocol_store.store.identity_store,
        &bundle.state,
        SystemTime::now(),
        &mut csprng,
    ))?;
    Ok(())
}

pub fn init_submodule(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(process_prekey_bundle))?;
    // module.add_wrapped(wrap_pyfunction!(process_prekey))?;
    Ok(())
}
