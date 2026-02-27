# Review 1: Foundation Layer (`crates/objs/`)

## Files Reviewed
- `crates/objs/src/db_enums.rs`, `json_vec.rs`, `access_request.rs`, `api_model_alias.rs`, `mcp.rs`, `model_metadata.rs`, `oai.rs`, `user_alias.rs`, `lib.rs`
- `crates/objs/Cargo.toml`

## Findings

### [Nice-to-have] strum serialize_all consistency
- **File**: `crates/objs/src/db_enums.rs`, `crates/objs/src/access_request.rs`, `crates/objs/src/mcp.rs`, `crates/objs/src/api_model_alias.rs`, `crates/objs/src/user_alias.rs`
- **Issue**: All DB enums use `snake_case` for strum serialization consistently, EXCEPT `ApiFormat` which uses `lowercase` (serializes to "openai", "placeholder"). This is intentional and appropriate for this enum.
- **Recommendation**: Consider standardizing to `snake_case` across all DB enums for uniformity, or document the `ApiFormat` exception explicitly.
