# Playwright Skill Enhancement Plan

Enhance the personal playwright skill at `~/.claude/skills/playwright/` with patterns discovered in BodhiApp's codebase.

## Summary of Decisions

Based on analysis and user feedback:
- **Add**: Responsive Test IDs, Semantic Context Attributes, Nested IDs (extend), Auth Flow Pattern, Optional Waits
- **Discourage**: CSS class-based state detection
- **Skip**: Streaming patterns (existing states sufficient), Component classes (keep page composition only), Toast UUID extraction (optional waits only)

---

## Changes by File

### 1. ui-patterns.md

**Add new section: "Responsive Test IDs"** (after "Dynamic Testids")

```markdown
## Responsive Test IDs

For responsive applications, use viewport-aware test ID prefixes to target device-specific elements:

| Viewport | Prefix | Example |
|----------|--------|---------|
| Mobile (<768px) | `m-` | `m-nav-menu`, `m-send-button` |
| Tablet (768-1023px) | `tab-` | `tab-nav-menu`, `tab-send-button` |
| Desktop (>=1024px) | (none) | `nav-menu`, `send-button` |

### React Hook Pattern

```tsx
function useResponsiveTestId() {
  const [viewport, setViewport] = useState<'mobile' | 'tablet' | 'desktop'>('desktop');

  useEffect(() => {
    const updateViewport = () => {
      const width = window.innerWidth;
      if (width < 768) setViewport('mobile');
      else if (width < 1024) setViewport('tablet');
      else setViewport('desktop');
    };
    updateViewport();
    window.addEventListener('resize', updateViewport);
    return () => window.removeEventListener('resize', updateViewport);
  }, []);

  return (baseId: string) => {
    if (viewport === 'mobile') return `m-${baseId}`;
    if (viewport === 'tablet') return `tab-${baseId}`;
    return baseId;
  };
}

// Usage
const getTestId = useResponsiveTestId();
<button data-testid={getTestId('send-button')} />
```

### Playwright Selector

```typescript
// Target any viewport variant
const sendButton = page.locator('[data-testid="send-button"], [data-testid="m-send-button"], [data-testid="tab-send-button"]');

// Or use attribute prefix selector
const mobileSend = page.locator('[data-testid="m-send-button"]');
```
```

---

**Add new section: "Semantic Context Attributes"** (after "Responsive Test IDs")

```markdown
## Semantic Context Attributes

Beyond `data-testid` and `data-test-state`, use semantic attributes for queryable metadata:

### Entity Name Attributes

For elements that need entity identification beyond simple IDs:

```tsx
// Pattern: data-test-{entity}-name
<div
  data-testid={`row-${item.id}`}
  data-test-item-name={item.name}
>
  {item.name}
</div>
```

Test selector:
```typescript
// Find row by name instead of ID
await page.locator('[data-test-item-name="My Widget"]').click();
```

### Scope/Context Attributes

For elements that belong to a specific scope or context:

```tsx
// Pattern: data-test-scope or data-test-{context}
<div
  data-testid={`config-section-${scope.id}`}
  data-test-scope={scope.type}
>
  {/* scope content */}
</div>
```

Test selector:
```typescript
// Find all elements in a specific scope
const adminSections = page.locator('[data-test-scope="admin"]');
```

### When to Use

- `data-testid`: Primary element identification
- `data-test-state`: Element state for waiting
- `data-test-{entity}-name`: Human-readable entity lookup
- `data-test-scope`: Grouping/filtering by context
```

---

**Extend "Dynamic Testids" section with hierarchical/nested example:**

Add after existing dynamic testid content:

```markdown
### Hierarchical Test IDs

For nested elements, include parent context in child IDs:

```tsx
// Parent container
<div data-testid={`group-${groupId}`}>
  {items.map((item) => (
    <div
      key={item.id}
      data-testid={`group-${groupId}-item-${item.id}`}
    >
      <button data-testid={`group-${groupId}-item-${item.id}-action`}>
        Action
      </button>
    </div>
  ))}
</div>
```

Test selector:
```typescript
// Specific item in specific group
await page.locator('[data-testid="group-tools-item-search-action"]').click();

// Any item action in a specific group
await page.locator('[data-testid^="group-tools-item-"][data-testid$="-action"]').first().click();
```
```

---

### 2. state-patterns.md

**Add brief anti-pattern note** (in or after "The Problem" section or new section):

```markdown
## CSS Classes vs data-test-state

**Prefer `data-test-state` over CSS classes for state detection.**

```typescript
// AVOID: CSS class-based state detection
await expect(page.locator('.message')).toHaveClass(/message-streaming/);
await expect(page.locator('.message')).toHaveClass(/message-completed/);

// PREFER: data-test-state attribute
await expect(page.locator('[data-testid="message"]')).toHaveAttribute('data-test-state', 'streaming');
await expect(page.locator('[data-testid="message"]')).toHaveAttribute('data-test-state', 'completed');
```

**Why:**
- CSS classes can change for styling reasons
- `data-test-state` is explicitly for testing, won't accidentally break
- State attributes are self-documenting
- Easier to query multiple states with CSS selectors
```

---

### 3. page-object-patterns.md

**Add new section: "Authentication Flow Testing"** (after "Composite Page Objects"):

```markdown
## Authentication Flow Testing

For OAuth/SSO flows that span multiple domains:

```typescript
class AuthPage {
  readonly page: Page;
  readonly appBaseUrl: string;
  readonly authServerUrl: string;

  readonly selectors = {
    loginButton: '[data-testid="btn-auth-login"]',
    usernameField: '#username',
    passwordField: '#password',
    signInButton: '[data-testid="btn-oauth-signin"]',
  };

  constructor(page: Page, appBaseUrl: string, authServerUrl: string) {
    this.page = page;
    this.appBaseUrl = appBaseUrl;
    this.authServerUrl = authServerUrl;
  }

  async performOAuthLogin(
    username: string,
    password: string,
    expectedRedirectPath = '/dashboard'
  ): Promise<void> {
    // 1. Click login on app
    await this.page.locator(this.selectors.loginButton).click();

    // 2. Wait for redirect to auth server (different domain)
    await this.page.waitForURL((url) => url.origin === this.authServerUrl);

    // 3. Fill credentials on auth server
    await this.page.fill(this.selectors.usernameField, username);
    await this.page.fill(this.selectors.passwordField, password);

    // 4. Submit and wait for redirect back to app
    await this.page.click(this.selectors.signInButton);
    await this.page.waitForURL(
      (url) =>
        url.origin === this.appBaseUrl &&
        url.pathname === expectedRedirectPath
    );
  }
}
```

**Key patterns:**
- Use URL predicates with `waitForURL((url) => ...)` for cross-domain flows
- Store both app and auth server URLs in the page object
- Verify final redirect lands on expected path
```

---

**Add new section: "Optional Assertions"** (after "Debugging Tips"):

```markdown
## Optional Assertions

For assertions that may fail in certain environments (CI vs local) without breaking the test:

```typescript
class BasePage {
  /**
   * Wait for toast with optional failure handling
   * Useful for flaky toast notifications in CI
   */
  async waitForToastOptional(
    message: string,
    options: { timeout?: number } = {}
  ): Promise<boolean> {
    const timeout = options.timeout ?? (process.env.CI ? 1000 : 5000);
    try {
      await expect(this.page.locator('[data-state="open"]'))
        .toContainText(message, { timeout });
      return true;
    } catch {
      console.log(`Toast check skipped: "${message}"`);
      return false;
    }
  }
}
```

**Use sparingly for:**
- Toast notifications that may not appear in CI
- Non-critical visual confirmations
- Environment-dependent behaviors

**Don't use for:**
- Core functionality assertions
- Data validation
- Navigation verification
```

---

## Verification

After making changes:

1. **Read all skill files** to verify no duplicate content:
   ```bash
   cat ~/.claude/skills/playwright/*.md
   ```

2. **Test the skill** by asking Claude to help with:
   - Creating a responsive React component with test IDs
   - Writing a page object for an OAuth flow
   - Adding semantic context attributes to a list component

3. **Check SKILL.md** is still accurate (no changes needed - already references all files)

---

## Files to Modify

| File | Changes |
|------|---------|
| `~/.claude/skills/playwright/ui-patterns.md` | Add Responsive Test IDs section, Semantic Context Attributes section, extend Dynamic Testids with hierarchical pattern |
| `~/.claude/skills/playwright/state-patterns.md` | Add CSS classes anti-pattern note |
| `~/.claude/skills/playwright/page-object-patterns.md` | Add Authentication Flow Testing section, Optional Assertions section |

---

## Not Changing (per user decisions)

- **test-organization.md** - Already covers journey-based testing and test.step patterns
- **Streaming patterns** - Existing states (idle/loading/success/error) are sufficient
- **Component class pattern** - Keep only page composition pattern
- **Toast UUID extraction** - Only documenting optional waits pattern
- **Radix UI integration** - Too library-specific
