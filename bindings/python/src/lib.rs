use pyo3::prelude::*;
use wyc_binding_contract::Handle;

#[pyfunction]
fn analyze_json(input: String) -> String {
    wyc_binding_contract::analyze_json(&input)
}

#[pyclass]
struct NativeState {
    handle: Handle,
}

#[pymethods]
impl NativeState {
    #[new]
    fn new() -> Self {
        Self {
            handle: Handle::default(),
        }
    }
    fn result_json(&self) -> String {
        self.handle.result_json()
    }
    fn identity_json(&self) -> String {
        self.handle.identity_json()
    }
    fn reset_json(&mut self, input: String) -> String {
        self.handle.reset_json(&input)
    }
    fn update_json(&mut self, input: String) -> String {
        self.handle.update_json(&input)
    }
    fn close(&mut self) {
        self.handle.close();
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(analyze_json, module)?)?;
    module.add_class::<NativeState>()?;
    Ok(())
}
