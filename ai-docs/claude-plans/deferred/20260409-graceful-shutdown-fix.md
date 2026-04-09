# Fix Dangling Process After Clean Shutdown (Post-llama.cpp Query)

## Context

**Symptom**: After the user presses Ctrl+C on `make app.run`, the bodhi server logs `received Ctrl+C, stopping server` and `received signal to shutdown the server`, releases port 1135, but the `target/debug/bodhi` process never exits and remains dangling (`lsof -ti:1135` still lists its PID).

**Trigger condition**: The hang **only** occurs after llama.cpp has been queried at least once. If the user starts the server and Ctrl+Cs without issuing a chat completion, shutdown is clean. This pinpoints the root cause to resources that are first materialized during `LlamaServer::start()` / `StandaloneInferenceService::forward_local()`.

**Observed log tail** (from user report):
```
srv operator(): cleaning up before exit...
server_app::shutdown: received Ctrl+C, stopping server
server_app::server: received signal to shutdown the server
<hangs here forever>
```

Note the absence of any log AFTER "received signal to shutdown the server" — in particular, there is no log confirming `inference_service.stop()` completed. The hang is inside the graceful-shutdown callback chain, before `axum_server.await` can return.

**Intended outcome**: Clean `Ctrl+C` -> all tasks drain -> `runtime.block_on` returns -> `main()` exits, with no dangling process, regardless of whether llama.cpp was queried.

---

## Root Cause Analysis

### Primary cause — `llama_server_proc` uses fully-blocking `std::process` + detached OS threads

File: `crates/llama_server_proc/src/server.rs`

1. **`LlamaServer` uses `std::process::Command` / `std::process::Child`** (lines 10, 105, 221). The struct holds `Mutex<Option<Child>>` where `Child` is the std (blocking) variant.

2. **`monitor_output()` spawns two detached `std::thread::spawn` OS threads** (lines 142–168) that drive `BufReader::lines()` on `ChildStdout` / `ChildStderr` in a blocking loop. These threads:
   - Are **never** stored as `JoinHandle`s.
   - Have **no** cancellation mechanism.
   - Only exit when the pipe reaches EOF.

3. **`stop_unboxed()` calls `process.kill()?; process.wait()?;` synchronously from an `async fn`** (lines 245–256). `std::process::Child::wait()` is a blocking `waitpid()` issued directly on a tokio worker thread. Same for the `Drop` impl (lines 203–215).

**Why this only manifests after a query**: Before any query, `LlamaServer::start()` has not run, so no child, no pipes, and no detached reader threads exist. After the first `forward_local`, the reader threads are alive and pipes are held.

**How the hang composes**:
- On `Ctrl+C`, terminal SIGINT is delivered to the whole foreground process group — **including** llama.cpp (because the child was spawned without a new process group). llama.cpp starts its own SIGINT handler (`srv operator(): cleaning up before exit...`), which on Apple Metal can take noticeable time to release GPU/Metal residency sets.
- Meanwhile, bodhi's `ShutdownInferenceCallback::shutdown()` fires and reaches `LlamaServer::stop_unboxed()`, which calls `process.kill()` (SIGKILL) then **blocks in `process.wait()` on a tokio worker thread**. The async executor cannot preempt this call.
- The detached reader threads still hold the read ends of the pipes; even after SIGKILL reaps the child, there is no coordinated shutdown for them.
- Result: `inference_service.stop()` never returns, so the `axum::serve().with_graceful_shutdown(..)` closure never completes, so `axum_server.await?` never returns, so the tokio `join_handle` never resolves, so `aexecute()` never returns, so `runtime.block_on` never returns — process dangling.

### Contributing cause — detached `tokio::spawn` tasks hold `Arc<dyn SharedContext>`

Even once the primary cause is fixed, the following fire-and-forget tasks can extend runtime life:

1. `crates/server_core/src/standalone_inference.rs:47, 59, 93` — keep-alive timer + request-completion handlers use `tokio::spawn(async move { ... })` with no stored `JoinHandle` (line 59 stores one; lines 47 and 93 do not).
2. `crates/server_core/src/shared_rw.rs:289–298` — `notify_state_listeners` spawns a detached task per listener. Each holds a cloned `Arc<dyn ServerStateListener>` and runs on the runtime. With a small worker pool, these can pile up if shutdown races their creation.
3. `crates/server_app/src/serve.rs:72–74` — `KeepAliveSettingListener::on_change` spawns a detached task, which is unavoidable from a sync trait method but compounds the issue.

### Contributing cause — no runtime shutdown timeout

`crates/bodhi/src-tauri/src/server_init.rs:65–68` (and `native_init.rs:200–203`):
```rust
let runtime = Builder::new_multi_thread().enable_all().build()?;
runtime.block_on(async move { ... });
// runtime dropped here without shutdown_timeout()
```

There is no `runtime.shutdown_timeout(Duration::from_secs(N))` safety net. If any blocking task or detached reader thread traps a worker, the default drop behavior is to wait indefinitely for the worker threads to finish their current task.

---

## Recommended Fix

### 1. Migrate `llama_server_proc` to `tokio::process::*` (primary fix)

File: `crates/llama_server_proc/src/server.rs`

- Replace imports:
  - `std::process::{Child, ChildStderr, ChildStdout, Command, Stdio}` -> `tokio::process::{Child, ChildStderr, ChildStdout, Command}; std::process::Stdio`
  - Keep `std::sync::Mutex` removed; it was only guarding short critical sections — switch to `tokio::sync::Mutex` to avoid mixing sync-lock + async code, OR keep `std::sync::Mutex` only for the `Option<Child>` slot since we only `take()` under the lock (no `.await` while held).
  - Remove `std::thread`, `std::io::{BufRead, BufReader}`.
  - Add `tokio::io::{AsyncBufReadExt, BufReader}`.
  - Add `tokio::task::JoinHandle`.

- Extend `LlamaServer` with stored reader handles:
  ```rust
  pub struct LlamaServer {
    process: Mutex<Option<Child>>,
    stdout_task: Mutex<Option<JoinHandle<()>>>,
    stderr_task: Mutex<Option<JoinHandle<()>>>,
    // existing fields...
  }
  ```

- Rewrite `start()` (around line 219) using `tokio::process::Command`:
  ```rust
  let mut command = Command::new(&self.executable_path);
  command
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);               // belt-and-suspenders cleanup

  // Put the child in its own process group so terminal Ctrl+C
  // is NOT delivered to llama.cpp — only our explicit kill is.
  #[cfg(unix)]
  {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
  }

  let mut process = command.spawn()
    .map_err(|e| ServerError::StartupError(e.to_string()))?;
  let stdout = process.stdout.take();
  let stderr = process.stderr.take();
  ```

- Rewrite `monitor_output()` to use `tokio::spawn` and return handles:
  ```rust
  fn monitor_output(
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
  ) -> (Option<JoinHandle<()>>, Option<JoinHandle<()>>) {
    let stdout_handle = stdout.map(|out| {
      tokio::spawn(async move {
        let mut lines = BufReader::new(out).lines();
        while let Ok(Some(line)) = lines.next_line().await {
          debug!(target: "bodhi_server", "{}", line);
        }
      })
    });
    let stderr_handle = stderr.map(|err| {
      tokio::spawn(async move {
        let mut lines = BufReader::new(err).lines();
        while let Ok(Some(line)) = lines.next_line().await {
          warn!(target: "bodhi_server", "{}", line);
        }
      })
    });
    (stdout_handle, stderr_handle)
  }
  ```
  Store the returned handles in `self.stdout_task` / `self.stderr_task`.

- Rewrite `stop_unboxed()` (around line 245) to use async:
  ```rust
  async fn stop_unboxed(self) -> Result<()> {
    let process = self.process.lock().unwrap().take();
    if let Some(mut process) = process {
      let _ = process.start_kill();           // send SIGKILL, non-blocking
      let _ = process.wait().await;           // ASYNC wait, yields to runtime
    }
    // Drain log forwarders: kill closes pipes -> readers hit EOF quickly.
    if let Some(h) = self.stdout_task.lock().unwrap().take() {
      let _ = h.await;
    }
    if let Some(h) = self.stderr_task.lock().unwrap().take() {
      let _ = h.await;
    }
    Ok(())
  }
  ```

- Rewrite `Drop` impl. With `kill_on_drop(true)` set on the command, explicit kill in Drop becomes optional; the remaining concern is that `tokio::process::Child::drop()` must run on a tokio runtime context. Drop is a sync function — it can still call `start_kill()` (sync) but cannot `await wait()`. Acceptable: the reader tasks will be cancelled by the runtime; `kill_on_drop` flag ensures the child is reaped. Replace with:
  ```rust
  impl Drop for LlamaServer {
    fn drop(&mut self) {
      if let Some(mut process) = self.process.lock().unwrap().take() {
        let _ = process.start_kill();
      }
      if let Some(h) = self.stdout_task.lock().unwrap().take() { h.abort(); }
      if let Some(h) = self.stderr_task.lock().unwrap().take() { h.abort(); }
    }
  }
  ```

- `wait_for_server_ready()` already uses async reqwest; no changes.

### 2. Remove detached `tokio::spawn` in `notify_state_listeners`

File: `crates/server_core/src/shared_rw.rs:289–298`

Listeners already run via `#[async_trait]` methods. Await them sequentially — no benefit to spawning:
```rust
async fn notify_state_listeners(&self, state: ServerState) {
  let listeners = self.state_listeners.read().await;
  for listener in listeners.iter() {
    listener.on_state_change(state.clone()).await;
  }
}
```
This is backward-compatible semantically; listeners currently have no ordering guarantees.

### 3. Track keep-alive cleanup tasks in `StandaloneInferenceService`

File: `crates/server_core/src/standalone_inference.rs`

- Line 59 already stores its handle — no change needed beyond the fix below.
- Lines 47 and 93 are fire-and-forget stop-ctx tasks (used when `keep_alive == 0`). Convert them to **directly await** inside the calling function:
  - Line 87 `fn on_request_completed(&self)` currently cannot `.await` because it is sync. Change it to `async fn on_request_completed(&self)` and await it from `forward_local` (line 123). This is a local, contained change.
  - Inside `start_timer` at line 43, the `keep_alive == 0` branch also fires a detached stop. Either await directly, or (cleaner) hoist the "stop if keep_alive == 0" decision into `set_keep_alive` (the only caller path that reaches here with a mutation).
- In `stop()` at line 146, after `cancel_timer()`, await the taken handle (if any) rather than just `abort()`. Update `cancel_timer` to return the `JoinHandle`; in `stop()` do:
  ```rust
  async fn stop(&self) -> Result<(), InferenceError> {
    if let Some(handle) = self.take_timer_handle() {
      handle.abort();
      let _ = handle.await; // returns JoinError::Cancelled quickly
    }
    self.ctx.stop().await.map_err(|e| InferenceError::Internal(e.to_string()))
  }
  ```

### 4. Add runtime shutdown timeout as safety net

Files: `crates/bodhi/src-tauri/src/server_init.rs` and `crates/bodhi/src-tauri/src/native_init.rs`

Refactor from owned-runtime drop to explicit `shutdown_timeout`:
```rust
let runtime = Builder::new_multi_thread().enable_all().build()?;
let result: Result<(), AppSetupError> = runtime.block_on(async move { ... });
runtime.shutdown_timeout(std::time::Duration::from_secs(10));
result
```
This guarantees the process cannot dangle more than 10 seconds regardless of upstream bugs.

### Out of scope (not the bug the user is hitting)

- `RefreshWorker` in `crates/services/src/utils/queue_service.rs` — spawned at startup, blocks on `notify.notified().await`. The user reports **clean shutdown without a query**, so this is cancelled correctly by tokio's default task cancellation when the runtime is dropped. No change needed here for this issue. (It remains a latent design issue that can be tackled separately.)

---

## Critical Files to Modify

| # | File | Nature |
|---|------|--------|
| 1 | `crates/llama_server_proc/src/server.rs` | Migrate to `tokio::process`, store reader `JoinHandle`s, async `stop_unboxed`, `kill_on_drop(true)`, `process_group(0)` |
| 2 | `crates/llama_server_proc/Cargo.toml` | Ensure `tokio` features: `process`, `io-util` (likely `full` already) |
| 3 | `crates/server_core/src/shared_rw.rs` | `notify_state_listeners`: await sequentially instead of `tokio::spawn` |
| 4 | `crates/server_core/src/standalone_inference.rs` | Make `on_request_completed` async; remove detached spawns; `stop()` awaits aborted timer handle |
| 5 | `crates/bodhi/src-tauri/src/server_init.rs` | Add `runtime.shutdown_timeout(10s)` safety net |
| 6 | `crates/bodhi/src-tauri/src/native_init.rs` | Add `runtime.shutdown_timeout(10s)` safety net |

---

## Reused Code / Utilities

- `tokio::process::Command::kill_on_drop(bool)` — built-in tokio; no new dependency.
- `std::os::unix::process::CommandExt::process_group(0)` — stdlib, cfg(unix) gated.
- `tokio::io::AsyncBufReadExt::lines()` + `Lines::next_line()` — already in the dependency tree via tokio.
- `tokio::sync::RwLock` is already in use across `server_core`; reuse patterns from `DefaultSharedContext`.
- `tokio::task::JoinHandle::abort()` + `.await` pattern is already used at `standalone_inference.rs:59` (storing the timer handle) — generalize.

---

## Verification Plan

### Unit tests

1. **`crates/llama_server_proc/tests/test_live_server_proc.rs`** — existing live tests cover start/stop lifecycle with the bundled Llama-68M model. Ensure:
   - Start -> chat completion -> stop -> reader JoinHandles complete with `Ok(())`.
   - Drop impl reliably kills the child (add an assertion that `pgrep llama-server` returns empty within 2s of drop).

2. **New test: reader-task cleanup**
   File: `crates/llama_server_proc/tests/test_live_server_proc.rs`
   ```rust
   #[rstest]
   #[tokio::test]
   #[serial(live)]
   async fn test_server_stop_drains_log_readers(...) -> anyhow::Result<()> {
     let server = LlamaServer::new(...)?;
     server.start().await?;
     // Issue one chat completion to ensure llama.cpp has produced stdout/stderr.
     let _ = server.chat_completions(&sample_body()).await?;
     let before = task_count(); // see below
     server.stop_unboxed().await?;
     // Reader tasks must be joined, not leaked.
     assert!(task_count() <= before, "reader tasks leaked");
     Ok(())
   }
   ```
   (If no clean `task_count()` helper exists, assert via direct handle inspection: after `stop_unboxed().await?` returns, the `Mutex<Option<JoinHandle>>` slots should be `None`.)

### Integration tests

3. **`crates/server_app/tests/test_live_*.rs`** — existing live tests spin the full stack. Add:
   - `test_live_server_shutdown_after_query` — boot server, POST one completion, send SIGINT (via `tokio::signal::unix::SignalKind::interrupt().raise()` or directly invoke `handle.shutdown()`), assert the `ServerShutdownHandle::shutdown()` future resolves within 5 seconds with `Ok(())`.

4. **`cargo test -p llama_server_proc`** and **`cargo test -p server_core`** after each change (layered development: upstream first).

### Manual E2E (what the user actually did)

5. `make app.run`
6. In another terminal: `curl http://localhost:1135/v1/chat/completions -d '{...}'` (one completion, any loaded alias).
7. In the first terminal: `Ctrl+C`.
8. Expect:
   - `received Ctrl+C, stopping server`
   - `received signal to shutdown the server`
   - **`application exited with success`** (or equivalent) log
   - Shell prompt returns in <5 seconds.
9. Verify: `ps -eaf | grep bodhi` shows no dangling process; `lsof -ti:1135` returns empty.
10. Re-run without a query and confirm the already-clean path still works.

### Gate checks (per user's standing preference)

- `cargo check -p llama_server_proc` after changes
- `make test.backend`
- `make build.ui-rebuild`
- Playwright E2E: `cd crates/lib_bodhiserver_napi && npm run test:playwright` — smoke suite to catch any regression in server startup/shutdown paths used by the NAPI harness.

---

## Notes / Open Questions

1. **`process_group(0)` behavior**: Putting llama.cpp in its own process group means terminal Ctrl+C will no longer hit llama.cpp directly. The "cleaning up before exit..." log currently comes from llama.cpp's own SIGINT handler; after this change, that message may disappear and llama.cpp will die via SIGKILL instead. This is the correct behavior — the parent should be the sole owner of the child's lifecycle — but worth noting in case anyone relies on seeing that log.

2. **`tokio::sync::Mutex` vs `std::sync::Mutex` for the Child slot**: `std::sync::Mutex` is fine if we only `take()` inside the lock (no `.await` while holding). Keep `std::sync::Mutex` to minimize churn; the audited code paths (`stop_unboxed`, `Drop`) do not await while the guard is live.

3. **Windows behavior**: `process_group(0)` is Unix-only. On Windows, tokio spawns children in the same console group by default; the existing `kill_on_drop(true)` + explicit `start_kill()` path already handles Windows correctly. No extra work needed for the reported macOS/Linux bug.
