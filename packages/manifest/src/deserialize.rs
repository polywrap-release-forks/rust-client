/**
 * This file was automatically generated by templates/deserialize.hbs.
 * DO NOT MODIFY IT BY HAND. Instead, modify templates/deserialize.hbs,
 * and run build script to regenerate this file.
 */

use polywrap_core::package::DeserializeManifestOptions;
use serde_json::Value;

use crate::{
    formats::{AnyManifest, WrapManifest, LATEST_MANIFEST_FORMAT},
    validate::validate_polywrap_manifest,
};

pub fn deserialize_polywrap_manifest(
    manifest: &str,
    options: Option<DeserializeManifestOptions>,
) -> Result<WrapManifest, polywrap_core::error::Error> {
    let any_polywrap_manifest_json = match serde_json::from_str::<Value>(manifest) {
        Ok(any_polywrap_manifest) => any_polywrap_manifest,
        Err(_) => serde_yaml::from_str::<Value>(manifest)
            .map_err(|e| polywrap_core::error::Error::ManifestError(e.to_string()))?,
    };

    let any_polywrap_manifest = AnyManifest::from_json_value(any_polywrap_manifest_json);

    match options {
        Some(opts) => {
            if opts.no_validate == false {
                validate_polywrap_manifest(any_polywrap_manifest.clone(), opts.ext_schema)?;
            };
        }
        None => validate_polywrap_manifest(any_polywrap_manifest.clone(), None)?,
    };

    let any_manifest_ver = semver::Version::parse(&any_polywrap_manifest.format())
        .map_err(|e| polywrap_core::error::Error::ManifestError(e.to_string()))?;

    let latest_manifest_ver = semver::Version::parse(LATEST_MANIFEST_FORMAT).unwrap();

    let version_compare = any_manifest_ver.cmp(&latest_manifest_ver);

    if version_compare.is_eq() {
        match any_polywrap_manifest {
            AnyManifest::WrapManifest01(manifest) => Ok(manifest),
        }
    } else {
        Err(polywrap_core::error::Error::ManifestError(format!(
            "Unsupported manifest version: {}. Latest supported version is {}",
            any_manifest_ver, latest_manifest_ver
        )))
    }
}