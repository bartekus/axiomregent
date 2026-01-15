use tonic::{transport::Server, Request, Response, Status};
use self::encore::daemon::daemon_server::{Daemon, DaemonServer};
use self::encore::daemon::{CheckRequest, CommandMessage};

pub mod encore {
    pub mod daemon {
        tonic::include_proto!("encore.daemon");
    }
}

#[derive(Default)]
pub struct MyDaemon {}


#[tonic::async_trait]
impl Daemon for MyDaemon {
    type CheckStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type RunStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type TestStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type ExecScriptStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type ExportStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type DBProxyStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;
    type DBResetStream = tokio_stream::wrappers::ReceiverStream<Result<CommandMessage, Status>>;

    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<Self::CheckStream>, Status> {
        println!("Check called with: {:?}", request);

        // Example: send a single CommandMessage and close the stream
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let msg = CommandMessage {
            msg: Some(encore::daemon::command_message::Msg::Output(
                encore::daemon::CommandOutput {
                    stdout: b"Check completed successfully\n".to_vec(),
                    stderr: vec![],
                },
            )),
        };
        tx.send(Ok(msg)).await.unwrap();

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn run(
        &self,
        _request: Request<encore::daemon::RunRequest>,
    ) -> Result<Response<Self::RunStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn test(
        &self,
        _request: Request<encore::daemon::TestRequest>,
    ) -> Result<Response<Self::TestStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn test_spec(
        &self,
        _request: Request<encore::daemon::TestSpecRequest>,
    ) -> Result<Response<encore::daemon::TestSpecResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn exec_script(
        &self,
        _request: Request<encore::daemon::ExecScriptRequest>,
    ) -> Result<Response<Self::ExecScriptStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn export(
        &self,
        _request: Request<encore::daemon::ExportRequest>,
    ) -> Result<Response<Self::ExportStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn db_connect(
        &self,
        _request: Request<encore::daemon::DbConnectRequest>,
    ) -> Result<Response<encore::daemon::DbConnectResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn db_proxy(
        &self,
        _request: Request<encore::daemon::DbProxyRequest>,
    ) -> Result<Response<Self::DBProxyStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn db_reset(
        &self,
        _request: Request<encore::daemon::DbResetRequest>,
    ) -> Result<Response<Self::DBResetStream>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn gen_client(
        &self,
        _request: Request<encore::daemon::GenClientRequest>,
    ) -> Result<Response<encore::daemon::GenClientResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn gen_wrappers(
        &self,
        _request: Request<encore::daemon::GenWrappersRequest>,
    ) -> Result<Response<encore::daemon::GenWrappersResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn secrets_refresh(
        &self,
        _request: Request<encore::daemon::SecretsRefreshRequest>,
    ) -> Result<Response<encore::daemon::SecretsRefreshResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn version(
        &self,
        _request: Request<()>,
    ) -> Result<Response<encore::daemon::VersionResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_namespace(
        &self,
        _request: Request<encore::daemon::CreateNamespaceRequest>,
    ) -> Result<Response<encore::daemon::Namespace>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn switch_namespace(
        &self,
        _request: Request<encore::daemon::SwitchNamespaceRequest>,
    ) -> Result<Response<encore::daemon::Namespace>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_namespaces(
        &self,
        _request: Request<encore::daemon::ListNamespacesRequest>,
    ) -> Result<Response<encore::daemon::ListNamespacesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn delete_namespace(
        &self,
        _request: Request<encore::daemon::DeleteNamespaceRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn dump_meta(
        &self,
        _request: Request<encore::daemon::DumpMetaRequest>,
    ) -> Result<Response<encore::daemon::DumpMetaResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn telemetry(
        &self,
        _request: Request<encore::daemon::TelemetryConfig>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_app(
        &self,
        _request: Request<encore::daemon::CreateAppRequest>,
    ) -> Result<Response<encore::daemon::CreateAppResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let daemon = MyDaemon::default();

    println!("DaemonServer listening on {addr}");

    Server::builder()
        .add_service(DaemonServer::new(daemon))
        .serve(addr)
        .await?;

    Ok(())
}