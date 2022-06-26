mod grpc;
use std::{path::PathBuf, sync::Arc};

use flaunch_core::{load_settings, script_engine::ScriptEngine, SettingKey, load_logging};
use grpc::run_gprc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_logging();
    let settings = load_settings();
    let engine = ScriptEngine::default();
    if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
        let path = PathBuf::from(script_path);
        engine.load(&path).await.unwrap();
    }
    run_gprc_server(Arc::new(engine)).await
}
