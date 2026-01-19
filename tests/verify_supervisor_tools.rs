use axiomregent::supervisor::SupervisorHandle;
use axiomregent::supervisor::buffer::LogBuffer;
use axiomregent::supervisor::tools::SupervisorTools;
use std::sync::Arc;

#[tokio::test]
async fn verify_supervisor_tools() {
    let log_buffer = Arc::new(LogBuffer::new(100));
    let (handle, _rx) = SupervisorHandle::new(log_buffer.clone());

    // Set some state safely
    {
        let mut s = handle.state.write().unwrap();
        s.state = axiomregent::supervisor::state::State::Starting;
    }

    let tools = SupervisorTools::new(handle);

    // Check Status
    let status_val = tools.status().unwrap();
    let status_obj = status_val.as_object().unwrap();
    assert_eq!(
        status_obj.get("state").unwrap().as_str().unwrap(),
        "starting"
    );

    // Check Logs
    log_buffer.push("test log line".to_string());
    let logs_val = tools.logs(10, 0).unwrap();
    let logs_arr = logs_val.get("logs").unwrap().as_array().unwrap();
    assert_eq!(logs_arr.len(), 1);
    assert_eq!(logs_arr[0].as_str().unwrap(), "test log line");

    // Restart
    let restart_res = tools.restart(false);
    assert!(restart_res.is_ok());
}
