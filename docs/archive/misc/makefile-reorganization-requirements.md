# Makefile Reorganization Requirements

## Overview

Reorganize the project's Makefile structure to improve maintainability, discoverability, and consistency through modular design and standardized naming conventions.

## Objectives

1. **Modular Architecture**: Split monolithic Makefile into domain-specific modules
2. **Naming Consistency**: Apply unified naming conventions across all targets
3. **Enhanced Discoverability**: Implement categorized help system for better UX
4. **Maintainability**: Consolidate utility functions and reduce duplication
5. **Documentation Sync**: Update all references across codebase and CI/CD

## File Structure Requirements

### Core Files

**Main Makefile**
- Must include all domain-specific .mk files
- Must contain only core development targets (test, build, format, run)
- Must implement categorized help system with dynamic generation
- Must consolidate .PHONY declarations at top

**Makefile.release.mk**
- Must contain all release-related targets
- Must contain shared release utility functions
- Must include version management for: TypeScript client, app bindings, app releases, Docker images
- Must implement git tag creation and version increment workflows

**Makefile.ci.mk**
- Must contain all CI/CD-specific targets
- Must contain workflow trigger targets
- Must support GitHub Actions integration

**Makefile.docker.mk**
- Must contain Docker build targets for all variants (CPU, CUDA)
- Must contain Docker runtime targets
- Must contain Docker image management targets

**Makefile.website.mk**
- Must contain documentation synchronization targets
- Must contain website release targets
- Must contain AI context management targets

### File Migration

- Must migrate scripts/release.mk → Makefile.release.mk to follow root-level .mk convention
- Must remove scripts/release.mk after migration

## Naming Convention Requirements

### Standard Pattern

All targets must follow: `<category>.<action>[-variant]`

**Rules:**
- Use dots (`.`) as hierarchy separator
- Use hyphens (`-`) for multi-word within same level (kebab-case)
- Maximum 3 hierarchy levels
- No snake_case (underscore) in target names

### Category Prefixes

Must support these category prefixes:
- `test.*` - Testing operations
- `build.*` - Build operations
- `format.*` - Code formatting
- `run.*` / `app.*` - Application runtime
- `release.*` - Release management
- `docker.*` - Docker operations
- `ci.*` - CI/CD automation
- `trigger.*` - Workflow triggers
- `docs.*` - Documentation operations
- `website.*` - Website operations

## Target Rename Requirements

### Release Targets
Must rename hyphenated release targets to dot notation:
- `release-ts-client` → `release.ts-client`
- `release-app` → `release.app`
- `release-app-bindings` → `release.app-bindings`
- `release-docker` → `release.docker`
- `release-docker-dev` → `release.docker-dev`

### Build Targets
Must consolidate build-related targets under `build.*` prefix:
- `ts-client` → `build.ts-client`
- `clean.ui` → `build.ui-clean`
- `rebuild.ui` → `build.ui-rebuild`

### Test Targets
Must consolidate test-related targets:
- `coverage` → `test.coverage`
- `extension.download` → `test.extension-download`

### Documentation Targets
Must use consistent `docs.*` prefix:
- `update-context-symlinks` → `docs.context-update`
- `update-context-symlinks-dry-run` → `docs.context-update-dry-run`
- `sync.docs` → `docs.sync`
- `sync.docs.check` → `docs.sync-check`

### Website Targets
Must convert snake_case to kebab-case:
- `website.update_releases` → `website.update-releases`
- `website.update_releases.check` → `website.update-releases-check`

### Docker Targets
Must move version checking to release domain:
- `check-docker-versions` → `docker.version-check` (in Makefile.release.mk)

### Target Removal
Must remove redundant targets:
- `ui.test` (redundant with `test.ui`)

## New Target Requirements

### Build Targets

**build**
- Must build command line app (server variant)
- Must use: `cargo build -p bodhi`
- Must appear in "Building" help section

**build.native**
- Must build native app with system tray
- Must use: `cd crates/bodhi/src-tauri && cargo tauri build --features native`
- Must appear in "Building" help section

**build.ui**
- Must build Next.js frontend and NAPI bindings
- Must appear in "Building" help section

### Runtime Targets

**run**
- Must run command line app
- Must use: `cargo run --bin bodhi -- serve --port 1135`
- Must appear in "App Runtime" help section

**run.native**
- Must run native app in development mode
- Must use: `cd crates/bodhi/src-tauri && cargo tauri dev`
- Must appear in "App Runtime" help section

## Help System Requirements

### Dynamic Generation

Must implement dynamic help using AWK pattern matching:
- Must not hardcode target lists
- Must auto-discover targets from .PHONY and ## comments
- Must support filtering by category prefix

### Category Groups

Must display targets in these groups (in order):
1. **Testing** - Pattern: `/^test[a-zA-Z0-9._-]*:/`
2. **Building** - Pattern: `/^build[a-zA-Z0-9._-]*:/`
3. **Formatting** - Pattern: `/^format[a-zA-Z0-9._-]*:/`
4. **App Runtime** - Pattern: `/^(run|app)[a-zA-Z0-9._-]*:/`
5. **Release** - Pattern: `/^release[a-zA-Z0-9._-]*:/`
6. **Docker** - Pattern: `/^docker[a-zA-Z0-9._-]*:/`
7. **Documentation** - Pattern: `/^docs[a-zA-Z0-9._-]*:/`
8. **Website** - Pattern: `/^website[a-zA-Z0-9._-]*:/`
9. **CI/Workflow** - Pattern: `/^(ci|trigger)[a-zA-Z0-9._-]*:/`

### Formatting

Must use ANSI color codes:
- Target names: Cyan (`\033[36m`)
- Reset: `\033[0m`
- Width: Left-aligned 30 characters for target names

### Help Text

Each target must have `## Description` comment:
```makefile
target.name: ## Description text here
	commands
```

## Structural Requirements

### .PHONY Declarations

Must consolidate .PHONY declarations:
- Main Makefile: Single .PHONY at top listing all core targets
- Each .mk file: Single .PHONY at top listing all targets in that file
- Must group related targets on same line with backslash continuation

**Format:**
```makefile
.PHONY: target1 target2 target3 \
	target4 target5 target6
```

### Target Organization

Within each file, must organize in order:
1. File header comment
2. .PHONY declaration
3. Target definitions (grouped by subcategory if applicable)
4. Utility functions (if applicable, at bottom)

### Include Order

Main Makefile must include in this order:
```makefile
include Makefile.release.mk
include Makefile.ci.mk
include Makefile.docker.mk
include Makefile.website.mk
```

## Cross-Reference Update Requirements

Must update all Makefile target references in:

### Documentation Files
**CLAUDE.md** - Root project documentation:
- `make ts-client` → `make build.ts-client`
- `make coverage` → `make test.coverage`
- `make release-ts-client` → `make release.ts-client`
- `make release-app-bindings` → `make release.app-bindings`
- `make release-docker` → `make release.docker`
- `make release-docker-dev` → `make release.docker-dev`
- `make check-docker-versions` → `make docker.version-check`
- `make clean.ui` → `make build.ui-clean`
- `make rebuild.ui` → `make build.ui-rebuild`
- `make update-context-symlinks` → `make docs.context-update`

**xtask/CLAUDE.md** - Build automation documentation:
- `make ts-client` → `make build.ts-client`

### CI/CD Files
**.github/actions/ts-client-check/action.yml**:
- `make ts-client` → `make build.ts-client`

### Sub-Makefiles
**crates/lib_bodhiserver_napi/Makefile**:
- `$(MAKE) -C ../.. clean.ui` → `$(MAKE) -C ../.. build.ui-clean`

## Utility Function Requirements

### Location
Must consolidate all release utility functions in Makefile.release.mk at bottom of file.

### Required Functions

**check_git_branch**
- Must verify current branch
- Must fetch latest changes from remote
- Must compare local vs remote HEAD
- Must prompt user for confirmation if mismatched
- Must abort if user declines

**delete_tag_if_exists**
- Must check if git tag exists locally or remotely
- Must prompt user for confirmation before deletion
- Must delete both local and remote tags if confirmed
- Must abort if user declines

**get_npm_version**
- Must query npmjs.org for package version
- Must accept package name as parameter
- Must return "0.0.0" if package not found
- Must use existing scripts/get_npm_version.js

**get_git_tag_version**
- Must find latest git tag with given prefix
- Must sort tags by semantic version
- Must return "0.0.0" if no tags found
- Must strip prefix from returned version

**get_app_version**
- Must query GitHub releases API
- Must handle migration from v* to app/v* tags
- Must use existing scripts/get_github_release_version.js

**increment_version**
- Must accept version string in X.Y.Z format
- Must increment patch version
- Must use existing scripts/increment_version.js

**get_ghcr_docker_version**
- Must query GitHub Container Registry
- Must accept variant parameter (production/development)
- Must check all hardware variants (cpu, cuda, rocm, vulkan)
- Must return highest version across all variants
- Must use existing scripts/get_ghcr_version.py

**create_docker_release_tag**
- Must accept variant and current version
- Must increment patch version
- Must create appropriate tag (docker/v* or docker-dev/v*)
- Must push tag to remote
- Must call delete_tag_if_exists before creation

## Backward Compatibility Requirements

### Breaking Changes Allowed
This reorganization introduces breaking changes:
- Old target names will NOT work
- No aliases will be provided
- Users must update their scripts/workflows

### Rationale
Clean break preferred over maintaining compatibility layer to:
- Avoid confusion with dual naming
- Simplify codebase
- Encourage adoption of new conventions

## Validation Requirements

### Functional Testing

Must verify after implementation:
1. `make help` displays all categories correctly
2. All renamed targets execute successfully
3. All new build/run targets work correctly
4. All utility functions execute without errors
5. GitHub Actions workflows pass with updated references
6. Sub-Makefile delegations work correctly

### Documentation Testing

Must verify:
1. All CLAUDE.md references updated
2. All workflow references updated
3. Help text displays correctly for all targets
4. No broken cross-references remain

### Naming Validation

Must verify:
1. No targets use snake_case (underscores)
2. All targets follow category.action pattern
3. All .PHONY declarations complete and accurate
4. No duplicate target names across files

## Implementation Guidelines

### Order of Operations

1. Create new .mk files with reorganized targets
2. Update main Makefile structure
3. Update all cross-references (documentation, CI/CD, sub-Makefiles)
4. Delete old scripts/release.mk
5. Test all targets and help system
6. Format and commit changes

### Testing Strategy

After each file modification:
- Run `make help` to verify help system
- Run sample target from each category to verify functionality
- Check for Make syntax errors

### Rollback Strategy

If issues discovered:
- Changes are contained in modular .mk files
- Can revert individual files without affecting others
- Can restore old structure by reverting commit

## Success Criteria

1. All targets follow consistent naming convention
2. Help system displays 9 categories with all targets
3. All cross-references updated successfully
4. All CI/CD workflows pass
5. All modular .mk files properly included
6. Zero broken target references in codebase
7. Documentation reflects all changes

## Non-Requirements

These are explicitly NOT part of this reorganization:

1. Changing target functionality (only renaming/reorganizing)
2. Adding new build features (except missing build/run targets)
3. Modifying CI/CD workflows (except target name updates)
4. Changing utility function implementations
5. Windows Makefile updates (separate concern)
6. Backward compatibility aliases
