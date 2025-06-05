# server_app - Standalone HTTP Server Application

## Overview

The `server_app` crate provides a standalone HTTP server application for BodhiApp. It implements a complete server binary that can run independently, providing all BodhiApp functionality through HTTP APIs without requiring the Tauri desktop application wrapper.

## Purpose

- **Standalone Server**: Independent HTTP server for BodhiApp
- **Production Deployment**: Server deployment for production environments
- **Headless Operation**: Server operation without GUI requirements
- **Service Integration**: Integration with system services and containers
- **Development Server**: Local development server for testing

## Key Components

### Server Management

#### Server Core (`server.rs`)
- **Server Initialization**: HTTP server setup and configuration
- **Service Binding**: Bind to network interfaces and ports
- **TLS Support**: HTTPS configuration and certificate management
- **Performance Tuning**: Server performance optimization

#### Server Runner (`run.rs`)
- **Application Lifecycle**: Complete application startup and shutdown
- **Configuration Loading**: Environment and file-based configuration
- **Service Initialization**: Initialize all application services
- **Error Recovery**: Graceful error handling and recovery

#### Server Serving (`serve.rs`)
- **Request Handling**: HTTP request processing
- **Route Integration**: Integration with routes_all
- **Middleware Stack**: Complete middleware configuration
- **Connection Management**: HTTP connection lifecycle

### Lifecycle Management

#### Shutdown Handling (`shutdown.rs`)
- **Graceful Shutdown**: Clean server shutdown procedures
- **Signal Handling**: SIGTERM, SIGINT signal processing
- **Resource Cleanup**: Proper resource cleanup on shutdown
- **Connection Draining**: Graceful connection termination

#### Interactive Mode (`interactive.rs`)
- **Interactive CLI**: Interactive command-line interface
- **Runtime Commands**: Runtime server management commands
- **Status Monitoring**: Real-time server status display
- **Configuration Updates**: Runtime configuration changes

### Network Management

#### Listener Variants (`listener_variant.rs`)
- **Network Binding**: Different network binding strategies
- **Port Management**: Dynamic port allocation and management
- **Interface Selection**: Network interface selection
- **Protocol Support**: HTTP/HTTPS protocol support

#### Keep-Alive Management (`listener_keep_alive.rs`)
- **Connection Keep-Alive**: HTTP keep-alive connection management
- **Timeout Handling**: Connection timeout configuration
- **Resource Optimization**: Connection pooling and reuse
- **Performance Monitoring**: Connection performance metrics

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── server.rs                 # Core server implementation
├── run.rs                    # Application runner
├── serve.rs                  # HTTP serving logic
├── shutdown.rs               # Graceful shutdown handling
├── interactive.rs            # Interactive CLI mode
├── listener_variant.rs       # Network listener variants
├── listener_keep_alive.rs    # Connection keep-alive management
├── error.rs                  # Server-specific errors
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    ├── mod.rs
    └── interactive.rs
```

## Key Features

### Production Ready
- **High Performance**: Optimized for production workloads
- **Scalability**: Handles high concurrent request loads
- **Reliability**: Robust error handling and recovery
- **Monitoring**: Built-in health checks and metrics

### Configuration Management
- **Environment Variables**: Environment-based configuration
- **Configuration Files**: YAML/TOML configuration support
- **Runtime Updates**: Dynamic configuration updates
- **Validation**: Configuration validation and defaults

### Security Features
- **TLS/HTTPS**: Full TLS support with certificate management
- **Security Headers**: Comprehensive security header configuration
- **Rate Limiting**: Built-in rate limiting and DDoS protection
- **Access Control**: IP-based access control and filtering

### Operational Features
- **Logging**: Structured logging with multiple output formats
- **Metrics**: Prometheus-compatible metrics export
- **Health Checks**: Kubernetes-compatible health endpoints
- **Graceful Shutdown**: Zero-downtime deployment support

## Server Configuration

### Basic Configuration
```yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
  
tls:
  enabled: true
  cert_file: "/path/to/cert.pem"
  key_file: "/path/to/key.pem"
  
logging:
  level: "info"
  format: "json"
  output: "stdout"
```

### Advanced Configuration
```yaml
performance:
  max_connections: 10000
  keep_alive_timeout: 60
  request_timeout: 30
  
security:
  rate_limit:
    requests_per_minute: 1000
    burst: 100
  cors:
    origins: ["https://example.com"]
    methods: ["GET", "POST"]
    
monitoring:
  metrics_enabled: true
  health_check_interval: 30
  tracing_enabled: true
```

## Usage Patterns

### Basic Server Startup
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_env()?;
    let server = Server::new(config).await?;
    server.run().await?;
    Ok(())
}
```

### Interactive Mode
```rust
async fn run_interactive() -> Result<(), ServerError> {
    let server = Server::new(config).await?;
    let interactive = InteractiveMode::new(server);
    interactive.run().await
}
```

### Graceful Shutdown
```rust
async fn run_with_shutdown() -> Result<(), ServerError> {
    let server = Server::new(config).await?;
    let shutdown_signal = shutdown::create_shutdown_signal();
    
    tokio::select! {
        result = server.run() => result,
        _ = shutdown_signal => {
            server.shutdown().await
        }
    }
}
```

## Dependencies

### Core Dependencies
- **routes_all**: Complete route composition
- **server_core**: HTTP server infrastructure
- **services**: All business logic services
- **objs**: Domain objects and configuration

### HTTP Server
- **axum**: HTTP framework
- **hyper**: HTTP implementation
- **tower**: Middleware and service abstractions
- **tokio**: Async runtime

### Configuration and Logging
- **serde**: Configuration serialization
- **tracing**: Structured logging and tracing
- **clap**: Command-line argument parsing

## Command-Line Interface

### Server Commands
```bash
# Start server with default configuration
bodhi-server

# Start with custom configuration
bodhi-server --config /path/to/config.yaml

# Start in interactive mode
bodhi-server --interactive

# Start with specific host and port
bodhi-server --host 127.0.0.1 --port 3000

# Start with TLS
bodhi-server --tls --cert cert.pem --key key.pem
```

### Interactive Commands
When running in interactive mode:
```
> status          # Show server status
> config          # Show current configuration
> reload          # Reload configuration
> shutdown        # Graceful shutdown
> metrics         # Show performance metrics
> help            # Show available commands
```

## Deployment Patterns

### Docker Deployment
```dockerfile
FROM rust:alpine AS builder
COPY . .
RUN cargo build --release --bin bodhi-server

FROM alpine:latest
COPY --from=builder /target/release/bodhi-server /usr/local/bin/
EXPOSE 8080
CMD ["bodhi-server"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bodhi-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: bodhi-server
  template:
    metadata:
      labels:
        app: bodhi-server
    spec:
      containers:
      - name: bodhi-server
        image: bodhi-server:latest
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
```

### Systemd Service
```ini
[Unit]
Description=Bodhi Server
After=network.target

[Service]
Type=simple
User=bodhi
ExecStart=/usr/local/bin/bodhi-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Monitoring and Observability

### Health Checks
- **Liveness Probe**: `/health/live` - Server is running
- **Readiness Probe**: `/health/ready` - Server is ready to serve traffic
- **Detailed Health**: `/health/detailed` - Comprehensive health information

### Metrics Export
- **Prometheus Metrics**: `/metrics` endpoint
- **Custom Metrics**: Application-specific metrics
- **Performance Metrics**: Request latency, throughput, error rates
- **Resource Metrics**: Memory usage, CPU usage, connection counts

### Logging
- **Structured Logging**: JSON-formatted logs
- **Request Tracing**: Distributed tracing support
- **Error Logging**: Comprehensive error logging
- **Audit Logging**: Security and access audit logs

## Performance Optimization

### Connection Management
- **Keep-Alive**: HTTP keep-alive optimization
- **Connection Pooling**: Efficient connection reuse
- **Timeout Management**: Proper timeout configuration
- **Backpressure**: Request backpressure handling

### Resource Management
- **Memory Optimization**: Efficient memory usage
- **CPU Optimization**: CPU-efficient request processing
- **I/O Optimization**: Async I/O for all operations
- **Caching**: Response and data caching

## Testing Support

### Integration Testing
- **Server Testing**: Full server integration tests
- **Load Testing**: Performance and load testing
- **Configuration Testing**: Configuration validation tests
- **Deployment Testing**: Deployment scenario testing

### Test Utilities
```rust
pub async fn create_test_server() -> TestServer {
    let config = test_server_config();
    TestServer::new(config).await
}

pub async fn test_request(
    server: &TestServer,
    request: Request,
) -> Response {
    server.handle_request(request).await
}
```

## Future Extensions

The server_app crate is designed to support:
- **Clustering**: Multi-node server clustering
- **Load Balancing**: Built-in load balancing capabilities
- **Auto-scaling**: Automatic scaling based on load
- **Advanced Monitoring**: Enhanced monitoring and alerting
- **Plugin System**: Dynamic plugin loading and management
