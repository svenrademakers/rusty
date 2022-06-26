use flaunch_core::script_engine::ArgumentType;
use std::{pin::Pin, sync::Arc};
use tokio_stream::{Stream, StreamExt};
pub mod proto {
    tonic::include_proto!("flaunch");
}

#[derive(Debug, Default)]
pub struct ScriptEngineService {
    engine: Arc<flaunch_core::script_engine::ScriptEngine>,
}

impl ScriptEngineService {
    pub fn new(engine: Arc<flaunch_core::script_engine::ScriptEngine>) -> Self {
        ScriptEngineService { engine }
    }
}

#[tonic::async_trait]
impl proto::script_engine_server::ScriptEngine for ScriptEngineService {
    async fn get_all(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::GetAllStream>, tonic::Status> {
        let scripts = self.engine.scripts().await;
        Ok(tonic::Response::new(Box::pin(
            tokio_stream::iter(scripts)
                .map(|d| Result::<proto::Script, tonic::Status>::Ok(d.into())),
        ) as Self::GetAllStream))
    }

    type GetAllStream =
        Pin<Box<dyn Stream<Item = Result<proto::Script, tonic::Status>> + Send + 'static>>;
}

impl From<flaunch_core::script_engine::Script> for proto::Script {
    fn from(s: flaunch_core::script_engine::Script) -> Self {
        proto::Script {
            name: s.name,
            description: s.description,
            file: s.file.to_string_lossy().to_string(),
            interpreter: 0,
            arguments: s
                .arguments
                .into_iter()
                .map(Into::<proto::ScriptArgument>::into)
                .collect(),
        }
    }
}

impl From<(String, ArgumentType, String)> for proto::ScriptArgument {
    fn from(from: (String, ArgumentType, String)) -> Self {
        proto::ScriptArgument {
            name: from.0,
            argument_type: proto::ArgumentType::from(from.1) as i32,
            default: from.2,
        }
    }
}
impl From<flaunch_core::script_engine::ArgumentType> for proto::ArgumentType {
    fn from(from: flaunch_core::script_engine::ArgumentType) -> Self {
        match from {
            ArgumentType::Boolean(_) => proto::ArgumentType::Boolean,
            ArgumentType::Int(_) => proto::ArgumentType::Integer,
            ArgumentType::Uint(_) => proto::ArgumentType::Uinteger,
            ArgumentType::Float(_) => proto::ArgumentType::Float,
            ArgumentType::String(_) => proto::ArgumentType::String,
            ArgumentType::List(_) => proto::ArgumentType::List,
            ArgumentType::NotSpecified => proto::ArgumentType::Notspecified,
        }
    }
}
