use std::{
    ffi::{CStr, CString},
    net::SocketAddr,
    os::raw::c_char,
    path::PathBuf,
    ptr,
    sync::{Arc, Mutex},
};

use moira_runtime::{
    MoiraPaths, MoiraRuntime, MoiraRuntimeConfig, NoopEventSink, TokioTaskSpawner,
};

static RUNTIME: Mutex<Option<FfiRuntime>> = Mutex::new(None);

struct FfiRuntime {
    root_dir: PathBuf,
    receiver_bind: SocketAddr,
    tokio: tokio::runtime::Runtime,
    runtime: Arc<MoiraRuntime>,
}

#[unsafe(no_mangle)]
pub extern "C" fn moira_runtime_status_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match status_json(root_dir, receiver_bind) {
        Ok(json) => {
            write_out(out_json, json);
            0
        }
        Err(error) => {
            write_out(out_error, error);
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moira_runtime_shutdown_json(
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match shutdown_json() {
        Ok(json) => {
            write_out(out_json, json);
            0
        }
        Err(error) => {
            write_out(out_error, error);
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moira_runtime_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(value);
    }
}

fn status_json(root_dir: *const c_char, receiver_bind: *const c_char) -> Result<String, String> {
    let root_dir = read_path(root_dir, "root_dir")?;
    let receiver_bind = read_socket_addr(receiver_bind, "receiver_bind")?;
    let mut guard = RUNTIME
        .lock()
        .map_err(|_| "failed to lock Moira runtime state".to_string())?;

    if let Some(runtime) = guard.as_ref() {
        if runtime.root_dir != root_dir || runtime.receiver_bind != receiver_bind {
            return Err(format!(
                "Moira runtime is already open at `{}` on `{}`",
                runtime.root_dir.display(),
                runtime.receiver_bind
            ));
        }
    } else {
        let tokio = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| format!("failed to create Moira async runtime: {err}"))?;
        let runtime = tokio
            .block_on(MoiraRuntime::open(MoiraRuntimeConfig {
                paths: MoiraPaths::from_root(root_dir.clone()),
                receiver_bind,
                event_sink: Arc::new(NoopEventSink),
                task_spawner: Arc::new(TokioTaskSpawner),
            }))
            .map_err(|err| err.to_string())?;

        *guard = Some(FfiRuntime {
            root_dir,
            receiver_bind,
            tokio,
            runtime,
        });
    }

    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let status = runtime
        .tokio
        .block_on(runtime.runtime.status())
        .map_err(|err| err.to_string())?;

    serde_json::to_string(&status)
        .map_err(|err| format!("failed to encode Moira runtime status: {err}"))
}

fn shutdown_json() -> Result<String, String> {
    let runtime = RUNTIME
        .lock()
        .map_err(|_| "failed to lock Moira runtime state".to_string())?
        .take();

    let Some(runtime) = runtime else {
        return Ok("null".to_string());
    };

    let outcome = runtime
        .tokio
        .block_on(runtime.runtime.shutdown())
        .map_err(|err| err.to_string())?;

    serde_json::to_string(&outcome.resources)
        .map_err(|err| format!("failed to encode Moira shutdown outcome: {err}"))
}

fn read_path(value: *const c_char, label: &str) -> Result<PathBuf, String> {
    let value = read_string(value, label)?;
    if value.trim().is_empty() {
        return Err(format!("{label} cannot be empty"));
    }
    Ok(PathBuf::from(value))
}

fn read_socket_addr(value: *const c_char, label: &str) -> Result<SocketAddr, String> {
    let value = read_string(value, label)?;
    value
        .parse::<SocketAddr>()
        .map_err(|err| format!("invalid {label} `{value}`: {err}"))
}

fn read_string(value: *const c_char, label: &str) -> Result<String, String> {
    if value.is_null() {
        return Err(format!("{label} cannot be null"));
    }

    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map(|value| value.to_string())
        .map_err(|err| format!("{label} must be valid UTF-8: {err}"))
}

fn clear_out(out: *mut *mut c_char) {
    if out.is_null() {
        return;
    }

    unsafe {
        *out = ptr::null_mut();
    }
}

fn write_out(out: *mut *mut c_char, value: String) {
    if out.is_null() {
        return;
    }

    let Ok(value) = CString::new(value) else {
        return;
    };

    unsafe {
        *out = value.into_raw();
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::CString, net::TcpListener, ptr};

    use super::{
        moira_runtime_shutdown_json, moira_runtime_status_json, moira_runtime_string_free,
    };

    #[test]
    fn status_json_opens_runtime_and_returns_snapshot() {
        let root = std::env::temp_dir().join(format!(
            "moira-ffi-test-{}",
            std::process::id()
        ));
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
            let json_text = std::ffi::CStr::from_ptr(json)
                .to_string_lossy()
                .to_string();
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
}
