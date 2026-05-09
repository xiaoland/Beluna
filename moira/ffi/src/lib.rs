use std::{
    ffi::{CStr, CString},
    net::SocketAddr,
    os::raw::c_char,
    path::PathBuf,
    ptr,
    sync::{Arc, Mutex, MutexGuard},
};

use moira_runtime::{
    MoiraLoomSelection, MoiraPaths, MoiraRuntime, MoiraRuntimeConfig, NoopEventSink,
    TokioTaskSpawner,
    clotho::model::{
        KnownLocalBuildRegistration, ProfileRef, SaveProfileDocumentRequest,
        SaveProfileDraftRequest, WakeInputRequest,
    },
};

static RUNTIME: Mutex<Option<FfiRuntime>> = Mutex::new(None);

#[cfg(test)]
mod tests;

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
pub extern "C" fn moira_runtime_loom_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    selected_run_id: *const c_char,
    selected_tick: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match loom_json(root_dir, receiver_bind, selected_run_id, selected_tick) {
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
pub extern "C" fn moira_runtime_wake_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    wake_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match wake_json_operation(root_dir, receiver_bind, wake_json) {
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
pub extern "C" fn moira_runtime_stop_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match stop_json(root_dir, receiver_bind) {
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
pub extern "C" fn moira_runtime_force_kill_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match force_kill_json(root_dir, receiver_bind) {
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
pub extern "C" fn moira_runtime_load_profile_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_id: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match load_profile_json(root_dir, receiver_bind, profile_id) {
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
pub extern "C" fn moira_runtime_save_profile_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match save_profile_json(root_dir, receiver_bind, profile_json) {
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
pub extern "C" fn moira_runtime_load_profile_draft_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_id: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match load_profile_draft_json(root_dir, receiver_bind, profile_id) {
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
pub extern "C" fn moira_runtime_save_profile_draft_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    draft_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match save_profile_draft_json(root_dir, receiver_bind, draft_json) {
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
pub extern "C" fn moira_runtime_register_known_local_build_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    registration_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> i32 {
    clear_out(out_json);
    clear_out(out_error);

    match register_known_local_build_json(root_dir, receiver_bind, registration_json) {
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
    let guard = ensure_runtime(root_dir, receiver_bind)?;
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

fn loom_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    selected_run_id: *const c_char,
    selected_tick: *const c_char,
) -> Result<String, String> {
    let selected_run_id = read_optional_string(selected_run_id, "selected_run_id")?;
    let selected_tick = read_optional_u64(selected_tick, "selected_tick")?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let snapshot = runtime
        .tokio
        .block_on(runtime.runtime.loom_snapshot(MoiraLoomSelection {
            run_id: selected_run_id,
            tick: selected_tick,
        }))
        .map_err(|err| err.to_string())?;

    serde_json::to_string(&snapshot)
        .map_err(|err| format!("failed to encode Moira Loom snapshot: {err}"))
}

fn wake_json_operation(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    wake_json: *const c_char,
) -> Result<String, String> {
    let request_json = read_string(wake_json, "wake_json")?;
    let request = serde_json::from_str::<WakeInputRequest>(&request_json)
        .map_err(|err| format!("failed to decode Moira wake request: {err}"))?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let status = runtime
        .tokio
        .block_on(runtime.runtime.atropos().wake(request))?;

    serde_json::to_string(&status)
        .map_err(|err| format!("failed to encode Moira Core status: {err}"))
}

fn stop_json(root_dir: *const c_char, receiver_bind: *const c_char) -> Result<String, String> {
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let status = runtime.tokio.block_on(runtime.runtime.atropos().stop())?;

    serde_json::to_string(&status)
        .map_err(|err| format!("failed to encode Moira Core status: {err}"))
}

fn force_kill_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
) -> Result<String, String> {
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let status = runtime
        .tokio
        .block_on(runtime.runtime.atropos().force_kill())?;

    serde_json::to_string(&status)
        .map_err(|err| format!("failed to encode Moira Core status: {err}"))
}

fn load_profile_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_id: *const c_char,
) -> Result<String, String> {
    let profile_id = read_string(profile_id, "profile_id")?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let document = runtime
        .runtime
        .clotho()
        .load_profile_document(&ProfileRef { profile_id })?;

    serde_json::to_string(&document)
        .map_err(|err| format!("failed to encode Moira profile document: {err}"))
}

fn save_profile_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_json: *const c_char,
) -> Result<String, String> {
    let request_json = read_string(profile_json, "profile_json")?;
    let request = serde_json::from_str::<SaveProfileDocumentRequest>(&request_json)
        .map_err(|err| format!("failed to decode Moira profile save request: {err}"))?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let document = runtime.runtime.clotho().save_profile_document(request)?;

    serde_json::to_string(&document)
        .map_err(|err| format!("failed to encode Moira profile document: {err}"))
}

fn load_profile_draft_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    profile_id: *const c_char,
) -> Result<String, String> {
    let profile_id = read_string(profile_id, "profile_id")?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let document = runtime
        .runtime
        .clotho()
        .load_profile_draft(&ProfileRef { profile_id })?;

    serde_json::to_string(&document)
        .map_err(|err| format!("failed to encode Moira profile draft: {err}"))
}

fn save_profile_draft_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    draft_json: *const c_char,
) -> Result<String, String> {
    let request_json = read_string(draft_json, "draft_json")?;
    let request = serde_json::from_str::<SaveProfileDraftRequest>(&request_json)
        .map_err(|err| format!("failed to decode Moira profile draft save request: {err}"))?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let document = runtime.runtime.clotho().save_profile_draft(request)?;

    serde_json::to_string(&document)
        .map_err(|err| format!("failed to encode Moira profile draft: {err}"))
}

fn register_known_local_build_json(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
    registration_json: *const c_char,
) -> Result<String, String> {
    let request_json = read_string(registration_json, "registration_json")?;
    let registration = serde_json::from_str::<KnownLocalBuildRegistration>(&request_json)
        .map_err(|err| format!("failed to decode Moira local build registration: {err}"))?;
    let guard = ensure_runtime(root_dir, receiver_bind)?;
    let runtime = guard
        .as_ref()
        .ok_or_else(|| "Moira runtime failed to initialize".to_string())?;
    let target = runtime
        .runtime
        .clotho()
        .register_known_local_build(registration)?;

    serde_json::to_string(&target)
        .map_err(|err| format!("failed to encode Moira launch target ref: {err}"))
}

fn ensure_runtime(
    root_dir: *const c_char,
    receiver_bind: *const c_char,
) -> Result<MutexGuard<'static, Option<FfiRuntime>>, String> {
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

    Ok(guard)
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

fn read_optional_string(value: *const c_char, label: &str) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(None);
    }

    let value = read_string(value, label)?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn read_optional_u64(value: *const c_char, label: &str) -> Result<Option<u64>, String> {
    let Some(value) = read_optional_string(value, label)? else {
        return Ok(None);
    };

    value
        .parse::<u64>()
        .map(Some)
        .map_err(|err| format!("invalid {label} `{value}`: {err}"))
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
