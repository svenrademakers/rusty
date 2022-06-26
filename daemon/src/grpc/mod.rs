mod script_engine_service;
use std::sync::Arc;

use tonic::transport::Server;

pub async fn run_gprc_server(
    engine: Arc<flaunch_core::script_engine::ScriptEngine>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let script_engine_service = script_engine_service::ScriptEngineService::new(engine);

    Server::builder()
        .add_service(
            script_engine_service::proto::script_engine_server::ScriptEngineServer::new(
                script_engine_service,
            ),
        )
        .serve(addr)
        .await?;

    Ok(())
}
