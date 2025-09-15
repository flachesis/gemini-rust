# Agent Development Guide

This document provides guidelines for developing agents and applications using the gemini-rust library.

## Logging

The gemini-rust library uses structured logging with the `tracing` crate for comprehensive observability.

### Setup

Initialize tracing in your main function:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    // Your code here
}
```

### Conventions

**Message formatting**: Use lowercase messages
```rust
info!("starting content generation");  // ✓
info!("Starting content generation");  // ✗
```

**Field naming**: Use dot notation and descriptive boolean flags
```rust
info!(status.code = 200, file.size = 1024, tools.present = true, "request completed");
```

**Value formatting**:
- Strings: `field = value`
- Errors: `error = %err`
- Complex types: `field = ?value`

**Span placeholders**: Define fields in `#[instrument]` and populate with `Span::current().record()`
```rust
#[instrument(skip_all, fields(model, status.code))]
async fn process(&self) -> Result<(), Error> {
    Span::current().record("model", self.model.as_str());
    debug!("processing request");
    // ... operation
    Span::current().record("status.code", 200);
}
```

**Log levels**: `debug!` for details, `info!` for status, `warn!` for issues, `error!` for failures.

### Example

```rust
info!(batch_name = "my-batch", requests.count = 10, "batch started");
error!(error = %err, model = "gemini-2.5-flash", "generation failed");
```