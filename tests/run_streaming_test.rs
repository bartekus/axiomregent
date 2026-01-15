// tests/run_streaming_test.rs

use axiomregent::run_tools::RunTools;
use std::thread;
use std::time::Duration;

#[test]
fn test_run_lifecycle_and_streaming() {
    let dir = tempfile::tempdir().unwrap();
    let tools = RunTools::new(dir.path());

    // 1. Execute a non-existent skill
    let run_id = tools
        .execute("non-existent-skill".to_string(), None)
        .unwrap();

    // 2. Wait for it to fail
    // We poll status until it's failed or completed, with a timeout
    let mut status_json = serde_json::Value::Null;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        status_json = tools.status(&run_id).unwrap();
        let status_str = status_json["status"].as_str().unwrap();
        if status_str == "failed" || status_str == "completed" {
            break;
        }
    }

    assert_eq!(status_json["status"], "failed");
    assert!(status_json["start_time"].is_string());

    // end_time might be null if it hasn't finished (shouldn't happen with our polling)
    // or string if finished.
    assert!(status_json["end_time"].is_string());
    assert!(status_json["exit_code"].is_number()); // Should be integer 1

    // 4. Check logs (full)
    // Since skill failed to start, the log might contain "Failed to create log file" (no this is logged to app log), implies log file creation failed?
    // Wait, the failure we see is "failed" status.
    // In code:
    // If File::create fails -> status=failed, end_time set. (Log file creation error is logged to error!(), no log file).
    // If File::create succeeds -> it runs.
    // If run_specific returns Err -> status=failed.
    // run_specific("non-existent") likely returns Err or Ok(false).
    // So log file SHOULD exist and contain something if Runner wrote to it.

    let _logs = tools.logs(&run_id, None, None).unwrap();
    // Verify we got something or at least empty string (if file exists).
    // The logs tool returns "Run ID not found" if run not found, or content.
    // If file doesn't exist (creation failed), it returns "".
    // We expect file creation to succeed (temp dir).

    // Since we don't know what Runner writes on missing skill, we just check implementation logic of logs method.
    // We cannot verify content easily without a known skill.

    // Let's manually write to the log file to verify streaming logic!
    // We can't access logs_path directly as it's private in RunContext.
    // But we know the path structure: .axiomregent/run/logs/<run_id>.log
    let logs_path = dir
        .path()
        .join(".axiomregent/run/logs")
        .join(format!("{}.log", run_id));
    if !logs_path.exists() {
        // If runner hasn't created it yet? It should have.
        // Or if runner crashed.
        // Let's write our own content to ensure we test the streaming logic.
        std::fs::write(&logs_path, "1234567890").unwrap();
    } else {
        // Append known content
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&logs_path)
            .unwrap();
        write!(f, "1234567890").unwrap();
    }

    // Now test streaming of "1234567890" (plus potentially whatever runner wrote)
    // Let's write a large known string to be unique.
    let unique_str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    std::fs::write(&logs_path, unique_str).unwrap(); // Overwrite for clarity

    // 5. Check logs with limit
    let limit = 5;
    let partial_logs = tools.logs(&run_id, None, Some(limit)).unwrap();
    assert_eq!(partial_logs.len(), limit as usize);
    assert_eq!(partial_logs, "ABCDE");

    // 6. Check logs with offset
    let offset = 2;
    // Expected: CDEFGHIJKLMNOPQRSTUVWXYZ
    let offset_logs = tools.logs(&run_id, Some(offset), None).unwrap();
    assert_eq!(offset_logs, "CDEFGHIJKLMNOPQRSTUVWXYZ");

    // 7. Check logs with offset AND limit
    let limit2 = 3;
    // Offset 2 ("C"), limit 3 -> "CDE"
    let slice = tools.logs(&run_id, Some(offset), Some(limit2)).unwrap();
    assert_eq!(slice, "CDE");
}
