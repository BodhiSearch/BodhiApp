# Architecture Diagrams: Token Refresh Concurrency Control

This document contains Mermaid diagrams illustrating the architecture and flow of the token refresh concurrency control solution.

## System Architecture

### Overall Component Architecture

```mermaid
graph TB
    subgraph "HTTP Layer"
        Request[HTTP Request]
        Response[HTTP Response]
    end

    subgraph "Auth Middleware"
        AuthMiddleware[auth_middleware]
        TokenService[DefaultTokenService]
    end

    subgraph "Services Layer"
        AppService[AppService Registry]
        AuthService[AuthService]
        ConcurrencyService[ConcurrencyControlService]
        SessionService[SessionService]
    end

    subgraph "Implementations"
        LocalImpl[LocalConcurrencyControlService]
        RedisImpl[RedisConcurrencyControlService]
    end

    subgraph "External"
        Keycloak[Keycloak OAuth2 Server]
        Redis[(Redis)]
    end

    Request --> AuthMiddleware
    AuthMiddleware --> TokenService
    TokenService --> ConcurrencyService
    TokenService --> AuthService
    TokenService --> SessionService
    ConcurrencyService --> LocalImpl
    ConcurrencyService -.Phase 2.-> RedisImpl
    AuthService --> Keycloak
    RedisImpl -.-> Redis
    TokenService --> Response
```

## Race Condition Flow (Before Fix)

### Problem: Concurrent Refresh Without Lock

```mermaid
sequenceDiagram
    participant R1 as Request 1
    participant R2 as Request 2
    participant TS as TokenService
    participant Session as Session Store
    participant Auth as AuthService
    participant KC as Keycloak

    Note over R1,KC: Both requests detect expired token simultaneously

    R1->>TS: get_valid_session_token()
    R2->>TS: get_valid_session_token()

    TS->>TS: Check expiration (R1)
    TS->>TS: Check expiration (R2)

    Note over TS: Both see expired token!

    TS->>Session: get refresh_token (R1)
    TS->>Session: get refresh_token (R2)

    Session-->>TS: "ey***og" (R1)
    Session-->>TS: "ey***og" (R2)

    Note over TS,KC: Both use SAME refresh token

    TS->>Auth: refresh_token("ey***og") [R1]
    TS->>Auth: refresh_token("ey***og") [R2]

    Auth->>KC: POST /token [R1]
    Auth->>KC: POST /token [R2]

    KC-->>Auth: 200 OK + new tokens [R1]
    Note over KC: Invalidate "ey***og"

    KC-->>Auth: 400 Token is not active [R2]

    Auth-->>TS: Success [R1]
    Auth-->>TS: Error [R2]

    TS-->>R1: 200 OK
    TS-->>R2: 500 Internal Server Error ❌
```

## Solution Flow (After Fix)

### With Concurrency Control Lock

```mermaid
sequenceDiagram
    participant R1 as Request 1
    participant R2 as Request 2
    participant TS as TokenService
    participant CS as ConcurrencyService
    participant Session as Session Store
    participant Auth as AuthService
    participant KC as Keycloak

    Note over R1,KC: Both requests detect expired token simultaneously

    R1->>TS: get_valid_session_token()
    R2->>TS: get_valid_session_token()

    TS->>TS: Check expiration (R1)
    TS->>TS: Check expiration (R2)

    Note over TS: Both see expired token

    TS->>CS: acquire_lock("user:123:token_refresh") [R1]
    TS->>CS: acquire_lock("user:123:token_refresh") [R2]

    CS-->>TS: Lock acquired ✅ [R1]
    Note over CS: R2 waits for lock...

    TS->>Session: get access_token (R1)
    Session-->>TS: current token (R1)
    TS->>TS: Re-check expiration (R1)
    Note over TS: Still expired, proceed with refresh

    TS->>Session: get refresh_token (R1)
    Session-->>TS: "ey***og" (R1)

    TS->>Auth: refresh_token("ey***og") [R1]
    Auth->>KC: POST /token [R1]
    KC-->>Auth: 200 OK + new tokens [R1]
    Auth-->>TS: Success [R1]

    TS->>Session: save new tokens (R1)
    Session-->>TS: Saved (R1)

    TS-->>R1: 200 OK ✅
    Note over TS: Lock released (R1)

    CS-->>TS: Lock acquired ✅ [R2]

    TS->>Session: get access_token (R2)
    Session-->>TS: NEW token (R2)
    TS->>TS: Re-check expiration (R2)
    Note over TS: Token VALID (refreshed by R1)!

    TS-->>R2: 200 OK (using R1's token) ✅
    Note over TS: Lock released (R2)

    Note over R1,KC: ✅ Success: Single refresh, both requests succeed
```

## ConcurrencyControlService Architecture

### Service Abstraction Design

```mermaid
classDiagram
    class ConcurrencyControlService {
        <<interface>>
        +acquire_lock(key: str) Future~LockGuard~
        +try_acquire_lock(key: str) Future~Option~LockGuard~~
        +acquire_lock_with_timeout(key: str, timeout: Duration) Future~LockGuard~
    }

    class LockGuard {
        <<interface>>
        +key() str
        +acquired_at() DateTime
        +ttl() Duration
    }

    class LocalConcurrencyControlService {
        -locks: HashMap~String, Arc~Mutex~~~
        -default_ttl: Duration
        -time_service: Arc~TimeService~
        -cleanup_task: JoinHandle
        +new(ttl: Duration, time: TimeService) Self
    }

    class RedisConcurrencyControlService {
        -connection_manager: ConnectionManager
        -default_ttl: Duration
        -acquire_script: Script
        -release_script: Script
        +new(redis_url: str, ttl: Duration) Self
    }

    class LocalLockGuard {
        -guard: MutexGuard
        -key: String
        -acquired_at: DateTime
        -ttl: Duration
    }

    class RedisLockGuard {
        -key: String
        -lock_value: String
        -acquired_at: DateTime
        -ttl: Duration
        -connection_manager: ConnectionManager
    }

    ConcurrencyControlService <|.. LocalConcurrencyControlService
    ConcurrencyControlService <|.. RedisConcurrencyControlService
    LockGuard <|.. LocalLockGuard
    LockGuard <|.. RedisLockGuard
    LocalConcurrencyControlService --> LocalLockGuard : creates
    RedisConcurrencyControlService --> RedisLockGuard : creates
```

## Token Service Integration

### DefaultTokenService with ConcurrencyControlService

```mermaid
classDiagram
    class DefaultTokenService {
        -auth_service: Arc~AuthService~
        -secret_service: Arc~SecretService~
        -cache_service: Arc~CacheService~
        -db_service: Arc~DbService~
        -setting_service: Arc~SettingService~
        -concurrency_service: Arc~ConcurrencyControlService~
        +get_valid_session_token(session, token) Future~Result~
    }

    class ConcurrencyControlService {
        <<interface>>
        +acquire_lock_with_timeout(key, timeout) Future~LockGuard~
    }

    class LockGuard {
        <<interface>>
        +key() str
    }

    DefaultTokenService --> ConcurrencyControlService : uses
    ConcurrencyControlService --> LockGuard : returns
```

## Deployment Architectures

### Single Instance Deployment (Local)

```mermaid
graph TB
    subgraph "Single Server/Desktop App"
        App[BodhiApp Process]
        LocalCS[LocalConcurrencyControlService]
        InMemory[In-Memory Lock HashMap]
    end

    subgraph "External Services"
        KC[Keycloak]
    end

    App --> LocalCS
    LocalCS --> InMemory
    App --> KC

    style InMemory fill:#90EE90
    style LocalCS fill:#90EE90
```

### Distributed Deployment (Redis - Phase 2)

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Pod 1"
            App1[BodhiApp Instance 1]
            RedisCS1[RedisConcurrencyControlService]
        end

        subgraph "Pod 2"
            App2[BodhiApp Instance 2]
            RedisCS2[RedisConcurrencyControlService]
        end

        subgraph "Pod 3"
            App3[BodhiApp Instance 3]
            RedisCS3[RedisConcurrencyControlService]
        end
    end

    subgraph "External Services"
        Redis[(Redis Cluster)]
        KC[Keycloak]
    end

    App1 --> RedisCS1
    App2 --> RedisCS2
    App3 --> RedisCS3

    RedisCS1 --> Redis
    RedisCS2 --> Redis
    RedisCS3 --> Redis

    App1 --> KC
    App2 --> KC
    App3 --> KC

    style Redis fill:#FFB6C1
    style RedisCS1 fill:#FFB6C1
    style RedisCS2 fill:#FFB6C1
    style RedisCS3 fill:#FFB6C1
```

## Lock Acquisition State Machine

```mermaid
stateDiagram-v2
    [*] --> Unlocked : Initial State

    Unlocked --> Acquiring : acquire_lock() called
    Acquiring --> Locked : Lock available
    Acquiring --> Waiting : Lock held by another

    Waiting --> Locked : Lock released by holder
    Waiting --> Timeout : Timeout elapsed
    Waiting --> Unlocked : Error occurred

    Locked --> Processing : Critical section executing
    Processing --> Releasing : Operation complete
    Releasing --> Unlocked : Lock released

    Timeout --> [*] : Return error
    Unlocked --> [*] : Success/Failure

    note right of Locked
        Lock automatically released
        when guard is dropped
    end note

    note right of Waiting
        Exponential backoff
        with max wait time
    end note
```

## Double-Check Pattern Flow

```mermaid
flowchart TD
    Start([Token Refresh Request]) --> CheckExpired{Token Expired?}
    CheckExpired -->|No| ReturnValid[Return Valid Token]
    CheckExpired -->|Yes| AcquireLock[Acquire Lock for user:ID]

    AcquireLock --> LockAcquired{Lock Acquired?}
    LockAcquired -->|Timeout| Error[Return Error]
    LockAcquired -->|Yes| DoubleCheck{Re-check Token Expiration}

    DoubleCheck -->|Still Expired| RefreshToken[Call AuthService.refresh_token]
    DoubleCheck -->|Now Valid| ReturnCachedToken[Return Current Token ✅]

    RefreshToken --> RefreshSuccess{Refresh Success?}
    RefreshSuccess -->|No| Error
    RefreshSuccess -->|Yes| SaveSession[Save New Tokens to Session]

    SaveSession --> ReleaseLock1[Release Lock]
    ReleaseLock1 --> ReturnNew[Return New Token ✅]

    ReturnCachedToken --> ReleaseLock2[Release Lock]
    ReleaseLock2 --> ReturnCached[Return Cached Token ✅]

    ReturnValid --> End([End])
    ReturnNew --> End
    ReturnCached --> End
    Error --> End

    style DoubleCheck fill:#FFD700
    style ReturnCachedToken fill:#90EE90
    style RefreshToken fill:#87CEEB
```

## Performance Comparison

### Lock Acquisition Latency by Implementation

```mermaid
graph LR
    subgraph "Local Implementation"
        L1[No Contention<br/>~1-10 μs]
        L2[Low Contention<br/>~10-100 μs]
        L3[High Contention<br/>~100-1000 μs]
    end

    subgraph "Redis Implementation"
        R1[No Contention<br/>~1-5 ms]
        R2[Low Contention<br/>~5-20 ms]
        R3[High Contention<br/>~20-100 ms]
    end

    style L1 fill:#90EE90
    style L2 fill:#90EE90
    style L3 fill:#90EE90
    style R1 fill:#FFB6C1
    style R2 fill:#FFB6C1
    style R3 fill:#FFB6C1
```

## Cleanup Mechanism

### Local Lock Cleanup Task

```mermaid
sequenceDiagram
    participant CT as Cleanup Task
    participant LM as Lock Map
    participant Guard as Lock Guards

    loop Every 30 seconds
        CT->>LM: Iterate lock entries
        LM-->>CT: Entry list

        loop For each entry
            CT->>CT: Check strong_count
            alt strong_count == 1 (only map holds reference)
                CT->>LM: Remove unused lock
                Note over CT: Lock no longer held by any guard
            else strong_count > 1
                CT->>CT: Keep lock (still in use)
                Note over CT: Guard still holds reference
            end
        end
    end

    Note over CT,Guard: Prevents memory leak from abandoned locks
```

## Error Handling Flow

```mermaid
flowchart TD
    Start([Lock Acquisition Attempt]) --> Try{Try Acquire}

    Try -->|Success| Success[Return LockGuard]
    Try -->|Lock Held| Backoff[Exponential Backoff Wait]
    Try -->|Service Error| ServiceError[Return ServiceUnavailable]

    Backoff --> Timeout{Timeout Elapsed?}
    Timeout -->|No| Try
    Timeout -->|Yes| TimeoutError[Return Timeout Error]

    Success --> End([End - Success])
    ServiceError --> End
    TimeoutError --> End

    style Success fill:#90EE90
    style ServiceError fill:#FFB6C1
    style TimeoutError fill:#FFB6C1
```

## Integration Points

### Service Composition in AppService

```mermaid
graph TD
    subgraph "AppService Registry"
        AS[AppService]
        CS[ConcurrencyControlService]
        AuthS[AuthService]
        SecretS[SecretService]
        CacheS[CacheService]
        DbS[DbService]
        SessionS[SessionService]
    end

    subgraph "Middleware"
        TokenS[DefaultTokenService]
    end

    TokenS --> CS
    TokenS --> AuthS
    TokenS --> SecretS
    TokenS --> CacheS
    TokenS --> DbS
    TokenS --> SessionS

    AS -.provides.-> CS
    AS -.provides.-> AuthS
    AS -.provides.-> SecretS
    AS -.provides.-> CacheS
    AS -.provides.-> DbS
    AS -.provides.-> SessionS

    style CS fill:#FFD700
```

---

## Legend

- 🟢 **Green**: Success path / Optimal performance
- 🔴 **Red**: Error path / High latency
- 🟡 **Yellow**: Critical component / Decision point
- 💙 **Blue**: Standard flow / Medium latency

## Rendering

These diagrams use Mermaid syntax and can be rendered in:
- GitHub Markdown
- VS Code with Mermaid extension
- Online Mermaid editors (https://mermaid.live/)
- Documentation generators supporting Mermaid
