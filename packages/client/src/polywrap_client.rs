use std::sync::Arc;

use async_trait::async_trait;
use polywrap_core::{
    client::{Client, UriRedirect},
    error::Error,
    invoke::{Invoker, InvokeArgs},
    loader::Loader,
    uri::Uri,
    uri_resolution_context::UriResolutionContext,
    uri_resolver::{UriResolverHandler, UriResolver},
    wrapper::Wrapper, env::{Env, Envs},
    interface_implementation::InterfaceImplementations
};
use polywrap_msgpack::{decode, DeserializeOwned};
use tokio::sync::Mutex;

use crate::{wrapper_invoker::WrapperInvoker, wrapper_loader::WrapperLoader};

#[derive(Clone)]
pub struct PolywrapClient {
    redirects: Vec<UriRedirect>,
    envs: Option<Envs>,
    loader: WrapperLoader,
    interfaces: Option<InterfaceImplementations>,
    invoker: WrapperInvoker,
}

impl PolywrapClient {
    pub fn new(resolver: Box<dyn UriResolver>, interfaces: Option<InterfaceImplementations>) -> Self {
      let resolver = Arc::new(Mutex::new(resolver as Box<dyn UriResolver>));
      let loader = WrapperLoader::new(resolver.clone());
      let invoker = WrapperInvoker::new(loader.clone(), interfaces.clone());

      Self {
        invoker,
        loader,
        redirects: vec![],
        envs: None,
        interfaces,
      }
    }

    pub fn environment<'a>(&'a mut self, envs: Envs) -> Self {
      self.envs = Some(envs);
      self.clone()
    }

    pub fn redirects<'a>(&'a mut self, redirects: Vec<UriRedirect>) -> Self {
      self.redirects = redirects;
      self.clone()
    }

    pub async fn invoke_wrapper_and_decode<T: DeserializeOwned>(
        &self,
        wrapper: Arc<Mutex<dyn Wrapper>>,
        uri: &Uri,
        method: &str,
        args: Option<&InvokeArgs>,
        env: Option<Env>,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<T, Error> {
        let result = self
            .invoke_wrapper(wrapper, uri, method, args, env, resolution_context)
            .await?;
        decode(result.as_slice())
            .map_err(|e| Error::InvokeError(format!("Failed to decode result: {}", e)))
    }

    pub async fn invoke_and_decode<T: DeserializeOwned>(
        &self,
        uri: &Uri,
        method: &str,
        args: Option<&InvokeArgs>,
        env: Option<Env>,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<T, Error> {
        let result = self.invoke(uri, method, args, env, resolution_context).await?;
        decode(result.as_slice())
            .map_err(|e| Error::InvokeError(format!("Failed to decode result: {}", e)))
    }
}

#[async_trait]
impl Invoker for PolywrapClient {
    async fn invoke(
        &self,
        uri: &Uri,
        method: &str,
        args: Option<&InvokeArgs>,
        env: Option<Env>,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<Vec<u8>, Error> {
        let env_uri = match env {
            Some(env) => Some(env),
            None => {
                if let Some(env) = self.get_env_by_uri(uri) {
                    Some(env.to_owned())
                } else {
                    None
                }
            }
        };
        self.invoker.invoke(uri, method, args, env_uri, resolution_context).await
    }

    async fn invoke_wrapper(
        &self,
        wrapper: Arc<Mutex<dyn Wrapper>>,
        uri: &Uri,
        method: &str,
        args: Option<&InvokeArgs>,
        env: Option<Env>,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<Vec<u8>, Error> {
        self.invoker.invoke_wrapper(wrapper, uri, method, args, env, resolution_context).await
    }

    fn get_implementations(&self, uri: Uri) -> Result<Vec<Uri>, Error> {
        self.invoker.get_implementations(uri)
    }
}

#[async_trait(?Send)]
impl Client for PolywrapClient {
    fn get_redirects(&self) -> &Vec<UriRedirect> {
        &self.redirects
    }

    fn get_env_by_uri(&self, uri: &Uri) -> Option<&Env> {
        if let Some(envs) = &self.envs {
            return envs.get(&uri.uri);
        }

        None
    }

    fn get_interfaces(&self) -> Option<&InterfaceImplementations> {
        if let Some(interfaces) = &self.interfaces {
            return Some(interfaces);
        }

        None
    }
}

#[async_trait]
impl UriResolverHandler for PolywrapClient {
    async fn try_resolve_uri(
        &self,
        uri: &Uri,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<polywrap_core::uri_resolution_context::UriPackageOrWrapper, Error> {
        self.loader.try_resolve_uri(uri, resolution_context).await
    }
}

#[async_trait]
impl Loader for PolywrapClient {
    async fn load_wrapper(
        &self,
        uri: &Uri,
        resolution_context: Option<&mut UriResolutionContext>,
    ) -> Result<Arc<Mutex<dyn Wrapper>>, Error> {
        self.loader.load_wrapper(uri, resolution_context).await
    }
}
