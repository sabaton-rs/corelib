use crate::error::CoreError;
pub struct InstanceSpecifier {
    spec: String,
}

impl InstanceSpecifier {
    pub fn from(instance: &str) -> Result<Self, CoreError> {
        Ok(InstanceSpecifier {
            spec: String::from(instance),
        })
    }

    pub fn get(&self) -> &str {
        &self.spec
    }
}
