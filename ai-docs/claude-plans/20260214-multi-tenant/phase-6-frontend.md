# Phase 6: Frontend Multi-Tenant Changes

## Goal
Update the Next.js frontend to support multi-tenant org awareness: org display, org switcher, and org-scoped API interactions.

## Prerequisites
- Phase 5 complete (backend fully multi-tenant, deployed with Traefik)

---

## Step 1: Org Context in Frontend

### New API Client Hooks

```typescript
// hooks/useOrg.ts
import { useQuery } from '@tanstack/react-query';

interface OrgInfo {
  org_id: string;
  slug: string;
  display_name: string;
  status: string;
}

interface OrgMembership {
  org_id: string;
  slug: string;
  display_name: string;
  role: string;
}

export function useCurrentOrg() {
  return useQuery<OrgInfo>({
    queryKey: ['org', 'current'],
    queryFn: () => fetch('/bodhi/v1/orgs/current').then(r => r.json()),
    staleTime: 5 * 60 * 1000, // 5 min cache
  });
}

export function useUserOrgs() {
  return useQuery<OrgMembership[]>({
    queryKey: ['org', 'memberships'],
    queryFn: () => fetch('/bodhi/v1/orgs/user-memberships').then(r => r.json()),
    staleTime: 5 * 60 * 1000,
  });
}
```

### Org Context Provider
```typescript
// contexts/OrgContext.tsx
import { createContext, useContext } from 'react';

interface OrgContextType {
  org: OrgInfo | null;
  isLoading: boolean;
}

const OrgContext = createContext<OrgContextType>({ org: null, isLoading: true });

export function OrgProvider({ children }: { children: React.ReactNode }) {
  const { data: org, isLoading } = useCurrentOrg();

  return (
    <OrgContext.Provider value={{ org: org ?? null, isLoading }}>
      {children}
    </OrgContext.Provider>
  );
}

export function useOrgContext() {
  return useContext(OrgContext);
}
```

---

## Step 2: Org Switcher Component

### OrgSwitcher
```typescript
// components/OrgSwitcher.tsx

export function OrgSwitcher() {
  const { org } = useOrgContext();
  const { data: memberships } = useUserOrgs();

  const handleOrgSwitch = (slug: string) => {
    // Navigate to the other org's subdomain
    const currentHost = window.location.host;
    const orgDomain = currentHost.split('.').slice(1).join('.');
    window.location.href = `${window.location.protocol}//${slug}.${orgDomain}/ui/chat`;
  };

  if (!org || !memberships || memberships.length <= 1) {
    // Single org or loading - just show org name
    return (
      <div className="flex items-center gap-2 px-3 py-2">
        <span className="text-sm font-medium">{org?.display_name ?? 'Loading...'}</span>
      </div>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="flex items-center gap-2 px-3 py-2">
        <span className="text-sm font-medium">{org.display_name}</span>
        <ChevronDown className="h-4 w-4" />
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        {memberships.map((m) => (
          <DropdownMenuItem
            key={m.org_id}
            onClick={() => handleOrgSwitch(m.slug)}
            data-testid={`org-switch-${m.slug}`}
          >
            <span>{m.display_name}</span>
            {m.slug === org.slug && <Check className="ml-auto h-4 w-4" />}
            <span className="text-xs text-muted-foreground ml-2">{m.role}</span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

---

## Step 3: Layout Integration

### Sidebar/Header Update
```typescript
// In the main layout where user info is displayed
// Add OrgSwitcher next to user profile

<header>
  <OrgSwitcher />
  <UserMenu />
</header>
```

### OrgProvider in App Layout
```typescript
// app/layout.tsx or equivalent
export default function RootLayout({ children }) {
  return (
    <OrgProvider>
      {children}
    </OrgProvider>
  );
}
```

---

## Step 4: API Call Adjustments

### No URL Changes Needed
- Frontend makes API calls to same origin (e.g., `/bodhi/v1/chat/completions`)
- The subdomain determines the org context server-side
- No client-side org_id injection needed in API calls

### Error Handling for Org Issues
```typescript
// Intercept org-related errors in API client
const apiClient = {
  async fetch(url: string, options?: RequestInit) {
    const response = await fetch(url, options);

    if (response.status === 404 && response.headers.get('X-Error-Code') === 'org_not_found') {
      // Org doesn't exist - redirect to main domain
      window.location.href = 'https://getbodhi.app';
      return;
    }

    if (response.status === 403 && response.headers.get('X-Error-Code') === 'org_suspended') {
      // Org suspended - show message
      // ...
    }

    return response;
  }
};
```

---

## Step 5: Login Flow (No Frontend Changes Needed)

The login flow is server-driven:
1. User clicks "Login" → frontend calls `/bodhi/v1/auth/login`
2. Backend uses org-specific KC client_id for OAuth redirect
3. User authenticates at KC
4. KC redirects back to org subdomain callback
5. Backend sets org-scoped session cookie

**No frontend changes needed for login.** The backend handles org-aware credential resolution.

---

## Step 6: Conditional Features Based on Mode

### Multi-Tenant vs Single-Tenant UI Differences
```typescript
// hooks/useAppMode.ts
export function useAppMode() {
  const { data } = useQuery({
    queryKey: ['app', 'mode'],
    queryFn: () => fetch('/bodhi/v1/app/info').then(r => r.json()),
    staleTime: Infinity, // Mode doesn't change at runtime
  });

  return {
    isMultiTenant: data?.multi_tenant ?? false,
    isHosted: data?.multi_tenant ?? false,
  };
}
```

### Hide Local LLM Features in Hosted Mode
```typescript
// In model management pages
const { isHosted } = useAppMode();

// Hide download/local model sections
{!isHosted && (
  <section>
    <h2>Local Models</h2>
    <ModelDownloadForm />
    <LocalModelList />
  </section>
)}

// Always show remote API configuration
<section>
  <h2>API Providers</h2>
  <ApiModelAliasForm />
</section>
```

---

## Step 7: New Backend Endpoint for App Info

### GET /bodhi/v1/app/info
```rust
pub async fn app_info_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  Ok(Json(json!({
    "version": state.app_service().setting_service().version(),
    "multi_tenant": state.app_service().setting_service().is_multi_tenant(),
  })))
}
```

This is a public endpoint (no auth required) so the frontend can determine mode before login.

---

## Step 8: Frontend Tests

### Component Tests
```typescript
// OrgSwitcher.test.tsx
describe('OrgSwitcher', () => {
  it('shows org name when single org', () => {
    render(<OrgSwitcher />, {
      wrapper: createWrapper({ org: mockOrg, memberships: [mockOrg] }),
    });
    expect(screen.getByText('Test Org')).toBeInTheDocument();
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
  });

  it('shows dropdown when multiple orgs', () => {
    render(<OrgSwitcher />, {
      wrapper: createWrapper({ org: mockOrg, memberships: [mockOrg, mockOrgBeta] }),
    });
    fireEvent.click(screen.getByText('Test Org'));
    expect(screen.getByTestId('org-switch-beta-org')).toBeInTheDocument();
  });

  it('navigates to different subdomain on switch', () => {
    // ... test window.location change
  });
});
```

### App Mode Tests
```typescript
// Test hosted mode hides local LLM features
describe('Hosted Mode', () => {
  it('hides local model management in hosted mode', () => {
    // Mock app/info to return multi_tenant: true
    // Verify download/local model sections are hidden
  });

  it('shows API provider config in all modes', () => {
    // Verify remote API configuration is always visible
  });
});
```

---

## Deliverable
- OrgContext provider and hooks
- OrgSwitcher component with subdomain navigation
- Conditional UI based on hosted/self-hosted mode
- Error handling for org-related errors
- App info endpoint for mode detection
- Frontend tests for org-related components
- No changes to API call patterns (subdomain handles routing)

## Testing Checklist
- [ ] OrgSwitcher shows current org
- [ ] OrgSwitcher dropdown works with multiple orgs
- [ ] Org switch navigates to correct subdomain
- [ ] Local LLM features hidden in hosted mode
- [ ] API provider config visible in all modes
- [ ] Login flow works without frontend changes
- [ ] Org suspension shows appropriate message
- [ ] Frontend tests pass (`npm test`)
- [ ] `make build.ui` succeeds with new components
