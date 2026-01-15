use self::encore::daemon::daemon_client::DaemonClient;
use self::encore::daemon::CheckRequest;

pub mod encore {
    pub mod daemon {
        tonic::include_proto!("encore.daemon");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the daemon server (adjust the address as needed)
    let mut client = DaemonClient::connect("http://[::1]:50051").await?;

    // Example: Send a CheckRequest
    let request = tonic::Request::new(CheckRequest {
        app_root: "/path/to/app".to_string(),
        working_dir: ".".to_string(),
        codegen_debug: false,
        parse_tests: false,
        environ: vec![],
    });

    let mut response = client.check(request).await?.into_inner();

    // Stream the responses
    while let Some(msg) = response.message().await? {
        println!("Received: {:?}", msg);
    }

    Ok(())
}