# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Workspace layout

Cargo workspace (resolver = "2") with two members declared in the root `Cargo.toml`:

- `gemini-sdk/` — async library crate. `src/lib.rs` re-exports everything from `client`, `types`, `error`. Public surface: `GeminiClient`, `GeminiClientTrait`, `GeminiError`, plus everything in `types.rs` via `pub use types::*`.
- `gemini-cli/` — clap-based binary built on the SDK (`src/main.rs`).

All shared dep versions live in `[workspace.dependencies]` at the root; member crates use `{ workspace = true }`. Add new shared deps at the root first.

## Commands

```bash
cargo check                                       # fast type-check
cargo build                                       # build whole workspace
cargo build -p gemini-sdk                         # build one member
cargo test                                        # run all tests (none yet)
cargo test -p gemini-sdk <pattern>                # single test by name substring
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# Run the CLI (requires GEMINI_API_KEY in env)
export GEMINI_API_KEY=...
cargo run -p gemini-cli -- generate "..." [-m <model>] [-i <image-path>]
cargo run -p gemini-cli -- chat       [-m <model>]
cargo run -p gemini-cli -- list-models
```

The CLI calls `std::env::var("GEMINI_API_KEY").expect(...)` at startup — missing key = panic, not graceful error.

## Architecture

### Request lifecycle (`gemini-sdk/src/client.rs`)

`GeminiClient::new` is the **only** constructor. It hardcodes a 60 req/min `governor` quota and a 30 s `reqwest` timeout. No builder, no `with_*` overrides, no `from_env`. If you need to tune these, add a real constructor — don't paper over with a wrapper.

Three private helpers funnel every outbound call:

- `make_request<T>` — POST + JSON in/out
- `make_get_request<T>` — GET + JSON out
- `make_stream_request` — POST returning a `Stream<Item = Result<String, GeminiError>>`

All three: `rate_limiter.until_ready().await` → `backoff::future::retry` with `ExponentialBackoff::default()`. **Important quirk:** every error inside the retry closure is wrapped as `backoff::Error::Permanent`, so the retry loop never actually retries — it just runs once and returns. If you expect transient-failure recovery, this needs to be changed to `Transient` for the retryable cases (5xx, network errors).

The `GeminiClientTrait` defines only three async methods: `generate_content`, `generate_content_stream`, `list_models`. The trait uses `#[async_trait]` for the first and third, and `-> impl Stream<...>` directly for the streaming method (which requires nightly-style RPITIT — works on stable since Rust 1.75).

URL construction passes the API key as a `?key=...` query param (`v1beta` for generation, `v1` for `list_models`). Don't switch versions casually; the response shapes differ.

### Streaming (`make_stream_request`)

The server returns SSE-style frames. The implementation:

1. Spawns a `tokio::spawn` task that owns the bytes stream and an `mpsc::unbounded_channel` sender.
2. Appends each chunk to a `String` buffer (`String::from_utf8_lossy` — lossy on partial multi-byte UTF-8 boundaries, currently acceptable but noteworthy).
3. Splits on `\n\n`, drains each complete frame, feeds it to `parse_sse_frame`.
4. `parse_sse_frame` keeps only `data:` lines, joins them, ignores empty/`[DONE]`, parses JSON, then `extract_text_from_value` walks the JSON recursively collecting every `text` field's string value (via `collect_texts`).
5. Returns an `UnboundedReceiverStream` to the caller.

The recursive text extraction is intentionally schema-agnostic — it works even when Gemini changes the response envelope, but it also silently merges text from anywhere in the tree. Don't rely on it for structured output.

### Types (`gemini-sdk/src/types.rs`)

All request/response structs are plain `pub` fields built with struct literals — no builders. `Part` is `#[serde(untagged)]`, so it deserializes by trying each variant. `SafetyCategory` and `SafetyThreshold` use `#[serde(rename = "...")]` to map to the SCREAMING_SNAKE API constants. `GeminiModel` has three named variants plus `Custom(String)`; `as_str()` returns the wire string.

### CLI (`gemini-cli/src/main.rs`)

Three subcommands: `Generate`, `ListModels`, `Chat`. Each model arg defaults to `"gemini-3.0-pro"`, which is **not** a `GeminiModel` enum variant — `parse_model` falls through to `GeminiModel::Custom`. Adding a new well-known model means updating both the `GeminiModel` enum *and* `parse_model`'s match arms.

`Chat` consumes the streaming endpoint, prints chunks live, and appends the accumulated text back to `history` as a `"model"`-role `Content`. Type `exit` to quit.

`load_image` uses `mime_guess` + base64 STANDARD engine; `save_blob` maps `mime_type` → file extension (`png`/`jpg`/`webp`/`gif`/`bmp`/`heic`, else `bin`) and writes `gemini_output_<unix_ts>.<ext>` to CWD.

## README drift to watch for

The `README.md` documents an `ImageGenerationModel` struct, a `generate_image` trait method, and an `Image` CLI subcommand. **None of these exist in the current code.** Don't cite the README as authoritative for the SDK surface — read `src/` instead. If you implement these (or remove them from the README), keep the two in sync.

## Working rules

Per repository policy: implement using TDD red-green, ship via stacked PRs (branch over branch) when changes are material, open PRs often, test always. No tests exist yet — the first non-trivial change should add `#[cfg(test)]` modules or an integration `tests/` directory and a way to exercise the HTTP layer without real network (e.g. `wiremock` or `mockito`) before extending behavior.
