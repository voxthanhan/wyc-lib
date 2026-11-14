use napi_derive::napi;
use wyc_binding_contract::Handle;

#[napi]
pub fn analyze_json(input: String) -> String {
    wyc_binding_contract::analyze_json(&input)
}

#[napi]
pub fn indicators_json(input: String) -> String {
    wyc_binding_contract::indicators_json(&input)
}

#[napi]
pub fn structure_json(input: String) -> String {
    wyc_binding_contract::structure_json(&input)
}

#[napi]
pub struct NativeState {
    handle: Handle,
}

#[napi]
impl NativeState {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            handle: Handle::default(),
        }
    }
    #[napi]
    pub fn result_json(&self) -> String {
        self.handle.result_json()
    }
    #[napi]
    pub fn identity_json(&self) -> String {
        self.handle.identity_json()
    }
    #[napi]
    pub fn reset_json(&mut self, input: String) -> String {
        self.handle.reset_json(&input)
    }
    #[napi]
    pub fn update_json(&mut self, input: String) -> String {
        self.handle.update_json(&input)
    }
    #[napi]
    pub fn close(&mut self) {
        self.handle.close();
    }
}

impl Default for NativeState {
    fn default() -> Self {
        Self::new()
    }
}
