# Contributing to UBL 3.0

> **Truth is not what you say. Truth is what you can prove.**

Thank you for considering contributing to UBL! This document outlines how to contribute effectively.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Submitting Changes](#submitting-changes)
- [Coding Standards](#coding-standards)
- [Architecture Guidelines](#architecture-guidelines)

---

## Code of Conduct

- Be respectful and constructive
- Focus on the code, not the person
- Document your decisions
- All contributions must maintain the security model

---

## Getting Started

### Prerequisites

- **Rust 1.75+** — `rustup`, `cargo`
- **PostgreSQL 16+** — with Unix socket support
- **Node.js 20+** — for frontend and TypeScript components
- **Git** — version control

### Fork & Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/OFFICE.git
cd OFFICE
```

---

## Development Setup

### 1. UBL Kernel (Rust)

```bash
cd ubl/kernel/rust

# Build
cargo build

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features
```

### 2. PostgreSQL

```bash
# Create database
createdb ubl_dev

# Apply migrations
psql ubl_dev -f ../../../sql/000_unified.sql
psql ubl_dev -f ../../../sql/030_console_complete.sql

# Set environment
export DATABASE_URL=postgres:///ubl_dev
```

### 3. Messenger Frontend

```bash
cd apps/messenger/frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

---

## Making Changes

### Branch Naming

```
feature/short-description
fix/issue-number-description
docs/what-changed
refactor/component-name
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(kernel): add pact validation for Evolution commits
fix(messenger): resolve WebSocket reconnection issue
docs(specs): clarify ubl-atom canonicalization rules
refactor(office): extract constitution middleware
```

### What to Include

- **Code changes** — well-tested and documented
- **Tests** — unit tests for new functionality
- **Documentation** — update relevant docs
- **Migration** — SQL migration if schema changes

---

## Submitting Changes

### Pull Request Process

1. **Create a branch** from `main`
2. **Make your changes** following the coding standards
3. **Run tests** — `cargo test` and `npm test`
4. **Update documentation** if needed
5. **Push to your fork**
6. **Open a Pull Request** with a clear description

### PR Requirements

- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`, `prettier`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Commit messages follow convention

### Review Process

- PRs require at least one approval
- Security-related changes require additional review
- Breaking changes must be discussed in an issue first

---

## Coding Standards

### Rust

```rust
// Use descriptive names
pub fn validate_link_signature(link: &UblLink) -> Result<(), ValidationError> {
    // Implementation
}

// Document public functions
/// Validates the cryptographic signature of a UBL link.
///
/// # Arguments
/// * `link` - The link to validate
///
/// # Returns
/// * `Ok(())` if signature is valid
/// * `Err(ValidationError)` if validation fails
pub fn validate_link_signature(link: &UblLink) -> Result<(), ValidationError> {
    // ...
}

// Handle errors properly - no unwrap() in production code
let result = some_operation().map_err(|e| AppError::from(e))?;
```

### TypeScript

```typescript
// Use strict types
interface UblLink {
  container_id: string;
  sequence: number;
  atom_hash: string;
  // ...
}

// Async/await with proper error handling
async function fetchBootstrap(): Promise<BootstrapResponse> {
  try {
    const response = await api.get('/messenger/bootstrap');
    return response;
  } catch (error) {
    throw new ApiError('Bootstrap failed', error);
  }
}
```

### SQL

```sql
-- Use lowercase for keywords
-- Use snake_case for identifiers
CREATE TABLE projection_messages (
    message_id      TEXT        PRIMARY KEY,
    conversation_id TEXT        NOT NULL,
    from_id         TEXT        NOT NULL,
    content_hash    TEXT        NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL,
    
    CONSTRAINT fk_conversation 
        FOREIGN KEY (conversation_id) 
        REFERENCES projection_conversations(conversation_id)
);

-- Always include comments for complex logic
CREATE INDEX idx_messages_conversation_time 
    ON projection_messages (conversation_id, timestamp DESC);
```

---

## Architecture Guidelines

### Core Principles

1. **Append-only** — Never modify or delete ledger entries
2. **Cryptographic proof** — All commits must be signed
3. **Containers are sovereign** — No state sharing between containers
4. **Mind/Body separation** — TypeScript for semantics, Rust for validation

### Security Requirements

- **Never** expose PostgreSQL externally (Unix socket only)
- **Always** validate signatures before accepting commits
- **Never** allow LLMs to bypass the constitution middleware
- **Always** use pacts for Evolution/Entropy operations

### Adding New Features

1. **Read the specs** — `/ubl/specs/` contains frozen specifications
2. **Check existing ADRs** — `/docs/adrs/` for architectural decisions
3. **Create an ADR** if making significant changes
4. **Follow container patterns** — boundary, inbox, local, outbox, projections

### Container Structure

```
containers/C.YourContainer/
├── README.md           # Container documentation
├── boundary/           # Public API interface
├── inbox/              # Incoming events
├── local/              # Private state
├── outbox/             # Outgoing events
├── projections/        # Derived views
├── pacts/              # Authority definitions
├── policy/             # Validation rules
└── tests/              # Container tests
```

---

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Join discussions in pull requests

---

**Made with ❤️ for trustworthy business operations**

