use std::{
    ffi::{CStr, CString},
    fs,
    net::TcpListener,
    ptr,
    sync::Mutex,
    thread,
    time::Duration,
};

use crate::{
    moira_runtime_force_kill_json, moira_runtime_load_profile_draft_json,
    moira_runtime_load_profile_json, moira_runtime_loom_json,
    moira_runtime_register_known_local_build_json, moira_runtime_save_profile_draft_json,
    moira_runtime_save_profile_json, moira_runtime_shutdown_json, moira_runtime_status_json,
    moira_runtime_stop_json, moira_runtime_string_free, moira_runtime_wake_json,
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

#[test]
fn profile_json_save_and_load_round_trip() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!("moira-ffi-profile-test-{}", std::process::id()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let profile = CString::new(r#"{"profileId":"default","contents":"{\n  // local\n}"}"#)
        .expect("profile request is c string");
    let mut save_json = ptr::null_mut();
    let mut save_error = ptr::null_mut();

    let save_code = moira_runtime_save_profile_json(
        root.as_ptr(),
        bind.as_ptr(),
        profile.as_ptr(),
        &mut save_json,
        &mut save_error,
    );

    assert_eq!(save_code, 0);
    assert!(save_error.is_null());
    assert!(!save_json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(save_json).to_string_lossy().to_string();
        assert!(json_text.contains("\"profileId\":\"default\""));
        assert!(json_text.contains("\"contents\""));
        moira_runtime_string_free(save_json);
    }

    let profile_id = CString::new("default").expect("profile id is c string");
    let mut load_json = ptr::null_mut();
    let mut load_error = ptr::null_mut();
    let load_code = moira_runtime_load_profile_json(
        root.as_ptr(),
        bind.as_ptr(),
        profile_id.as_ptr(),
        &mut load_json,
        &mut load_error,
    );

    assert_eq!(load_code, 0);
    assert!(load_error.is_null());
    assert!(!load_json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(load_json).to_string_lossy().to_string();
        assert!(json_text.contains("\"profileId\":\"default\""));
        assert!(json_text.contains("// local"));
        moira_runtime_string_free(load_json);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}

#[test]
fn profile_draft_json_save_and_load_round_trip_environment_sources() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!(
        "moira-ffi-profile-draft-test-{}",
        std::process::id()
    ));
    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let draft = CString::new(
        r#"{
            "profileId": "default",
            "coreConfig": "{ logging: { dir: \"./logs\" } }",
            "envFiles": [{ "path": "./local.env", "required": false }],
            "inlineEnvironment": [{ "key": "OPENAI_API_KEY", "value": "inline-openai" }]
        }"#,
    )
    .expect("draft request is c string");
    let mut save_json = ptr::null_mut();
    let mut save_error = ptr::null_mut();

    let save_code = moira_runtime_save_profile_draft_json(
        root.as_ptr(),
        bind.as_ptr(),
        draft.as_ptr(),
        &mut save_json,
        &mut save_error,
    );

    assert_eq!(save_code, 0);
    assert!(save_error.is_null());
    assert!(!save_json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(save_json).to_string_lossy().to_string();
        assert!(json_text.contains("\"profileId\":\"default\""));
        assert!(json_text.contains("\"coreConfig\""));
        assert!(json_text.contains("\"envFiles\""));
        assert!(json_text.contains("\"inlineEnvironment\""));
        moira_runtime_string_free(save_json);
    }

    let profile_id = CString::new("default").expect("profile id is c string");
    let mut load_json = ptr::null_mut();
    let mut load_error = ptr::null_mut();
    let load_code = moira_runtime_load_profile_draft_json(
        root.as_ptr(),
        bind.as_ptr(),
        profile_id.as_ptr(),
        &mut load_json,
        &mut load_error,
    );

    assert_eq!(load_code, 0);
    assert!(load_error.is_null());
    assert!(!load_json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(load_json).to_string_lossy().to_string();
        assert!(json_text.contains("\"path\":\"./local.env\""));
        assert!(json_text.contains("\"key\":\"OPENAI_API_KEY\""));
        assert!(json_text.contains("\"value\":\"inline-openai\""));
        moira_runtime_string_free(load_json);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}

#[test]
fn register_known_local_build_json_updates_launch_targets() {
    let _guard = TEST_RUNTIME_LOCK
        .lock()
        .expect("test lock should be available");
    let root = std::env::temp_dir().join(format!("moira-ffi-target-test-{}", std::process::id()));
    let build_dir = root.join("fixture-build");
    let executable_path = build_dir.join("beluna");
    fs::create_dir_all(&build_dir).expect("build dir should exist");
    fs::write(&executable_path, "#!/bin/sh\n").expect("build executable should write");

    let listener = TcpListener::bind("127.0.0.1:0").expect("free port should bind");
    let bind = listener.local_addr().expect("free port should have addr");
    drop(listener);

    let root = CString::new(root.to_string_lossy().to_string()).expect("root is c string");
    let bind = CString::new(bind.to_string()).expect("bind is c string");
    let registration = CString::new(format!(
        r#"{{"buildId":"dev-core","executablePath":"{}","workingDir":"{}"}}"#,
        executable_path.display(),
        build_dir.display()
    ))
    .expect("registration request is c string");
    let mut json = ptr::null_mut();
    let mut error = ptr::null_mut();

    let code = moira_runtime_register_known_local_build_json(
        root.as_ptr(),
        bind.as_ptr(),
        registration.as_ptr(),
        &mut json,
        &mut error,
    );

    assert_eq!(code, 0);
    assert!(error.is_null());
    assert!(!json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(json).to_string_lossy().to_string();
        assert!(json_text.contains("\"kind\":\"knownLocalBuild\""));
        assert!(json_text.contains("\"buildId\":\"dev-core\""));
        moira_runtime_string_free(json);
    }

    let selected_run = CString::new("").expect("run selection is c string");
    let selected_tick = CString::new("").expect("tick selection is c string");
    let mut loom_json = ptr::null_mut();
    let mut loom_error = ptr::null_mut();
    let loom_code = moira_runtime_loom_json(
        root.as_ptr(),
        bind.as_ptr(),
        selected_run.as_ptr(),
        selected_tick.as_ptr(),
        &mut loom_json,
        &mut loom_error,
    );

    assert_eq!(loom_code, 0);
    assert!(loom_error.is_null());
    assert!(!loom_json.is_null());

    unsafe {
        let json_text = CStr::from_ptr(loom_json).to_string_lossy().to_string();
        assert!(json_text.contains("\"buildId\":\"dev-core\""));
        moira_runtime_string_free(loom_json);
    }

    let mut shutdown_json = ptr::null_mut();
    let mut shutdown_error = ptr::null_mut();
    let shutdown_code = moira_runtime_shutdown_json(&mut shutdown_json, &mut shutdown_error);
    assert_eq!(shutdown_code, 0);
    moira_runtime_string_free(shutdown_json);
    moira_runtime_string_free(shutdown_error);
}
