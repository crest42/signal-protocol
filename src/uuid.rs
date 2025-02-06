use pyo3::prelude::*;
use uuid::uuid;

#[pyclass]
#[derive(Debug, Clone)]
pub struct MyUuid {
    pub uuid: uuid::Uuid,
}


#[pymethods]
impl MyUuid {
    #[new]
    fn new() -> Self {
        MyUuid {
            uuid: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8") //TODO!!!!
        }
    }
}

pub fn init_submodule(module: &PyModule) -> PyResult<()> {
    module.add_class::<MyUuid>()?;
    Ok(())
}
