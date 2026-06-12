# OpenAPI-Driven Type-Safe API Solutions Research Report

## Executive Summary

This document presents comprehensive research on OpenAPI-driven type-safe API solutions compatible with TanStack Query v5, conducted on September 29, 2025. The research evaluates multiple code generation tools, their ecosystem maturity, SSE support capabilities, and suitability for React applications using TypeScript and axios.

## Research Methodology

### Search Queries Conducted

1. **TanStack Query v5 Features**
   - `tanstack query v5 type safety features typescript 2024`
   - `tanstack query v5 server sent events SSE streaming axios 2024`
   - `react-query v3 to tanstack query v5 migration guide breaking changes 2024`

2. **OpenAPI Integration Research**
   - `openapi-typescript react-query integration fetch client 2024`
   - `openapi-fetch server sent events SSE streaming support 2024`
   - `openapi-typescript tanstack query v5 integration axios real world implementation 2024`

3. **HeyAPI Ecosystem**
   - `hey-api openapi-ts ecosystem features tanstack query axios fetch 2024`
   - `"@hey-api/openapi-ts" tanstack query plugin axios integration 2024`

4. **Alternative Solutions**
   - `orval openapi code generator tanstack query v5 axios typescript 2024`
   - `zodios type-safe api client openapi tanstack query react typescript 2024`
   - `openapi-qraft tanstack query typescript zero runtime proxy design 2024`
   - `"best openapi code generator" react tanstack query 2024 comparison axios fetch typescript`

5. **SSE Support Investigation**
   - `orval "server sent events" SSE streaming support openapi tanstack query 2024`
   - `axios vs fetch openapi-typescript tanstack query custom fetcher middleware interceptors 2024`

6. **Market Analysis**
   - `npm trends orval hey-api openapi-qraft zodios openapi-react-query-codegen 2024 comparison`

## Detailed Findings by Solution

### 1. Orval - The Industry Standard

**Market Position:**
- 410,675 weekly downloads (as of research date)
- 4,609 GitHub stars
- Most widely adopted solution in production

**Technical Capabilities:**
- **OpenAPI Support:** Full OpenAPI v3 and Swagger v2
- **Code Generation:** TypeScript models, Axios client, TanStack Query hooks
- **Client Support:** Angular, axios, axios-functions, react-query, svelte-query, vue-query, swr, zod, fetch
- **Additional Features:**
  - Mock generation with Faker
  - MSW (Mock Service Worker) integration
  - Swagger validation
  - Custom templates and overrides

**TanStack Query Integration:**
- Generates one custom hook per path in OpenAPI specification
- Supports query key management
- Includes prefetch functions for SSR
- Compatible with TanStack Query v5

**SSE Support:**
- ❌ No native SSE support
- Can represent SSE endpoints using `text/event-stream` MIME type
- Requires custom implementation for streaming

**Configuration Example:**
```yaml
petstore:
  input: ./openapi.json
  output:
    target: ./src/api/endpoints
    client: axios-functions
    mode: react-query
    mock: true
    override:
      mutator:
        path: ./src/api/axios-instance.ts
        name: apiClient
```

**Pros:**
- Battle-tested in production environments
- Comprehensive feature set
- Active community support
- Native axios integration

**Cons:**
- No built-in SSE support
- Can generate large amounts of code
- Complex configuration for advanced scenarios

### 2. @hey-api/openapi-ts - The Modern Ecosystem

**Market Position:**
- Latest version: 0.84.0 (updated 8 hours ago as of research)
- 63 projects using it in npm registry
- Successor to openapi-typescript-codegen

**Technical Capabilities:**
- **Plugin Architecture:**
  - `@tanstack/react-query` plugin
  - `@tanstack/vue-query` plugin
  - `@tanstack/solid-query` plugin
  - Zod schema generation
  - Axios client support (`@hey-api/client-axios`)

**TanStack Query Plugin Features:**
- Generates queryOptions functions (e.g., `getPetByIdOptions()`)
- Query key management with metadata support
- Customizable naming patterns
- Tag-based cache invalidation
- Full compatibility with SDKs and transformers

**Configuration:**
```javascript
export default {
  input: 'hey-api/backend',
  output: 'src/client',
  plugins: [
    '@tanstack/react-query',
    '@hey-api/client-axios',
    '@hey-api/schemas'
  ]
};
```

**2024-2025 Roadmap:**
- First stable v1.0 release planned within 12 months
- Experimental parser becoming default
- OpenAPI 2.0 support backporting
- Instantiable SDKs development

**Pros:**
- Very active development
- Modern plugin architecture
- Strong TypeScript support
- Customizable code generation

**Cons:**
- No SSE support
- Still pre-1.0 (less stable)
- Newer ecosystem, less battle-tested

### 3. OpenAPI-Qraft - Zero-Runtime Innovation

**Market Position:**
- Version 2.12.0 (published 1 month ago)
- 2 projects using it in npm registry
- Innovative Proxy-based architecture

**Technical Innovation:**
- **Proxy-Based Design:** Zero-runtime overhead using JavaScript Proxies
- **Dynamic Hook Generation:** Creates hooks on-demand without code generation bloat
- **Full TanStack Query v5 Support:** Complete functionality integration

**Installation:**
```bash
npm install @openapi-qraft/react
npx @openapi-qraft/cli --plugin tanstack-query-react --plugin openapi-typescript --output-dir src/api
```

**Key Features:**
- Type-safe API requests
- Modular design with callbacks
- Seamless SSR support
- No runtime overhead
- Modern architecture

**Usage Pattern:**
```typescript
import { createAPIClient } from './api';

const api = createAPIClient({
  baseUrl: 'https://api.example.com',
  // Proxy-based - no actual code here
});

// Hooks are created dynamically
const { data } = api.pets.getPetById.useQuery({ id: '123' });
```

**Pros:**
- Zero runtime overhead
- Modern architecture
- Excellent SSR support
- Clean API design

**Cons:**
- No SSE support
- Smaller community
- Less documentation
- Newer, less proven

### 4. Zodios + openapi-zod-client - Runtime Validation

**Market Position:**
- Zodios v10.5.0 (2 years old)
- openapi-zod-client actively maintained
- Focus on runtime validation

**Ecosystem Components:**
1. **Zodios:** TypeScript HTTP client with Zod validation
2. **openapi-zod-client:** Generates Zodios client from OpenAPI
3. **@zodios/openapi:** Bidirectional OpenAPI ↔ Zodios conversion

**Unique Features:**
- Runtime validation with Zod schemas
- Type-safe at compile AND runtime
- Declarative API definition
- Plugin system
- Query key management

**Integration Pattern:**
```typescript
import { Zodios } from '@zodios/core';
import { ZodiosHooks } from '@zodios/react';
import { api } from './generated-api';

const apiClient = new Zodios(baseURL, api);
const hooks = new ZodiosHooks('myAPI', apiClient);

// Usage
const { data } = hooks.useGetPet({ params: { id: '123' } });
```

**Pros:**
- Runtime type validation
- Clean, declarative API
- Full-stack support (client + server)
- TanStack Query integration

**Cons:**
- No SSE support
- Runtime overhead from Zod
- Less recent updates
- Smaller community

### 5. openapi-react-query-codegen - Dedicated Code Generator

**Market Position:**
- Version 1.6.2 (8 months ago)
- 1 project using it
- Purpose-built for React Query

**Features:**
- Generates React Query hooks (useQuery, useSuspenseQuery, useMutation, useInfiniteQuery)
- Creates prefetch functions for SSR
- Uses @hey-api/openapi-ts for type generation
- Simple CLI interface

**Usage:**
```bash
npm install -D @7nohe/openapi-react-query-codegen
npx openapi-rq -i ./openapi.json -o ./src/api
```

**Generated Code Structure:**
```typescript
// Generated hooks
export const useGetPet = (id: string, options?: UseQueryOptions) => {
  return useQuery({
    queryKey: ['getPet', id],
    queryFn: () => getPet(id),
    ...options
  });
};
```

**Pros:**
- Simple, focused tool
- Direct TanStack Query integration
- SSR support with prefetch functions

**Cons:**
- No SSE support
- Limited customization
- Smaller community
- Basic feature set

### 6. openapi-react-query - Lightweight Wrapper

**Market Position:**
- 1kb bundle size
- Part of openapi-typescript ecosystem
- Minimal overhead approach

**Architecture:**
- Wrapper around @tanstack/react-query
- Uses openapi-fetch for requests
- Uses openapi-typescript for types

**Usage:**
```typescript
import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./my-openapi-3-schema";

const fetchClient = createFetchClient<paths>({
  baseUrl: "https://myapi.dev/v1/",
});

const $api = createClient(fetchClient);

// Usage
const { data, error, isPending } = $api.useQuery(
  "get",
  "/blogposts/{post_id}",
  {
    params: {
      path: { post_id: 5 },
    },
  }
);
```

**Pros:**
- Tiny bundle size (1kb)
- Clean API
- Direct integration with openapi-typescript
- Minimal learning curve

**Cons:**
- No SSE support (inherits from openapi-fetch)
- Requires separate openapi-fetch setup
- Less feature-rich
- Limited customization

## Critical Analysis: SSE Support

### Current State (2024)

**No major solution has native SSE support.** This is a critical finding that affects all evaluated tools.

### Root Causes:

1. **OpenAPI Specification Limitation**
   - SSE not first-class citizen in OpenAPI
   - Can only represent as `text/event-stream` MIME type
   - No standardized schema for SSE messages

2. **Technical Challenges**
   - SSE requires specialized handling
   - Different from standard REST request/response pattern
   - Requires persistent connections and stream processing

3. **Community Status**
   - 5+ years of discussion without resolution
   - Issue #396 on OpenAPI-Specification GitHub (open since 2015)
   - Various workarounds but no standard approach

### Current Workarounds:

1. **Represent in OpenAPI:**
   ```yaml
   /api/events:
     get:
       responses:
         '200':
           content:
             text/event-stream:
               schema:
                 type: string
   ```

2. **Custom Implementation:**
   ```typescript
   // Keep SSE separate from generated code
   export function useSSEEndpoint() {
     return useMutation({
       mutationFn: async () => {
         const response = await fetch('/api/events', {
           headers: { 'Accept': 'text/event-stream' }
         });
         // Handle streaming...
       }
     });
   }
   ```

3. **TanStack Query v5 Experimental Support:**
   ```typescript
   import { streamedQuery } from '@tanstack/react-query';

   const query = streamedQuery({
     queryKey: ['stream'],
     queryFn: async function* () {
       // AsyncIterable implementation
     }
   });
   ```

## TanStack Query v5 Migration Considerations

### Breaking Changes from v3 to v5

1. **Package Rename:**
   - `react-query` → `@tanstack/react-query`
   - React 18+ requirement

2. **API Changes:**
   - Single object signature (removed overloads)
   - `isLoading` → `isPending`
   - Removed callbacks (onSuccess, onError)
   - `useErrorBoundary` → `throwOnError`

3. **New Features:**
   - Global error/meta type configuration
   - queryOptions helper
   - Suspense-specific hooks
   - 20% smaller bundle size

### Migration Tools:
- Codemod available for automated migration
- Works for most common patterns
- Manual review required

## Comparison Matrix

| Feature | Orval | Hey API | OpenAPI-Qraft | Zodios | RQ-Codegen | openapi-react-query |
|---------|-------|---------|---------------|--------|------------|-------------------|
| **Adoption** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐ |
| **TanStack Query v5** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Axios Support** | ✅ Native | ✅ Plugin | ⚠️ Custom | ✅ | ✅ | ⚠️ Custom |
| **SSE Support** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Bundle Size** | Large | Medium | Small | Medium | Medium | Tiny (1kb) |
| **Runtime Validation** | ❌ | Via Zod | ❌ | ✅ Native | ❌ | ❌ |
| **Mock Generation** | ✅ MSW | ⚠️ Plugin | ❌ | ❌ | ❌ | ❌ |
| **SSR Support** | ✅ | ✅ | ✅ Excellent | ✅ | ✅ | ✅ |
| **Active Development** | ✅ | ✅✅ Very | ✅ | ⚠️ Slower | ⚠️ | ✅ |
| **Learning Curve** | Medium | Medium | Low | Medium | Low | Very Low |
| **Customization** | High | Very High | Medium | High | Low | Low |

## Recommendations by Use Case

### For Production/Enterprise Applications
**Recommended: Orval**
- Most battle-tested
- Comprehensive features
- Large community support
- Native axios integration

### For Modern Architecture
**Recommended: @hey-api/openapi-ts or OpenAPI-Qraft**
- Hey API for plugin flexibility
- OpenAPI-Qraft for zero-runtime overhead

### For Runtime Validation Requirements
**Recommended: Zodios**
- Zod schema validation
- Type-safe at runtime
- Good for external API integration

### For Bundle-Size Critical Applications
**Recommended: openapi-react-query**
- Only 1kb overhead
- Minimal features
- Simple integration

### For SSE-Heavy Applications
**Recommended: Hybrid Approach**
- Use any tool for regular endpoints
- Keep SSE implementation separate
- Wait for ecosystem maturity

## Implementation Strategy for BodhiApp

### Current Constraints:
1. React Query v3 (needs v5 migration)
2. Axios with interceptors (auth, logging)
3. Critical SSE for chat completions
4. TypeScript type safety requirements
5. MSW v2 for testing

### Recommended Approach: Phased Hybrid Migration

#### Phase 1: TanStack Query v5 Migration (Week 1)
```bash
npm uninstall react-query
npm install @tanstack/react-query@^5
```
- Apply breaking changes
- Keep existing architecture
- Preserve SSE implementation

#### Phase 2: Orval Integration (Weeks 2-3)
```yaml
# orval.config.yaml
bodhi:
  input:
    target: ../openapi.json
  output:
    target: ./src/api/generated
    client: axios-functions
    mode: react-query
    mock: true
    override:
      mutator:
        path: ./src/lib/apiClient.ts
        name: default
```

#### Phase 3: SSE Handling (Week 4)
- Keep existing fetch-based SSE
- Create type-safe wrappers
- Unify error handling

### Why This Strategy?

1. **Risk Mitigation:**
   - Gradual migration
   - Preserve working SSE
   - Maintain axios interceptors

2. **Future Flexibility:**
   - Easy to switch tools later
   - SSE can evolve independently
   - Open to ecosystem improvements

3. **Immediate Benefits:**
   - Type safety improvements
   - Modern TanStack Query features
   - Better developer experience

## Future Considerations

### Ecosystem Monitoring (2025-2026):

1. **OpenAPI Specification:**
   - Watch for SSE standardization
   - OpenAPI 4.0 discussions
   - AsyncAPI integration

2. **Tool Evolution:**
   - Hey API v1.0 stable release
   - Orval SSE support
   - TanStack Query streaming improvements

3. **Alternative Approaches:**
   - GraphQL with subscriptions
   - tRPC for type-safe APIs
   - WebSocket migration for real-time

### Emerging Patterns:

1. **Streaming Support:**
   - TanStack Query streamedQuery maturation
   - React 18 Suspense streaming
   - Edge runtime compatibility

2. **Type Safety Evolution:**
   - TypeScript 5.x improvements
   - Runtime type checking trends
   - Schema-first development

## Conclusion

The OpenAPI to TanStack Query ecosystem has matured significantly but still lacks native SSE support across all solutions. For BodhiApp's requirements, a hybrid approach using Orval for standard endpoints while maintaining custom SSE implementation provides the best balance of:

- **Type safety** through OpenAPI generation
- **Stability** with proven tools
- **Flexibility** for SSE requirements
- **Future-proofing** with TanStack Query v5

The recommended strategy prioritizes working software over perfect architecture, allowing gradual improvement while maintaining critical functionality.

## Appendix: Research Resources

### Official Documentation
- [TanStack Query v5 Docs](https://tanstack.com/query/v5)
- [Orval Documentation](https://orval.dev/)
- [Hey API Documentation](https://heyapi.dev/)
- [OpenAPI-Qraft](https://openapi-qraft.github.io/openapi-qraft/)
- [Zodios Documentation](https://www.zodios.org/)

### GitHub Repositories
- [orval-labs/orval](https://github.com/orval-labs/orval)
- [hey-api/openapi-ts](https://github.com/hey-api/openapi-ts)
- [OpenAPI-Qraft/openapi-qraft](https://github.com/OpenAPI-Qraft/openapi-qraft)
- [ecyrbe/zodios](https://github.com/ecyrbe/zodios)
- [7nohe/openapi-react-query-codegen](https://github.com/7nohe/openapi-react-query-codegen)

### Community Resources
- [TanStack Query Community Projects](https://tanstack.com/query/latest/docs/framework/react/community/community-projects)
- [OpenAPI SSE Discussion](https://github.com/OAI/OpenAPI-Specification/issues/396)
- [NPM Trends Comparison](https://npmtrends.com/orval-vs-@hey-api/openapi-ts)

---

*Research conducted: September 29, 2025*
*Document version: 1.0*
*Status: Complete*