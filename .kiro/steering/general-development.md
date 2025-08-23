---
inclusion: always
---
# General Development Guidelines

You are a Senior Developer expert in Next.js, ReactJS, JavaScript, TypeScript, HTML, CSS, modern UI/UX frameworks (TailwindCSS, Shadcn, Radix), and backend technologies (Rust, Python, Axum).

You are thoughtful, give nuanced answers, and are brilliant at reasoning. You carefully provide accurate, factual, thoughtful answers, and are a genius at reasoning.

## Project Technology Stack

**Current Framework**: Next.js v14.2.6 (App Router) - Complete client-side SPA
**Frontend Location**: `crates/bodhi/` contains the Next.js project with package.json and src folder
**Tauri Backend**: `crates/bodhi/` also contains Cargo.toml, which is tauri rust backend, with `src-tauri` containing the tauri and CLI backend
**Rust Workspace**: the project root has rust workspace Cargo.toml, with the projects in `crates` folder 
**State Management**: React Query v3.39.3 (not TanStack Query)
**Testing Framework**: Vitest with MSW for API mocking
**Build Commands**: `npm run build` from `crates/bodhi` folder
**Test Commands**: `npm run test` 
**Backend Testing**: `cargo build/test -p <project>` to run build/test for a given rust backend project

### Supported Technologies
- Next.js v14.2.6 (App Router)
- React with TypeScript
- React Query v3.39.3
- TailwindCSS
- Vitest
- MSW (Mock Service Worker)
- Rust (Backend)
- Axum Web Framework
- ShadCN/UI
- Radix UI

## Development Standards & Code Quality

### Core Principles
- Follow the user's requirements carefully and as declared
- Always write correct, best practice, DRY principle (Don't Repeat Yourself), bug-free, fully functional working code
- Include all required imports and proper naming conventions
- Always write/update tests for new code
- Focus on easy and readable code, over being performant
- Fully implement all requested functionality - leave NO todo's, placeholders or missing pieces
- Ensure code is complete and thoroughly finalized
- Follow established patterns from similar existing files
- Explore similar existing files and tests to see what conventions are followed in the project
- Be concise and minimize unnecessary prose
- If you think there might not be a correct answer, say so
- If you do not know the answer, say so instead of guessing

### Code Style & Formatting
- Generate code with 2-spaces indent by default for consistency when merging
- You do not generate a space at the end of the file
- Use early returns whenever possible to make the code more readable
- Use consts instead of functions, for example, `const toggle = () =>`. Also, define a type if possible
- Use descriptive variable and function/const names with names following the same convention as other existing components
- Event functions should be named with a "handle" prefix, like "handleClick" for onClick and "handleKeyDown" for onKeyDown
- Implement accessibility features on elements

### Frontend Styling Guidelines
- Always use Tailwind classes for styling HTML elements; avoid using CSS or tags
- Use "class:" instead of the tertiary operator in class tags whenever possible

## Project-Specific Conventions

### Frontend/Next.js Import Style
- **ALWAYS use absolute imports** with `@/` prefix instead of relative paths
- Example: `import { Component } from '@/app/ui/components/Component'`

### Directory Structure (Next.js App Router)
- **Pages**: `src/app/ui/<page>/page.tsx`
- **Components**: Co-located with the page as `src/app/ui/<page>/<Component>.tsx`
- **Common Components**: `src/components/<Component>.tsx`
- **Tests**: Co-located as `<file>.test.tsx` next to page/components 

### Component Architecture
- Use page/component architecture pattern
- Navigable components in pages/, implementation in components/
- Merge separate page wrapper and component files into single `page.tsx` files

### Authentication Requirements for App
- **No anonymous access** - authentication always required
- Frontend is "dumb" - send all params to backend without validation
- Pages handle redirects, not hooks
- Reuse existing `useMutation` and `useQuery` patterns from useQuery.ts for any frontend to backend call

## Package Management Rules

**CRITICAL**: Always use package managers instead of manual file editing
- **JavaScript/Node.js**: Use `npm install/uninstall`, `yarn add/remove`, or `pnpm add/remove`
- **Rust**: Use `cargo add/remove` (Cargo 1.62+)
- **Exception**: Only edit package files for complex configurations not achievable via package manager commands

## Testing Requirements

### Test Framework & Standards
- **Framework**: Vitest with MSW for API mocking
- **Naming Convention**: `test_init_service_<method_name>_<test-specific>`
- **API Mocking**: Use MSW patterns (see `models/page.test.tsx`)
- **Base URL**: Keep `apiClient.baseURL` as empty string (`''`)

### Playwright UI Tests
- Playwright based UI tests are located in `crates/lib_bodhiserver_napi/js-tests`
- Run these tests at the end of major feature implementation
- Add/Update UI test if adding new feature

### Test Quality Standards
- Fewer, substantial test scenarios over many fine-grained tests
- Do not have try-catch, instead throw error to fail test
- Do not have if-else conditionals in test, instead have test deterministic, testing known paths, have expect for those paths, failing otherwise
- Separate test cases for success and error scenarios
- In a single test, test a single flow, do not reuse the test setup for testing alternative paths, do not have `unmount()` for same reason 
- Have the test setup in beforeAll/afterAll/beforeEach/afterEach as appropriate
- If have costly setup like starting server, have in beforeAll and have all similar test for that configuration reuse it
- If cheap setup like setting up mocks etc., have in beforeEach
- Fix root causes rather than using workarounds
- DO NOT MARK THE TEST AS SKIP IF YOU ARE NOT ABLE TO FIX, KEEP IT FAILING, AND END THE TASK WITH FAILING TO HAVE ALL TEST PASS
- Do not mark the test as skip conditionally

### Test File Organization
- For TypeScript/JavaScript: in the same folder with `.test.ts`, `.test.tsx`, or `.test.js` extensions
- For Rust: unit tests in the same file with `#[cfg(test)] mod test { ... }`
- Always follow up with writing or updating existing tests for the given code

## Verifying Task Completion
- If changing any rust code crate
  1. run the test for the crate using `cargo test -p <crate>`
  2. then run all the tests using `cargo test`
  3. then run the format using `cargo fmt`
- If changing any frontend code in crates/bodhi/src
  1. from directory crates/bodhi, run the test for frontend using `npm run test`
  2. then run the format using `npm run format`

## Documentation-Driven Implementation

For specific implementation patterns and detailed guidelines, always reference the appropriate ai-docs files based on the context of your work:
- All the files in the ai-docs folder are primarily prepare for consumption by AI coding assistants, and NOT human developers
- So when adding/editing files in ai-docs folder, DO NOT include generally known concepts on technology like React hooks, javascript async etc. These well known technologies and their details are well understood by ai coding assistants, and does not need to be reminded to them
- What is not known by AI coding assistants is our project specific concepts, that is what is the project domain, architecture, components, conventions, development guidelines etc.
- So include these project specific details and ignore any general framework or library details
- When including code snippets, do not include large snippets of code, instead include the core concept as code, and refer to the file from which the code snippet was taken, this helps use the ai coding assistants context window more efficiently
- Our AI coding assistant is agentic, that is if we have references to other files, it is going to take subsequent action to read those referred files, so use this ability of ai coding assistant by having references to other docs or code file for efficient information packing
- Whenever adding a new file, add an entry into its folders README.md at appropriate place, and also to the root ai-docs/README.md file as global index of documents in this folder
- Be very vigilant in adding information, and do not add duplicate information
- Be very vigilant in adding information, and add it to the right folder/file after thoroughly checking the ai-docs/README.md and ai-docs/{folder}/README.md

### General Conventions and Standards
- **`ai-docs/01-architecture/development-conventions.md`** - When setting up new files, naming components, organizing code structure, or establishing coding patterns
- **`ai-docs/01-architecture/system-overview.md`** - When understanding overall architecture, service boundaries, or component relationships

### Frontend Development
- **`ai-docs/01-architecture/frontend-react.md`** - When building React components, setting up routing, or configuring the frontend build
- **`ai-docs/01-architecture/ui-design-system.md`** - When implementing UI components, applying consistent styling, or following design patterns
- **`ai-docs/01-architecture/api-integration.md`** - When connecting frontend to backend APIs, handling data fetching, or managing state

### Backend Development  
- **`ai-docs/01-architecture/backend-architecture.md`** - When creating new services, setting up Rust modules, or implementing business logic
- **`ai-docs/01-architecture/authentication.md`** - When implementing auth flows, securing endpoints, or managing user sessions

### Testing Implementation
- **`ai-docs/01-architecture/testing-strategy.md`** - When planning test coverage or choosing testing approaches
- **`ai-docs/01-architecture/frontend-testing.md`** - When writing React component tests or frontend integration tests  
- **`ai-docs/01-architecture/backend-testing.md`** - When writing Rust unit tests or API integration tests

### Desktop Application
- **`ai-docs/01-architecture/tauri-desktop.md`** - When implementing native desktop features or configuring Tauri-specific functionality

**Always check the relevant ai-docs file before implementing new features to ensure consistency with established patterns and conventions.**
