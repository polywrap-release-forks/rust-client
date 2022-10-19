use crate::{wrapper::Wrapper, error::CoreError};
use async_trait::async_trait;

#[async_trait]
pub trait WrapPackage {
  async fn create_wrapper(&self) -> Result<Box<dyn Wrapper>, CoreError>;
}