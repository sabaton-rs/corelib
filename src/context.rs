use crate::error::CoreError;

// The context structure


pub struct Context {

}

impl Context {
 
    pub fn create() -> Self {
        Context {

        }
    }
 
    pub fn initialize(&mut self) -> Result<(),CoreError> {
       Err(CoreError::NotImplemented)
    }

    pub fn deinitialize(&mut self) -> Result<(),CoreError> {
        Err(CoreError::NotImplemented)
    }
}