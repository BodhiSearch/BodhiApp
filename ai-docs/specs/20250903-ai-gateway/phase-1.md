# BodhiApp AI Gateway - Phase 1: Foundation Enhancement (2-Week Sprint)

**Sprint Duration**: 2 weeks (10 working days)  
**Team Size**: 1-2 developers  
**Sprint Goal**: Enhance existing API model infrastructure to support team-based management and cost controls  
**Document Date**: September 3, 2025  

## Current State Analysis

### What We Have Already Built ✅

Based on the codebase analysis, BodhiApp has successfully implemented:

1. **API Model Management Core**
   - `ApiModelAlias` domain object with provider, base_url, models support
   - Database layer with encrypted API key storage (AES-GCM)
   - CRUD operations for API model configurations
   - REST API endpoints with OpenAPI documentation

2. **AI API Service Integration**
   - Test prompt functionality (30 char limit for cost control)
   - Model fetching from providers
   - Chat completion forwarding with streaming support
   - Error handling for rate limits, auth failures

3. **Security Infrastructure**
   - Encrypted credential storage in database
   - Role-based access control (Admin, Manager, PowerUser, User)
   - JWT token authentication system
   - Session management with Tower Sessions

4. **Frontend Management Interface**
   - React-based UI for API model configuration
   - Form validation with test connection
   - Responsive design implementation
   - Delete functionality with confirmation

### What We Can Build on Top (Existing Infrastructure)

- Database service with transaction support
- Authentication middleware with role checking
- Time service abstraction for consistent timestamps
- Error handling system with localization support
- RouterState pattern for service coordination

## Phase 1 Goals: Team Foundation & Cost Controls

### Sprint Objectives

Transform the existing single-user API model system into a team-aware, budget-controlled gateway foundation that:
1. Associates API models with teams/workspaces
2. Tracks usage and costs per request
3. Implements basic budget limits with alerts
4. Provides usage analytics dashboards
5. Adds simple retry mechanism for reliability

### Why These Features First?

- **Builds on existing code**: Extends current `ApiModelAlias` and database structure
- **High customer value**: Teams and budgets are top requested features
- **Low technical risk**: Uses patterns already established in codebase
- **Quick wins**: Visible improvements in 2 weeks
- **Foundation for future**: Sets up data model for advanced features

## Implementation Plan

### Week 1: Data Model & Backend Foundation

#### Day 1-2: Team & Workspace Data Model

**Task 1.1: Extend Database Schema**
```sql
-- Add workspace/team support
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL,
    role TEXT NOT NULL, -- 'owner', 'admin', 'member'
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (workspace_id, user_id)
);

-- Extend api_model_aliases table
ALTER TABLE api_model_aliases 
ADD COLUMN workspace_id TEXT REFERENCES workspaces(id);

-- Add usage tracking table
CREATE TABLE api_usage_logs (
    id TEXT PRIMARY KEY,
    api_model_id TEXT NOT NULL REFERENCES api_model_aliases(id),
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cost_usd DECIMAL(10,6) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add budget limits table
CREATE TABLE workspace_budgets (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id),
    monthly_limit_usd DECIMAL(10,2),
    current_month_usage_usd DECIMAL(10,2) DEFAULT 0,
    alert_threshold_percent INTEGER DEFAULT 80,
    last_reset_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Task 1.2: Extend Domain Objects**
```rust
// In crates/objs/src/workspace.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub workspace_id: String,
    pub user_id: String,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLog {
    pub id: String,
    pub api_model_id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBudget {
    pub workspace_id: String,
    pub monthly_limit_usd: Option<Decimal>,
    pub current_month_usage_usd: Decimal,
    pub alert_threshold_percent: u32,
    pub last_reset_at: Option<DateTime<Utc>>,
}
```

#### Day 3-4: Service Layer Extensions

**Task 1.3: Extend DbService for Workspaces**
```rust
// In crates/services/src/db/service.rs
#[async_trait]
impl DbService for DefaultDbService {
    // Add new methods
    async fn create_workspace(&self, workspace: &Workspace) -> Result<()>;
    async fn get_workspace(&self, id: &str) -> Result<Option<Workspace>>;
    async fn list_user_workspaces(&self, user_id: &str) -> Result<Vec<Workspace>>;
    async fn add_workspace_member(&self, member: &WorkspaceMember) -> Result<()>;
    
    // Usage tracking
    async fn log_api_usage(&self, log: &ApiUsageLog) -> Result<()>;
    async fn get_workspace_usage(&self, workspace_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<ApiUsageLog>>;
    
    // Budget management
    async fn set_workspace_budget(&self, budget: &WorkspaceBudget) -> Result<()>;
    async fn get_workspace_budget(&self, workspace_id: &str) -> Result<Option<WorkspaceBudget>>;
    async fn update_workspace_usage(&self, workspace_id: &str, cost: Decimal) -> Result<()>;
}
```

**Task 1.4: Usage Tracking in AiApiService**
```rust
// Modify crates/services/src/ai_api_service.rs
impl AiApiService for DefaultAiApiService {
    async fn forward_chat_completion(
        &self,
        id: &str,
        request: CreateChatCompletionRequest,
        user_id: &str, // Add user context
        workspace_id: &str, // Add workspace context
    ) -> Result<Response> {
        // Check budget before forwarding
        let budget = self.db_service.get_workspace_budget(workspace_id).await?;
        if let Some(budget) = budget {
            if let Some(limit) = budget.monthly_limit_usd {
                if budget.current_month_usage_usd >= limit {
                    return Err(AiApiServiceError::BudgetExceeded);
                }
            }
        }
        
        // Forward request (existing logic)
        let response = self.forward_to_provider(id, request).await?;
        
        // Extract token usage from response
        let (input_tokens, output_tokens) = extract_token_usage(&response);
        let cost = calculate_cost(model, input_tokens, output_tokens);
        
        // Log usage
        let usage_log = ApiUsageLog {
            id: generate_id(),
            api_model_id: id.to_string(),
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
            model: request.model.clone(),
            input_tokens,
            output_tokens,
            cost_usd: cost,
            created_at: self.time_service.utc_now(),
        };
        
        self.db_service.log_api_usage(&usage_log).await?;
        self.db_service.update_workspace_usage(workspace_id, cost).await?;
        
        // Check if alert needed
        self.check_budget_alert(workspace_id, budget).await?;
        
        Ok(response)
    }
}
```

#### Day 5: Simple Retry Mechanism

**Task 1.5: Add Retry Logic to AiApiService**
```rust
// In crates/services/src/ai_api_service.rs
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: [u64; 3] = [1000, 2000, 4000]; // Exponential backoff

impl DefaultAiApiService {
    async fn forward_with_retry(
        &self,
        url: &str,
        headers: HeaderMap,
        body: String,
    ) -> Result<Response> {
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            match self.client.post(url)
                .headers(headers.clone())
                .body(body.clone())
                .send()
                .await 
            {
                Ok(response) if response.status().is_success() => {
                    return Ok(response);
                }
                Ok(response) if response.status() == StatusCode::TOO_MANY_REQUESTS 
                    || response.status().is_server_error() => {
                    // Retryable error
                    last_error = Some(format!("Attempt {} failed with status {}", attempt + 1, response.status()));
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS[attempt as usize])).await;
                    }
                }
                Ok(response) => {
                    // Non-retryable error
                    return Err(Self::status_to_error(response.status(), response.text().await.unwrap_or_default()));
                }
                Err(e) if is_network_error(&e) => {
                    // Network error, retry
                    last_error = Some(format!("Network error: {}", e));
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS[attempt as usize])).await;
                    }
                }
                Err(e) => {
                    // Non-retryable error
                    return Err(AiApiServiceError::from(e));
                }
            }
        }
        
        Err(AiApiServiceError::MaxRetriesExceeded(last_error.unwrap_or_default()))
    }
}
```

### Week 2: API Extensions & Frontend

#### Day 6-7: API Endpoint Extensions

**Task 2.1: Workspace Management Endpoints**
```rust
// In crates/routes_app/src/routes_workspaces.rs
#[utoipa::path(
    post,
    path = "/api/v1/workspaces",
    operation_id = "createWorkspace",
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
    )
)]
pub async fn create_workspace_handler(
    State(state): State<Arc<dyn RouterState>>,
    Json(payload): Json<CreateWorkspaceRequest>,
    Extension(user_id): Extension<String>, // From auth middleware
) -> Result<Json<WorkspaceResponse>, ApiError> {
    let workspace = Workspace {
        id: generate_id(),
        name: payload.name,
        created_at: state.app_service().time_service().utc_now(),
        updated_at: state.app_service().time_service().utc_now(),
    };
    
    state.app_service().db_service().create_workspace(&workspace).await?;
    
    // Add creator as owner
    let member = WorkspaceMember {
        workspace_id: workspace.id.clone(),
        user_id,
        role: WorkspaceRole::Owner,
        joined_at: state.app_service().time_service().utc_now(),
    };
    
    state.app_service().db_service().add_workspace_member(&member).await?;
    
    Ok(Json(WorkspaceResponse::from(workspace)))
}

// Budget management endpoint
#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/budget",
    operation_id = "setWorkspaceBudget",
    request_body = SetBudgetRequest,
)]
pub async fn set_budget_handler(
    State(state): State<Arc<dyn RouterState>>,
    Path(workspace_id): Path<String>,
    Json(payload): Json<SetBudgetRequest>,
) -> Result<Json<BudgetResponse>, ApiError> {
    // Verify user is admin of workspace
    // ... authorization logic ...
    
    let budget = WorkspaceBudget {
        workspace_id,
        monthly_limit_usd: payload.monthly_limit_usd,
        alert_threshold_percent: payload.alert_threshold_percent.unwrap_or(80),
        current_month_usage_usd: Decimal::zero(),
        last_reset_at: Some(state.app_service().time_service().utc_now()),
    };
    
    state.app_service().db_service().set_workspace_budget(&budget).await?;
    
    Ok(Json(BudgetResponse::from(budget)))
}
```

**Task 2.2: Usage Analytics Endpoints**
```rust
// In crates/routes_app/src/routes_analytics.rs
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/usage",
    operation_id = "getWorkspaceUsage",
    params(
        ("start_date" = Option<String>, Query, description = "Start date ISO 8601"),
        ("end_date" = Option<String>, Query, description = "End date ISO 8601"),
    )
)]
pub async fn get_usage_handler(
    State(state): State<Arc<dyn RouterState>>,
    Path(workspace_id): Path<String>,
    Query(params): Query<UsageQueryParams>,
) -> Result<Json<UsageResponse>, ApiError> {
    let start = params.start_date.unwrap_or_else(|| {
        // Default to start of current month
        Utc::now().with_day(1).unwrap().with_hour(0).unwrap()
    });
    let end = params.end_date.unwrap_or_else(|| Utc::now());
    
    let logs = state.app_service().db_service()
        .get_workspace_usage(&workspace_id, start, end)
        .await?;
    
    // Aggregate by model, user, day
    let summary = UsageSummary {
        total_requests: logs.len(),
        total_cost_usd: logs.iter().map(|l| l.cost_usd).sum(),
        total_input_tokens: logs.iter().map(|l| l.input_tokens).sum(),
        total_output_tokens: logs.iter().map(|l| l.output_tokens).sum(),
        by_model: aggregate_by_model(&logs),
        by_user: aggregate_by_user(&logs),
        by_day: aggregate_by_day(&logs),
    };
    
    Ok(Json(UsageResponse {
        workspace_id,
        start_date: start,
        end_date: end,
        summary,
        logs: logs.into_iter().map(UsageLogResponse::from).collect(),
    }))
}
```

#### Day 8-9: Frontend Dashboard Implementation

**Task 2.3: Workspace Selector Component**
```typescript
// In crates/bodhi/src/components/WorkspaceSelector.tsx
export const WorkspaceSelector: React.FC = () => {
  const { data: workspaces, isLoading } = useQuery({
    queryKey: ['workspaces'],
    queryFn: () => apiClient.workspaces.list(),
  });
  
  const [selectedWorkspace, setSelectedWorkspace] = useWorkspaceStore();
  
  return (
    <Select 
      value={selectedWorkspace?.id}
      onValueChange={(id) => {
        const workspace = workspaces?.find(w => w.id === id);
        setSelectedWorkspace(workspace);
      }}
    >
      <SelectTrigger>
        <SelectValue placeholder="Select workspace" />
      </SelectTrigger>
      <SelectContent>
        {workspaces?.map(workspace => (
          <SelectItem key={workspace.id} value={workspace.id}>
            {workspace.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
```

**Task 2.4: Usage Dashboard Component**
```typescript
// In crates/bodhi/src/components/UsageDashboard.tsx
export const UsageDashboard: React.FC = () => {
  const workspace = useWorkspaceStore(state => state.current);
  const { data: usage, isLoading } = useQuery({
    queryKey: ['usage', workspace?.id],
    queryFn: () => apiClient.analytics.getUsage(workspace.id),
    enabled: !!workspace,
  });
  
  const { data: budget } = useQuery({
    queryKey: ['budget', workspace?.id],
    queryFn: () => apiClient.workspaces.getBudget(workspace.id),
    enabled: !!workspace,
  });
  
  const percentUsed = budget?.monthly_limit_usd 
    ? (usage?.summary.total_cost_usd / budget.monthly_limit_usd) * 100
    : 0;
  
  return (
    <div className="space-y-4">
      {/* Budget Status */}
      <Card>
        <CardHeader>
          <CardTitle>Monthly Budget</CardTitle>
        </CardHeader>
        <CardContent>
          <Progress value={percentUsed} className={percentUsed > 90 ? 'bg-red-500' : ''} />
          <div className="mt-2 flex justify-between text-sm">
            <span>${usage?.summary.total_cost_usd.toFixed(2)} used</span>
            <span>${budget?.monthly_limit_usd?.toFixed(2)} limit</span>
          </div>
        </CardContent>
      </Card>
      
      {/* Usage Chart */}
      <Card>
        <CardHeader>
          <CardTitle>Daily Usage</CardTitle>
        </CardHeader>
        <CardContent>
          <LineChart data={usage?.summary.by_day} />
        </CardContent>
      </Card>
      
      {/* Model Breakdown */}
      <Card>
        <CardHeader>
          <CardTitle>Usage by Model</CardTitle>
        </CardHeader>
        <CardContent>
          <PieChart data={usage?.summary.by_model} />
        </CardContent>
      </Card>
    </div>
  );
};
```

**Task 2.5: Budget Settings Component**
```typescript
// In crates/bodhi/src/components/BudgetSettings.tsx
export const BudgetSettings: React.FC = () => {
  const workspace = useWorkspaceStore(state => state.current);
  const { data: budget } = useQuery({
    queryKey: ['budget', workspace?.id],
    queryFn: () => apiClient.workspaces.getBudget(workspace.id),
  });
  
  const updateBudget = useMutation({
    mutationFn: (data: SetBudgetRequest) => 
      apiClient.workspaces.setBudget(workspace.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries(['budget', workspace.id]);
      toast.success('Budget updated successfully');
    },
  });
  
  const form = useForm<SetBudgetRequest>({
    defaultValues: {
      monthly_limit_usd: budget?.monthly_limit_usd,
      alert_threshold_percent: budget?.alert_threshold_percent || 80,
    },
  });
  
  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(data => updateBudget.mutate(data))}>
        <FormField
          control={form.control}
          name="monthly_limit_usd"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Monthly Budget Limit (USD)</FormLabel>
              <FormControl>
                <Input type="number" step="0.01" {...field} />
              </FormControl>
              <FormDescription>
                Leave blank for unlimited usage
              </FormDescription>
            </FormItem>
          )}
        />
        
        <FormField
          control={form.control}
          name="alert_threshold_percent"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Alert Threshold (%)</FormLabel>
              <FormControl>
                <Slider
                  min={50}
                  max={100}
                  step={10}
                  value={[field.value]}
                  onValueChange={([value]) => field.onChange(value)}
                />
              </FormControl>
              <FormDescription>
                Get alerted when usage reaches this percentage
              </FormDescription>
            </FormItem>
          )}
        />
        
        <Button type="submit">Save Budget Settings</Button>
      </form>
    </Form>
  );
};
```

#### Day 10: Testing & Documentation

**Task 2.6: Integration Tests**
```rust
// In crates/integration-tests/tests/test_workspaces.rs
#[tokio::test]
async fn test_workspace_budget_enforcement() {
    let (app, _temp_dir) = setup_test_app().await;
    let user_id = create_test_user(&app).await;
    
    // Create workspace with budget
    let workspace = create_workspace(&app, &user_id, "Test Workspace").await;
    set_workspace_budget(&app, &workspace.id, 10.0, 80).await;
    
    // Create API model in workspace
    let api_model = create_api_model(&app, &workspace.id, "test-api").await;
    
    // Make requests that exceed budget
    for i in 0..5 {
        let result = forward_chat_completion(&app, &api_model.id, &user_id, &workspace.id).await;
        
        if i < 3 {
            assert!(result.is_ok(), "Request {} should succeed", i);
        } else {
            assert!(matches!(result, Err(e) if e.is_budget_exceeded()), 
                   "Request {} should fail with budget exceeded", i);
        }
    }
    
    // Verify usage was tracked
    let usage = get_workspace_usage(&app, &workspace.id).await;
    assert_eq!(usage.summary.total_requests, 3);
    assert!(usage.summary.total_cost_usd > 0.0);
}

#[tokio::test]
async fn test_retry_mechanism() {
    let (app, _temp_dir) = setup_test_app_with_mock_provider().await;
    
    // Configure mock to fail twice then succeed
    configure_mock_failures(&app, 2).await;
    
    // Request should succeed after retries
    let result = forward_chat_completion(&app, "test-api", "user1", "workspace1").await;
    assert!(result.is_ok());
    
    // Verify retry count
    let metrics = get_request_metrics(&app).await;
    assert_eq!(metrics.retry_count, 2);
}
```

**Task 2.7: API Documentation**
```rust
// Update OpenAPI spec in crates/routes_app/src/openapi.rs
impl BodhiAppOpenApi {
    pub fn openapi() -> utoipa::openapi::OpenApi {
        #[derive(OpenApi)]
        #[openapi(
            paths(
                // Existing paths...
                routes_workspaces::create_workspace_handler,
                routes_workspaces::list_workspaces_handler,
                routes_workspaces::set_budget_handler,
                routes_analytics::get_usage_handler,
                routes_analytics::get_usage_summary_handler,
            ),
            components(schemas(
                Workspace,
                WorkspaceRole,
                WorkspaceBudget,
                ApiUsageLog,
                UsageSummary,
                CreateWorkspaceRequest,
                SetBudgetRequest,
                UsageResponse,
            )),
            tags(
                (name = "Workspaces", description = "Workspace and team management"),
                (name = "Analytics", description = "Usage analytics and reporting"),
            )
        )]
        struct ApiDoc;
        
        ApiDoc::openapi()
    }
}
```

## Testing Strategy

### Unit Tests
- Service layer methods for workspace operations
- Budget calculation and enforcement logic
- Retry mechanism with various failure scenarios
- Usage aggregation functions

### Integration Tests
- End-to-end workspace creation and management
- Budget enforcement across API calls
- Usage tracking accuracy
- Retry behavior with real HTTP calls

### Manual Testing Checklist
- [ ] Create workspace and invite members
- [ ] Set monthly budget and verify enforcement
- [ ] Make API calls and verify usage tracking
- [ ] Check analytics dashboard displays correctly
- [ ] Test alert notifications at threshold
- [ ] Verify retry on transient failures

## Migration & Rollout Plan

### Database Migration
1. Run migration script to add new tables
2. Backfill existing API models with default workspace
3. Migrate existing users to workspace owners

### Feature Flags
```rust
pub struct FeatureFlags {
    pub workspaces_enabled: bool,
    pub budget_enforcement_enabled: bool,
    pub usage_tracking_enabled: bool,
    pub retry_enabled: bool,
}
```

### Gradual Rollout
1. Day 1: Enable for internal testing
2. Day 3: Enable for beta users (5%)
3. Day 5: Enable for 25% of users
4. Day 7: Enable for 50% of users
5. Day 10: Enable for all users

## Success Metrics

### Technical Metrics
- [ ] Zero regression in existing functionality
- [ ] API response time <200ms for analytics queries
- [ ] Budget checks add <10ms latency
- [ ] Retry reduces failures by >50%

### Business Metrics
- [ ] 80% of users create workspace in first week
- [ ] 60% of workspaces set budget limits
- [ ] 30% reduction in surprise overages
- [ ] 90% success rate with retry enabled

## Risk Mitigation

### Technical Risks
1. **Database performance**: Index on workspace_id and created_at
2. **Budget calculation accuracy**: Use Decimal type for money
3. **Retry storms**: Exponential backoff and circuit breaker
4. **Migration failure**: Backup before migration, rollback plan ready

### Business Risks
1. **User confusion**: Clear onboarding flow and documentation
2. **Budget too restrictive**: Soft limits with override option
3. **Analytics overwhelming**: Start with simple metrics
4. **Breaking existing integrations**: Backward compatibility layer

## Next Phase Preview

After successfully completing Phase 1, we'll have the foundation for:

**Phase 2: Advanced Reliability (Weeks 3-4)**
- Provider fallback chains
- Smart caching system
- Load balancing across providers
- Circuit breaker pattern

**Phase 3: Team Collaboration (Weeks 5-6)**
- Prompt template library
- Team-based routing rules
- Shared workspace settings
- Audit logging

## Summary

This 2-week sprint transforms BodhiApp's existing API model system into a team-aware, budget-controlled gateway foundation. By building on the current codebase and focusing on high-value features, we can deliver significant improvements quickly while setting up the architecture for future enhancements.

**Key Deliverables:**
- ✅ Workspace/team management system
- ✅ Usage tracking and analytics
- ✅ Budget limits and alerts
- ✅ Simple retry mechanism
- ✅ Analytics dashboard UI

The implementation is achievable with 1-2 developers and provides immediate value to users who need team coordination and cost control for their AI operations.