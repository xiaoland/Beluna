use std::{
    ffi::{CStr, CString},
    net::TcpListener,
    ptr,
    sync::Mutex,
    thread,
    time::Duration,
};

use crate::{
    moira_runtime_force_kill_json, moira_runtime_loom_json, moira_runtime_shutdown_json,
    moira_runtime_status_json, moira_runtime_stop_json, moira_runtime_string_free,
    moira_runtime_wake_json,
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

#[test]
fn wake_json_accepts_typed_request_and_reports_missing_target() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!("moira-ffi-wake-test-{}", std::process::id()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    wait_for_receiver_ready(&root, &bind);
    let wake = CString::new(
        r#"{"target":{"kind":"knownLocalBuild","buildId":"missing-core"},"profile":null}"#,
    )
    .expect("wake request is c string");
    let mut json = ptr::null_mut();
    let mut error = ptr::null_mut();

    let code = moira_runtime_wake_json(
        root.as_ptr(),
        bind.as_ptr(),
        wake.as_ptr(),
        &mut json,
        &mut error,
    );

    assert_eq!(code, 1);
    assert!(json.is_null());
    assert!(!error.is_null());

    unsafe {
        let error_text = CStr::from_ptr(error).to_string_lossy().to_string();
        assert!(error_text.contains("missing-core"));
        moira_runtime_string_free(error);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}

fn wait_for_receiver_ready(root: &CString, bind: &CString) {
    for _ in 0..25 {
        let mut json = ptr::null_mut();
        let mut error = ptr::null_mut();
        let code = moira_runtime_status_json(root.as_ptr(), bind.as_ptr(), &mut json, &mut error);

        let ready = if code == 0 && !json.is_null() {
            unsafe {
                let json_text = CStr::from_ptr(json).to_string_lossy().to_string();
                json_text.contains(r#""wakeState":"listening""#)
                    || json_text.contains(r#""wakeState":"awake""#)
            }
        } else {
            false
        };

        moira_runtime_string_free(json);
        moira_runtime_string_free(error);

        if ready {
            return;
        }

        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn core_control_json_reports_no_running_core() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!(
        "moira-ffi-core-control-test-{}",
        std::process::id()
    ));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let mut stop_json = ptr::null_mut();
    let mut stop_error = ptr::null_mut();

    let stop_code = moira_runtime_stop_json(
        root.as_ptr(),
        bind.as_ptr(),
        &mut stop_json,
        &mut stop_error,
    );

    assert_eq!(stop_code, 1);
    assert!(stop_json.is_null());
    assert!(!stop_error.is_null());

    unsafe {
        let error_text = CStr::from_ptr(stop_error).to_string_lossy().to_string();
        assert!(error_text.contains("no supervised Core"));
        moira_runtime_string_free(stop_error);
    }

    let mut kill_json = ptr::null_mut();
    let mut kill_error = ptr::null_mut();
    let kill_code = moira_runtime_force_kill_json(
        root.as_ptr(),
        bind.as_ptr(),
        &mut kill_json,
        &mut kill_error,
    );

    assert_eq!(kill_code, 1);
    assert!(kill_json.is_null());
    assert!(!kill_error.is_null());

    unsafe {
        let error_text = CStr::from_ptr(kill_error).to_string_lossy().to_string();
        assert!(error_text.contains("no supervised Core"));
        moira_runtime_string_free(kill_error);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}
