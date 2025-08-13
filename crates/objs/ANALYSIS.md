## Purpose
This crate provides shared domain objects, error types, localization plumbing, GGUF parsing utilities, Hugging Face hub helpers, OpenAI request/context parameter models, role/scope types, and test fixtures used across the Bodhi workspace. It centralizes cross-crate primitives and conventions so other services can rely on consistent behavior (error semantics, i18n messages, scope parsing, logging, etc.).

## Crate structure (high-level)
- **Public exports (see `src/lib.rs`)**:
  - **Errors**: `error::*` (e.g., `AppError`, `ApiError`, I/O/serde/builder errors, GGUF errors)
  - **Localization**: `FluentLocalizationService`, `LocalizationService`, `l10n::L10N_RESOURCES`
  - **GGUF**: `gguf::*` (constants, metadata parser, value model, l10n resources)
  - **Models**: `Alias`, `AliasBuilder`, `Repo`, `HubFile`, `RemoteModel`
  - **LLM params**: `OAIRequestParams`, `GptContextParams`
  - **Access control**: `Role`, `TokenScope`, `UserScope`, `ResourceScope`
  - **API**: `api_tags` constants
  - **Utils**: `to_safe_filename`, `is_default`, HTTP log helpers (`log::*`)
  - **Test helpers**: `test_utils::*` (behind `--features test-utils`; auto-available in tests)

- **Key modules**
  - `src/error/`: error taxonomy, conversions, HTTP/OpenAI error mapping
  - `src/localization_service.rs`: fluent-rs i18n with embedded resources
  - `src/gguf/`: GGUF constants, parser, value model, errors, resources
  - `src/alias.rs`, `src/repo.rs`, `src/hub_file.rs`, `src/remote_file.rs`: model metadata
  - `src/oai.rs`, `src/gpt_params.rs`: OpenAI params + clap/serde/utoipa integration
  - `src/role.rs`, `src/token_scope.rs`, `src/user_scope.rs`, `src/resource_scope.rs`: access control primitives
  - `src/envs.rs`: settings data model (`SettingMetadata`) with parsing/validation
  - `src/log.rs`: PII-safe HTTP logging helpers
  - `src/utils.rs`: misc helpers
  - `src/resources/` and `src/gguf/resources/`: localized error messages (`.ftl`)
  - `src/test_utils/`: rstest fixtures, temp dirs, l10n bootstrapping, data generators (Python)

## Dependencies (selected)
- **Localization**: `fluent`, `include_dir`, `unic-langid`
- **HTTP/Schema**: `axum`, `http-body-util` (dev), `utoipa`
- **OpenAI**: `async-openai`
- **Parsing/Serde**: `regex`, `serde`, `serde_json`, `serde_yaml`, `byteorder`, `memmap2`
- **Validation/Builders**: `validator`, `derive_builder`, `derive-new`, `strum`
- **Logging**: `tracing`, `tracing-subscriber`
- **HF Hub**: `hf-hub`
- **Testing**: `rstest`, `tempfile`, `pretty_assertions` (+ Python scripts)

- **Feature flags**
  - `test-utils`: enables `dircpy`, `dirs`, `fs_extra`, `http-body-util`, `rstest`, `tempfile`, `tracing-subscriber` for fixtures/data generation.

## Error system & localization
- **Core trait**: `AppError` defines `error_type()`, `status()`, `code()`, `args()`.
- **Derives**: errors use `thiserror::Error` + `errmeta_derive::ErrorMeta` to auto-implement `AppError` with:
  - **HTTP status** via `ErrorType` (BadRequest, InternalServer, Authentication, Forbidden, NotFound, ServiceUnavailable).
  - **Stable code strings** used as i18n keys.
- **HTTP/OpenAI mapping**:
  - `ApiError` is the handler-facing envelope; `IntoResponse` produces OpenAI-compatible JSON (`OpenAIApiError`).
  - `OpenAIApiError` serializes `{ error: { message, type, code, param? } }` and carries `status` (not serialized).
- **Localization flow**:
  - `FluentLocalizationService::get_instance()` returns a singleton (non-test); tests override via `test_utils/l10n.rs`.
  - `ApiError` → localized message via fluent; on failure, falls back to a safe default and warns.

- **When adding a new error**
  1. Add the variant with `#[error("key")]` and `#[error_meta(error_type = ErrorType::..., ...)]`.
  2. Add translations for the key in `src/resources/*/messages.ftl` (and other crates if the error is cross-cutting).
  3. Prefer feeding variables via `args()` into fluent placeholders instead of formatting by hand.
  4. Add tests asserting localized messages (see `error/common.rs` tests).

## Localization service
- Loads `.ftl` files from embedded dirs; filters by extension; warns on invalid content or non-dirs.
- Thread-safe via `RwLock`; includes concurrent access test.
- Supported locales are `en-US` and `fr-FR` by default; unsupported → `LocalizationSetupError::LocaleNotSupported`.
- Test fixture `setup_l10n()` loads resources from this and peer crates to ensure cross-crate errors localize.

## GGUF utilities
- **Constants**: `GGUF_MAGIC = 0x46554747`, supported versions 2–3; endian autodetection based on version word.
- **Parser**: `GGUFMetadata::new(Path)` mmaps file, validates magic/version, reads KV pairs into `BTreeMap<String, GGUFValue>`.
- **Values**: `GGUFValueType` + `GGUFValue` with typed accessors (`as_u32`, `as_str`, `as_bool`, etc.) returning `TypeMismatch` when incorrect.
- **Errors**: `GGUFMetadataError` (I/O, magic/version, invalid types, unexpected EOF, array type mismatch) are localized.

- **Extending GGUF**
  - New types: extend `constants.rs`/`value.rs`, update `metadata.rs` match arms, add tests.
  - New versions: adjust supported constants and checks; add fixtures.

## HF Hub models, aliases, repos
- **Repo**: strict `user/name`, `FromStr`, serde as string, `path()` aligns with `hf_hub` folder naming.
- **HubFile**: describes a cached file (`hf_cache`, `repo`, `filename`, `snapshot`, `size`). `TryFrom<PathBuf>` validates HF cache layout: `.../hub/models--<user>--<name>/snapshots/<hash>/<file>`.
- **Alias**: YAML-serializable alias config with `OAIRequestParams` and `GptContextParams`.
  - `config_filename()` replaces separators and illegal chars → `<alias>.yaml` (e.g., `llama3--instruct.yaml`).
- **RemoteModel**: deserializable spec for remote alias/model metadata.

- **Test builders** (useful patterns)
  - `Repo::{llama3(), llama2(), tinyllama(), testalias(), ...}`
  - `HubFileBuilder::{testalias(), testalias_exists(), llama3_tokenizer(), ...}`
  - `AliasBuilder::{testalias(), llama3(), tinyllama(), ...}`

## OpenAI params and context
- **OAIRequestParams**: clap-validated ranges, serde skipping, and `update(&mut CreateChatCompletionRequest)` that fills only missing fields (non-destructive overlay); `stop` writes `Stop::StringArray` if unset.
- **GptContextParams**: llama.cpp runtime knobs (`n_ctx`, `n_parallel`, `n_predict`, etc.) with clap/serde and builder support.

- **Usage**: Construct with builder or literal; call `params.update(&mut request)` to respect pre-filled values.

## Roles and scopes
- **Role**: `resource_*` roles with total ordering (Admin > Manager > PowerUser > User). Helpers: `has_access_to`, `included_roles`, `from_resource_role`.
- **TokenScope**: `scope_token_*` parsed from a space-separated string; requires `offline_access`; helpers `has_access_to`, `included_scopes`.
- **UserScope**: `scope_user_*` parsed similarly but without `offline_access` requirement.
- **ResourceScope**: union type with `try_parse` across token/user scopes and helpful error on failure.

- **Guidelines**
  - Keep string/serde names consistent with upstream identity provider and middleware expectations.
  - Maintain ordering when adding variants; update parsing/tests accordingly.

## Settings metadata (`envs.rs`)
- `Setting`, `SettingInfo`, `SettingSource`, `SettingType`, `SettingMetadata` (String, Number{min,max}, Boolean, Option{options}).
- `SettingMetadata::parse` normalizes from YAML values; `convert` converts/validates from JSON values, returning localized `SettingsMetadataError` for invalid/unsupported values.

- **Extending**: Add new variant(s), update `parse`/`convert`, add tests for valid/invalid paths.

## API tags (`api_tags.rs`)
- Centralized OpenAPI tag constants: `system`, `setup`, `auth`, `api-keys`, `models`, `settings`, `openai`, `ollama`.

## Logging helpers (`log.rs`)
- `mask_sensitive_value`, `mask_form_params`, `log_http_request/response/error` using `tracing` fields; use these to avoid exposing secrets/PII.

## Utilities
- `to_safe_filename`: replace illegal characters with `--`, strip whitespace/control, truncate to 255.
- `is_default`: useful for serde `skip_serializing_if` on default-like sub-objects.
- `impl_error_from!` macro: ergonomic conversion glue between external errors and `AppError` types.

## Test utilities (feature: `test-utils`)
- **Temp dirs & HF cache**: `temp_dir()`, `temp_bodhi_home()`, `empty_bodhi_home()`, `temp_hf_home()`, `empty_hf_home()`
- **I/O & HTTP**: `copy_test_dir(src, dst)`, `parse<T>(Response)`, `parse_txt(Response)`
- **Localization**: `setup_l10n()` loads resources from many crates; overrides `FluentLocalizationService::get_instance()` during tests.
- **Data generators (Python)**: `generate_test_data_gguf_metadata()`, `generate_test_data_gguf_files()`, `generate_test_data_chat_template()`
- **Tracing**: `enable_tracing()` configures `tracing_subscriber` in tests.

Notes: Prefer these fixtures over custom setup in tests across crates.

## Safe extension playbook
- **New API error**: add error variant + `.ftl` keys; return `Result<_, ApiError>` and `?`-propagate your `AppError`.
- **New GGUF support**: update value/types and parser; add fixtures via Python scripts and tests asserting typed getters.
- **New scope/role**: maintain ordering; update `FromStr`, serde names, and tests for ordering/inclusion/parse.
- **New setting type/rule**: extend `SettingMetadata`, update `parse`/`convert`, add localized errors/tests.
- **Alias changes**: maintain YAML schema; `is_default` helps keep serialization lean; adjust `to_safe_filename` policy only if app-wide.

## Cross-crate integration notes
- Error codes must be present in at least one loaded resource dir. `setup_l10n` mirrors prod by loading resources from multiple crates.
- The OpenAI error envelope (`OpenAIApiError`) is consumed by APIs; keep message/type/code semantics stable for the frontend.
- `Repo`/`HubFile` are used where HF cache/GGUFs are needed; changes affect discovery/validation.

## Build & test
- Unit tests co-located and use rstest fixtures from `test_utils`.
- Run: `cargo test -p objs` (Python 3 required for data generators).
- If you add frontend-visible error codes, ensure translations are added and `setup_l10n` loads the crate providing them.

## Known invariants & pitfalls
- Only `en-US` and `fr-FR` supported by default in this crate.
- `TokenScope::from_scope` requires `offline_access`; both scope parsers are case-sensitive.
- `Repo` is strict `user/name`; empty user/name serializes as empty string.
- GGUF parser performs bounds checks and returns `UnexpectedEOF` for truncations rather than panicking.
- Use logging helpers to avoid leaking secrets.

## Quick API index (selected)
- **Errors**: `AppError`, `ErrorType`, `ApiError`, `OpenAIApiError`, I/O/serde/builder errors, `GGUFMetadataError`, `SettingsMetadataError`
- **L10n**: `FluentLocalizationService::get_instance()`, `LocalizationService::get_message`
- **GGUF**: `GGUFMetadata::new`, `GGUFValue::{as_*}`
- **Models**: `Alias`, `Repo`, `HubFile`, `RemoteModel`
- **Params**: `OAIRequestParams::update`, `GptContextParams`
- **Access**: `Role::{has_access_to,included_roles,from_resource_role}`, `TokenScope::{from_scope,included_scopes}`, `UserScope::{from_scope,included_scopes}`, `ResourceScope::try_parse`
- **Utils**: `to_safe_filename`, `mask_sensitive_value`, `log_http_request/response/error`
