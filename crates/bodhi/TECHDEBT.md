# bodhi (frontend) Technical Debt

## Unify create/update forms to match single `*Form` API types

- **Currently**: Frontend has separate create and update form conversion functions (e.g., `convertFormToCreateApi`, `convertFormToUpdateApi` in `schemas/alias.ts`)
- **Should be**: Unified to use a single conversion since the backend now uses a single `*Form` type for both create and update (e.g., `UserAliasForm`, `ApiModelForm`)
- **Reason**: Reduces duplication and aligns frontend with backend's unified form pattern


## App Crash now no longer prints on stdout

Probably after boostrap-service refactoring, app silently crashing
