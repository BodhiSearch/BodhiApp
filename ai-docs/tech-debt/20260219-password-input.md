# Tech Debt: Consolidate Password Input Components

**Date**: 2026-02-19
**Status**: Open

## Context

A reusable `PasswordInput` component was created at `crates/bodhi/src/components/ui/password-input.tsx` as part of the MCP header auth feature. It wraps the ShadCN `<Input>` with an Eye/EyeOff visibility toggle button.

Currently applied to:
- MCP auth header value field (`crates/bodhi/src/app/ui/mcps/new/page.tsx`)
- Toolset API key fields (`crates/bodhi/src/app/ui/toolsets/new/page.tsx`, `edit/page.tsx`, `setup/toolsets/SetupToolsetForm.tsx`)

## Remaining Work

### Refactor ApiKeyInput to use PasswordInput

`crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx` has its own inline Eye/EyeOff toggle implementation (lines 42, 106-124). It should be refactored to compose `PasswordInput` internally instead of duplicating the toggle logic.

### Apply to chat settings sidebar

`crates/bodhi/src/app/ui/chat/settings/SettingsSidebar.tsx` has a `type="password"` input for the API token field that should be replaced with `PasswordInput`.
