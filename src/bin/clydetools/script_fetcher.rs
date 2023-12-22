// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use boa_engine::property::Attribute;
use boa_engine::{js_string, Context, JsObject, JsResult, JsValue, NativeFunction, Source};
use boa_runtime::Console;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use serde_json;

use clyde::arch_os::ArchOs;
use clyde::package::{FetcherConfig, Package};
use clyde::ui::Ui;

use crate::fetch::{Fetcher, UpdateStatus};

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context) {
    // We first add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}

fn http_get(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let url = args
        .get(0)
        .unwrap()
        .to_string(context)?
        .to_std_string()
        .unwrap();
    let response = Client::new().get(url).send().unwrap();

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
    urls: HashMap<String, String>,
}

impl Fetcher for ScriptFetcher {
    fn can_fetch(&self, _package: &Package) -> bool {
        // We can't fetch unless we are explicitly set as the fetcher
        false
    }

    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus> {
        ui.info("Running fetcher script");
        let script = match &package.fetcher {
            FetcherConfig::Script { script } => script,
            _ => panic!("ScriptFetcher should not be called with a FetcherConfig other Script"),
        };

        let response = eval_script(script)?;

        let version = Version::parse(&response.version)?;
        if let Some(latest_version) = package.get_latest_version() {
            if version <= *latest_version {
                return Ok(UpdateStatus::UpToDate);
            }
        }

        let urls: HashMap<ArchOs, String> = response
            .urls
            .iter()
            .map(|(arch_os_str, url)| (ArchOs::parse(arch_os_str).unwrap(), url.clone()))
            .collect();

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

    Ok(serde_json::from_value::<ScriptResponse>(json_result)?)
}
