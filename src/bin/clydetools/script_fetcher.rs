// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::vec::Vec;

use anyhow::{anyhow, Result};
use boa_engine::property::Attribute;
use boa_engine::{
    js_string, Context, JsError, JsObject, JsResult, JsValue, NativeFunction, Source,
};
use boa_runtime::Console;
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap, HeaderValue};
use semver::Version;
use serde::Deserialize;
use serde_json;

use clyde::package::Package;
use clyde::ui::Ui;

use crate::fetch::{Fetcher, UpdateStatus};
use crate::url_selector::{select_best_urls, BestUrlOptions};

const SCRIPT_FILE_NAME: &str = "fetch.js";

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context) {
    // We first add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}

fn create_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("clydetools"));

    Ok(headers)
}

fn http_get(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let url = args
        .first()
        .unwrap()
        .to_string(context)?
        .to_std_string()
        .unwrap();

    let headers = create_headers()
        .map_err(|x| JsError::from_opaque(format!("Failed to create headers: {}", x).into()))?;

    let response = Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .map_err(|x| JsError::from_opaque(format!("Failed to fetch {url}: {}", x).into()))?;

    let status = response.status().as_u16();
    let text = response.text().unwrap();

    let rv = JsObject::default();
    let _ = rv.create_data_property("status", JsValue::Integer(status.into()), context);
    let _ = rv.create_data_property("text", js_string!(text), context);

    Ok(JsValue::Object(rv))
}

#[derive(Default)]
pub struct ScriptFetcher {}

#[derive(Debug, Deserialize)]
struct ScriptResponse {
    version: String,
    urls: Vec<String>,
}

impl Fetcher for ScriptFetcher {
    fn can_fetch(&self, _package: &Package) -> bool {
        // We can't fetch unless we are explicitly set as the fetcher
        false
    }

    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus> {
        ui.info("Loading fetcher script");
        let script_path = package.package_dir.join(SCRIPT_FILE_NAME);
        let script = fs::read_to_string(script_path)?;

        ui.info("Running fetcher script");
        let response = eval_script(&script)?;

        let version = Version::parse(&response.version)?;
        if let Some(latest_version) = package.get_latest_version() {
            if version <= *latest_version {
                return Ok(UpdateStatus::UpToDate);
            }
        }

        let urls = select_best_urls(ui, &response.urls, BestUrlOptions::default())?;

        Ok(UpdateStatus::NeedUpdate { version, urls })
    }
}

fn eval_script(script: &str) -> Result<ScriptResponse> {
    let mut context = Context::default();

    // Add console
    add_runtime(&mut context);

    // Add httpGet
    context
        .register_global_builtin_callable("httpGet", 1, NativeFunction::from_fn_ptr(http_get))
        .unwrap();

    // Run script
    let source = Source::from_bytes(&script);
    let result = match context.eval(source) {
        Ok(x) => x,
        Err(x) => return Err(anyhow!("Fetcher script failed: {}", x)),
    };

    // Convert result into a ScriptResponse
    let json_result = match result.to_json(&mut context) {
        Ok(x) => x,
        Err(x) => return Err(anyhow!("Could not turn results into JSON: {}", x)),
    };

    if json_result.is_null() {
        return Err(anyhow!("Fetch script did not find any available versions"));
    }

    Ok(serde_json::from_value::<ScriptResponse>(json_result)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_script_return_version() {
        // GIVEN a script which returns a version
        let script = r#"
            function main() {
                return {
                    "version": "1.2.3",
                    "urls": [
                        "https://acme.com/1",
                        "https://acme.com/2"
                    ]
                }
            }
            main()
        "#;

        // WHEN eval_script() is called on it
        let response = eval_script(&script);

        // THEN it returns a ServerResponse object
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(
            Version::parse(&response.version).unwrap(),
            Version::new(1, 2, 3)
        );

        let expected_urls = vec![
            "https://acme.com/1".to_string(),
            "https://acme.com/2".to_string(),
        ];
        assert_eq!(response.urls, expected_urls);
    }

    #[test]
    fn eval_script_return_null() {
        // GIVEN a script which returns null
        let script = r#"
            function main() {
                return null
            }
            main()
        "#;

        // WHEN eval_script() is called on it
        let response = eval_script(&script);

        // THEN it returns an error
        assert!(response.is_err());
    }
}
