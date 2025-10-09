# getbodhi.app - BodhiApp Marketing & Documentation Website

This is the official marketing and documentation website for [BodhiApp](https://getbodhi.app), built with [Next.js](https://nextjs.org) and deployed via GitHub Pages.

## Overview

The getbodhi.app website provides:

- **Marketing Pages**: Product information, features, and download links
- **Documentation**: Comprehensive user guides and API documentation
- **Developer Resources**: Integration guides and technical documentation

The website is deployed to GitHub Pages and served from the custom domain [getbodhi.app](https://getbodhi.app).

## Architecture

This website is part of the BodhiApp monorepo and synchronizes documentation from the embedded application:

- **Source of Truth**: Documentation in `crates/bodhi/src/docs/`
- **Website Docs**: Synchronized copy in `getbodhi.app/src/docs/`
- **Automated Sync**: Build-time synchronization ensures documentation stays current

### Documentation Sync System

The documentation sync system keeps the website documentation in sync with the embedded application documentation:

**What Gets Synced:**

1. **Documentation Content**: Markdown files (`*.md`) and metadata (`_meta.json`)
2. **Documentation Images**: Image assets from `public/doc-images/`
3. **Rendering Components**: React components from `src/app/docs/` (excluding tests)

**Sync Sources:**

- Content: `crates/bodhi/src/docs/` → `getbodhi.app/src/docs/`
- Images: `crates/bodhi/public/doc-images/` → `getbodhi.app/public/doc-images/`
- Components: `crates/bodhi/src/app/docs/` → `getbodhi.app/src/app/docs/`

**Implementation:**

- Uses [fs-extra](https://github.com/jprichardson/node-fs-extra) for reliable file operations
- Uses [glob](https://github.com/isaacs/node-glob) for file pattern matching
- Simple, maintainable Node.js script with minimal dependencies
- Test files automatically excluded from sync

## Development

### Prerequisites

- Node.js 18+ and npm
- Access to the parent BodhiApp monorepo

### Getting Started

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run start
```

Open [http://localhost:3000](http://localhost:3000) to view the website.

### Documentation Sync Commands

#### Sync Documentation

Synchronize documentation from the embedded app to the website:

```bash
npm run sync:docs
```

Or using Make:

```bash
make sync.docs
```

This copies the latest documentation content, images, and rendering components from the embedded application.

#### Check Sync Status

Verify if documentation is in sync without making changes (dry-run):

```bash
npm run sync:docs:check
```

Or using Make:

```bash
make sync.docs.check
```

Exit codes:

- `0` - Documentation is in sync
- `1` - Documentation is out of sync

#### Automatic Sync

Documentation is automatically synced before builds via the `prebuild` npm lifecycle hook:

```bash
npm run build  # Automatically syncs docs first
```

### Project Structure

```
getbodhi.app/
├── src/
│   ├── app/              # Next.js App Router pages
│   │   ├── docs/         # Documentation rendering components (synced)
│   │   └── ...           # Marketing and info pages
│   ├── docs/             # Documentation content (synced)
│   ├── components/       # React components
│   └── lib/              # Utility functions
├── public/
│   ├── doc-images/       # Documentation images (synced)
│   └── ...               # Other static assets
├── scripts/
│   └── sync-docs.js      # Documentation sync script
├── Makefile              # Make targets for common tasks
└── package.json          # npm dependencies and scripts
```

### Technology Stack

- **Framework**: Next.js 14 with App Router
- **Styling**: TailwindCSS
- **Deployment**: GitHub Pages with custom domain
- **Documentation**: Markdown with MDX support
- **Sync Tools**: fs-extra, glob

## Deployment

The website is deployed to GitHub Pages via GitHub Actions:

1. **Build**: Next.js static export
2. **Deploy**: Automatic deployment to GitHub Pages
3. **Domain**: Served from custom domain `getbodhi.app`

### Manual Deployment

```bash
# Build static export
npm run build

# Output is in the `out/` directory
```

## Documentation Maintenance

### Keeping Documentation in Sync

The documentation in this website should always reflect the latest version in the embedded application:

1. **Make Changes**: Edit documentation in `crates/bodhi/src/docs/` (source of truth)
2. **Sync to Website**: Run `npm run sync:docs` to update website
3. **Verify**: Run `npm run sync:docs:check` to confirm sync
4. **Build**: Run `npm run build` (automatically syncs first)

### Documentation Workflow

**For Documentation Authors:**

1. Edit markdown files in `crates/bodhi/src/docs/`
2. Add images to `crates/bodhi/public/doc-images/`
3. Test in embedded app first
4. Sync to website: `cd getbodhi.app && npm run sync:docs`
5. Review website changes before commit

**For Rendering Changes:**

If you modify documentation rendering components:

1. Edit components in `crates/bodhi/src/app/docs/`
2. Test thoroughly in embedded app
3. Sync to website: `npm run sync:docs`
4. Verify rendering on website with `npm run dev`

### CI Integration

The sync check can be used as a release gate:

```bash
# In CI/CD pipeline
npm run sync:docs:check || exit 1
```

This ensures documentation is always in sync before deployment.

## Sync System Architecture

### Design Philosophy

The sync system follows these principles:

- **Simple**: Minimal code using battle-tested libraries
- **Maintainable**: Clear logic with well-defined sync targets
- **Reliable**: Proper error handling and exit codes
- **Fast**: Efficient file comparison using binary buffers
- **Deterministic**: Consistent behavior across environments

### Implementation Details

**File Operations:**

- `fs-extra`: Enhanced fs module with copy(), remove(), ensureDir()
- Automatic directory creation when copying files
- Binary file comparison using Buffer.equals()

**Pattern Matching:**

- `glob`: Industry-standard file pattern matching
- Support for multiple patterns per target
- Ignore patterns for test files

**Sync Logic:**

1. Scan source directory for matching files
2. Scan destination directory for existing files
3. Compare files to detect changes:
   - **Added**: Files in source but not destination
   - **Modified**: Files with different content
   - **Deleted**: Files in destination but not source
4. Perform operations (or report in check mode)

**Exit Codes:**

- `0`: Success (sync completed or all in sync)
- `1`: Failure (errors or out of sync in check mode)

## Contributing

When contributing to the website:

1. **Documentation Changes**: Make changes in `crates/bodhi/src/docs/` first
2. **Component Changes**: Test in embedded app before syncing
3. **Marketing Content**: Direct edits in `getbodhi.app/src/app/` are fine
4. **Always Sync**: Run `npm run sync:docs:check` before committing
5. **Test Locally**: Verify changes with `npm run dev`

## Learn More

- [BodhiApp Documentation](https://getbodhi.app/docs)
- [BodhiApp GitHub](https://github.com/BodhiSearch/BodhiApp)
- [Next.js Documentation](https://nextjs.org/docs)

## License

See the main BodhiApp repository for license information.
