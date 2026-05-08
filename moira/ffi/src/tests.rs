use std::{
    ffi::{CStr, CString},
    net::TcpListener,
    ptr,
    sync::Mutex,
};

use crate::{
    moira_runtime_loom_json, moira_runtime_shutdown_json, moira_runtime_status_json,
    moira_runtime_string_free,
};

static TEST_RUNTIME_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn status_json_opens_runtime_and_returns_snapshot() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!("moira-ffi-test-{}", std::process::id()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let mut json = ptr::null_mut();
    let mut error = ptr::null_mut();

    let code = moira_runtime_status_json(root.as_ptr(), bind.as_ptr(), &mut json, &mut error);

    assert_eq!(code, 0);
    assert!(error.is_null());
    assert!(!json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(json).to_string_lossy().to_string();
        assert!(json_text.contains("\"lifecycle\""));
        moira_runtime_string_free(json);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}

#[test]
fn loom_json_returns_minimum_operator_snapshot() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!("moira-ffi-loom-test-{}", std::process::id()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let selected_run = CString::new("").expect("run selection is c string");
    let selected_tick = CString::new("").expect("tick selection is c string");
    let mut json = ptr::null_mut();
    let mut error = ptr::null_mut();

    let code = moira_runtime_loom_json(
        root.as_ptr(),
        bind.as_ptr(),
        selected_run.as_ptr(),
        selected_tick.as_ptr(),
        &mut json,
        &mut error,
    );

    assert_eq!(code, 0);
    assert!(error.is_null());
    assert!(!json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(json).to_string_lossy().to_string();
        assert!(json_text.contains("\"status\""));
        assert!(json_text.contains("\"launchTargets\""));
        assert!(json_text.contains("\"runs\""));
        assert!(json_text.contains("\"tickDetail\""));
        moira_runtime_string_free(json);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}
