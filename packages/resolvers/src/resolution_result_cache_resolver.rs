use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use polywrap_core::{
    invoker::Invoker,
    uri::Uri,
    resolution::{
        uri_resolution_context::{UriPackageOrWrapper, UriResolutionContext, UriResolutionStep},
        uri_resolver::UriResolver
    },
    error::Error
};
use crate::cache::basic_resolution_result_cache::BasicResolutionResultCache;
use crate::cache::resolution_result_cache::ResolutionResultCache;
use crate::uri_resolver_aggregator::UriResolverAggregator;

/// A URI resolver that uses a cache to store and retrieve wrappers that pass through.
pub struct ResolutionResultCacheResolver {
    resolver: Arc<dyn UriResolver>,
    cache: Mutex<Box<dyn ResolutionResultCache>>,
}

impl ResolutionResultCacheResolver {
    /// Creates a new `ResolutionResultCacheResolver`.
    ///
    /// # Arguments
    ///
    /// * `resolver` - The `UriResolver` to use when resolving URIs.
    /// * `cache` - The cache to store and retrieve resolved URIs.
    ///
    /// # Returns
    ///
    /// * A new `ResolutionResultCacheResolver`.
    pub fn new(resolver: Arc<dyn UriResolver>, cache: Mutex<Box<dyn ResolutionResultCache>>) -> ResolutionResultCacheResolver {
        ResolutionResultCacheResolver { resolver, cache }
    }

    fn cache_resolution_path(&self, resolution_context: Arc<Mutex<UriResolutionContext>>, result: Arc<Result<UriPackageOrWrapper, Error>>) {
        let resolution_path = resolution_context.lock().unwrap().get_resolution_path();
        for uri in resolution_path {
            self.cache.lock().unwrap().set(uri, result.clone());
        }
    }
}

impl UriResolver for ResolutionResultCacheResolver {
    /// Tries to resolve the given URI using a cache and returns the result.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to resolve.
    /// * `invoker` - The invoker of the resolution.
    /// * `resolution_context` - The context for the resolution.
    ///
    /// # Returns
    ///
    /// * A `Result` containing the resolved `UriPackageOrWrapper` on success, or an exception on failure.
    fn try_resolve_uri(
        &self,
        uri: &Uri,
        invoker: Arc<dyn Invoker>,
        resolution_context: Arc<Mutex<UriResolutionContext>>,
    ) -> Result<UriPackageOrWrapper, Error> {
        if let Some(cache_result) = self.cache.lock().unwrap().get(uri) {
            let result = cache_result.clone().deref().clone();
            resolution_context.lock().unwrap().track_step(
                UriResolutionStep {
                    source_uri: uri.clone(),
                    result: result.clone(),
                    sub_history: None,
                    description: Some("ResolutionResultCacheResolver (Cache)".to_string()),
                }
            );
            return result;
        }

        let sub_context = resolution_context.lock().unwrap().create_sub_history_context();
        let sub_context = Arc::new(Mutex::new(sub_context));
        let result = self.resolver.try_resolve_uri(uri, invoker.clone(), sub_context.clone());

        if result.is_ok() {
            self.cache_resolution_path(sub_context.clone(), Arc::from(result.clone()));
        }

        resolution_context.lock().unwrap().track_step(
            UriResolutionStep {
                source_uri: uri.clone(),
                result: result.clone(),
                sub_history: Some(sub_context.lock().unwrap().get_history().clone()),
                description: Some("ResolutionResultCacheResolver".to_string()),
            }
        );

        return result;
    }
}

impl fmt::Debug for ResolutionResultCacheResolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResolutionResultCacheResolver")
    }
}

impl From<Vec<Box<dyn UriResolver>>> for ResolutionResultCacheResolver {
    fn from(resolvers: Vec<Box<dyn UriResolver>>) -> Self {
        ResolutionResultCacheResolver::from(
            UriResolverAggregator::from(resolvers)
        )
    }
}

impl From<UriResolverAggregator> for ResolutionResultCacheResolver {
    fn from(resolver: UriResolverAggregator) -> Self {
        ResolutionResultCacheResolver::new(
            Arc::new(resolver),
            Mutex::new(Box::new(BasicResolutionResultCache::new()))
        )
    }
}

impl From<Box<dyn UriResolver>> for ResolutionResultCacheResolver {
    fn from(resolver: Box<dyn UriResolver>) -> Self {
        ResolutionResultCacheResolver::new(
            Arc::from(resolver),
            Mutex::new(Box::new(BasicResolutionResultCache::new()))
        )
    }
}