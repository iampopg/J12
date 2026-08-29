# Contributing to J12 Forensic

Thank you for your interest in contributing! This document will help you get started.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Project Structure](#project-structure)
3. [Architecture Overview](#architecture-overview)
4. [Coding Standards](#coding-standards)
5. [Making Changes](#making-changes)
6. [Testing](#testing)
7. [Submitting Changes](#submitting-changes)
8. [Code Review Process](#code-review-process)

---

## Development Setup

### Prerequisites

- **Node.js** 20+ and npm 10+
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Cargo** (comes with Rust)
- **Tauri CLI** (`cargo install tauri-cli`)

### Platform-Specific Requirements

#### macOS
```bash
xcode-select --install
```

#### Windows
- Microsoft Visual Studio C++ Build Tools
- WebView2 Runtime (included with Windows 11)

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/iampopg/J12.git
cd J12

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run in development mode
cd frontend && cargo tauri dev
```

### Development Credentials

Default login credentials for development:
- **Username:** `admin`
- **Password:** `admin123`

---

## Project Structure

```
J12-forensic/
├── src/                         # React frontend
│   ├── components/              # Reusable UI components
│   │   ├── AIChatWidget.tsx
│   │   ├── BookmarkButton.tsx
│   │   ├── EmailDetailModal.tsx
│   │   ├── J12Logo.tsx
│   │   └── RichEmailBodyViewer.tsx
│   ├── views/                   # View components
│   │   ├── artifacts/           # Artifact scanning views
│   │   ├── attachments/         # Attachment management views
│   │   ├── email-list/          # Email list views
│   │   ├── graph/               # Communication graph views
│   │   ├── report/              # Report generation views
│   │   ├── search/              # Search views
│   │   ├── timeline/            # Timeline views
│   │   ├── AISetupPage.tsx
│   │   ├── DocumentationView.tsx
│   │   ├── EntityDiveView.tsx
│   │   ├── EvidenceLockerView.tsx
│   │   ├── FindingsView.tsx
│   │   ├── NotesView.tsx
│   │   ├── SearchView.tsx
│   │   └── TargetProfileView.tsx
│   ├── pages/                   # Top-level pages
│   │   ├── workspace/           # Workspace sub-views
│   │   ├── CaseListPage.tsx
│   │   └── LoginPage.tsx
│   ├── context/                 # React context providers
│   │   └── AcquisitionContext.tsx
│   ├── utils/                   # Utility functions
│   │   └── scanState.ts
│   ├── App.tsx                  # Main app component
│   ├── auth.tsx                 # Authentication logic
│   └── main.tsx                 # Entry point
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── commands/            # Tauri command handlers
│   │   │   ├── ai.rs            # AI commands
│   │   │   ├── analysis.rs      # Analysis commands
│   │   │   ├── artifacts.rs     # Artifact scanning commands
│   │   │   ├── attachments.rs   # Attachment commands
│   │   │   ├── bookmarks.rs     # Bookmark commands
│   │   │   ├── cases.rs         # Case management commands
│   │   │   ├── emails.rs        # Email commands
│   │   │   ├── evidence.rs      # Evidence commands
│   │   │   ├── helpers.rs       # Helper functions
│   │   │   ├── imap.rs          # IMAP commands
│   │   │   ├── pop3.rs          # POP3 commands
│   │   │   └── reports.rs       # Report commands
│   │   ├── analysis/            # Analysis engines
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs          # Email authentication (SPF/DKIM/DMARC)
│   │   │   ├── doc_extractor.rs # Document text extraction
│   │   │   ├── entropy.rs       # File entropy analysis
│   │   │   ├── fts_search.rs    # FTS5 full-text search
│   │   │   ├── heuristics.rs    # Threat heuristics
│   │   │   ├── ocr_engine.rs    # Image OCR
│   │   │   └── threats.rs       # Threat detection
│   │   ├── ai/                  # AI integration
│   │   │   ├── mod.rs
│   │   │   ├── analysis.rs      # AI analysis
│   │   │   ├── chat.rs          # AI chat
│   │   │   ├── context.rs       # AI context management
│   │   │   ├── models.rs        # AI model fetching
│   │   │   ├── plans.rs         # Investigation plans
│   │   │   └── prompts.rs       # AI prompts
│   │   ├── imap_acquisition/    # IMAP client
│   │   │   ├── mod.rs
│   │   │   ├── client.rs        # IMAP protocol client
│   │   │   ├── oauth.rs         # OAuth2 authentication
│   │   │   └── stream.rs        # Email streaming
│   │   ├── db/                  # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs    # Database migrations
│   │   │   ├── schema.rs        # Table definitions
│   │   │   └── utils.rs         # Database utilities
│   │   ├── models.rs            # Rust struct definitions
│   │   ├── parser.rs            # Email parsing (EML/MBOX)
│   │   ├── pst.rs               # PST/OST/MSG parsing
│   │   ├── audit_logger.rs      # Audit trail logging
│   │   ├── bip39_wordlist.rs    # BIP-39 crypto wordlist
│   │   └── main.rs              # Tauri entry point
│   └── Cargo.toml
└── docs/                        # Documentation
    ├── ARCHITECTURE.md
    ├── CONTRIBUTING.md
    ├── ROADMAP.md
    └── ...
│   ├── package.json
│   └── vite.config.ts
│
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── commands/            # Tauri command handlers
│   │   │   ├── analysis.rs      # Analysis commands
│   │   │   ├── artifacts.rs     # Artifact extraction
│   │   │   ├── attachments.rs   # Attachment handling
│   │   │   ├── bookmarks.rs     # Bookmark management
│   │   │   ├── cases.rs         # Case management
│   │   │   ├── emails.rs        # Email operations
│   │   │   ├── evidence.rs      # Evidence handling
│   │   │   ├── imap.rs          # IMAP commands
│   │   │   ├── mod.rs           # Command exports
│   │   │   ├── pop3.rs          # POP3 commands
│   │   │   └── reports.rs       # Report generation
│   │   ├── ai.rs                # AI integration
│   │   ├── analysis.rs          # Email analysis engines
│   │   ├── db.rs                # Database layer
│   │   ├── imap_acquisition.rs  # IMAP acquisition logic
│   │   ├── models.rs            # Data structures
│   │   ├── parser.rs            # Email parsing
│   │   ├── pst.rs               # PST file parsing
│   │   └── main.rs              # Application entry
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/                        # Documentation
│   ├── AI_ARCHITECTURE.md       # AI investigation engine design
│   ├── API_REFERENCE.md         # Backend API documentation
│   ├── DATABASE_REFERENCE.md    # Complete database schema
│   ├── INSTALLATION.md          # Setup guide
│   ├── SYSTEM_AUDIT.md          # Complete system audit
│   └── USER_GUIDE.md            # User manual
│
├── public/                      # Static assets
├── j12-logo-v3.png              # Application logo
├── README.md                    # Project overview
└── LICENSE                      # MIT License
```

---

## Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              J12 FORENSIC                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        FRONTEND (React + TypeScript)                 │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐ │    │
│  │  │  Components  │  │    Views    │  │    Pages    │  │   Utils   │ │    │
│  │  │  (Reusable)  │  │  (Pages)    │  │  (Top-level)│  │ (Helpers) │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘ │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                      │                                       │
│                              Tauri IPC Bridge                                │
│                                      │                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        BACKEND (Rust)                                │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐ │    │
│  │  │  Commands   │  │   Analysis  │  │     AI      │  │  Parser   │ │    │
│  │  │  (Handlers) │  │   Engines   │  │  Integration│  │ (EML/PST) │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘ │    │
│  │                                      │                               │    │
│  │                              ┌─────────────┐                        │    │
│  │                              │  Database   │                        │    │
│  │                              │  (SQLite)   │                        │    │
│  │                              └─────────────┘                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DATA FLOW                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Evidence File ──► Parser ──► Database ──► Analysis ──► Frontend            │
│       │              │           │             │             │              │
│       │              │           │             │             │              │
│       ▼              ▼           ▼             ▼             ▼              │
│  ┌─────────┐   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │  EML    │   │ RawEmail│  │ emails  │  │ Findings│  │  Views  │        │
│  │  MBOX   │──►│ RawAttch│─►│ entities│─►│ Artifacts│─►│ Search  │        │
│  │  PST    │   │         │  │ attach  │  │ Scores  │  │ Timeline│        │
│  │  MSG    │   │         │  │         │  │         │  │ Graph   │        │
│  └─────────┘   └─────────┘  └─────────┘  └─────────┘  └─────────┘        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Component Hierarchy

```
App.tsx
├── LoginPage.tsx
├── CaseListPage.tsx
│   └── CaseWorkspace.tsx
│       ├── EmailListView.tsx
│       │   ├── EmailDetailModal.tsx
│       │   │   ├── RichEmailBodyViewer.tsx
│       │   │   └── BookmarkButton.tsx
│       │   └── BookmarkButton.tsx
│       ├── SearchView.tsx
│       ├── EntityDiveView.tsx
│       ├── TimelineView.tsx
│       ├── GraphView.tsx
│       ├── AttachmentsView.tsx
│       ├── ArtifactsView.tsx
│       ├── FindingsView.tsx
│       ├── NotesView.tsx
│       ├── ReportView.tsx
│       ├── TargetProfileView.tsx
│       ├── EvidenceLockerView.tsx
│       ├── AISetupPage.tsx
│       └── AIChatWidget.tsx
```

---

## Coding Standards

### Rust Standards

1. **Formatting:** Use `rustfmt` with default settings
2. **Linting:** Follow `clippy` warnings
3. **Error Handling:** Use `Result<T, String>` for commands
4. **Naming:** snake_case for functions/variables, PascalCase for types
5. **Comments:** Document all public functions with `///`

```rust
/// Creates a new case in the database
///
/// # Arguments
/// * `state` - Application state containing database connection
/// * `input` - Case creation input with title, description, etc.
///
/// # Returns
/// The created Case on success, error message on failure
#[tauri::command]
pub async fn case_create(
    state: State<'_, AppState>,
    input: CaseCreateInput,
) -> Result<Case, String> {
    // Implementation
}
```

### TypeScript Standards

1. **Formatting:** Use Prettier with default settings
2. **Linting:** Follow ESLint configuration
3. **Types:** Define interfaces for all props and state
4. **Naming:** PascalCase for components, camelCase for functions/variables
5. **Comments:** Document complex logic with JSDoc

```typescript
interface Props {
    /** Email ID to display */
    emailId: string;
    /** Callback when modal closes */
    onClose: () => void;
}

/**
 * Displays email details in a modal dialog
 */
const EmailDetailModal: React.FC<Props> = ({ emailId, onClose }) => {
    // Implementation
};
```

### Database Standards

1. **Naming:** snake_case for tables and columns
2. **Primary Keys:** UUID v4 strings
3. **Timestamps:** ISO 8601 format
4. **JSON:** Store arrays as TEXT with JSON encoding
5. **Indexes:** Add indexes for all foreign keys and frequently queried columns

---

## Making Changes

### Branch Naming

- `feature/description` - New features
- `bugfix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring

### Commit Messages

```
feat: Add new artifact extraction for crypto wallets
fix: Correct risk score calculation for attachments
docs: Update API reference with new endpoints
refactor: Simplify email parsing logic
```

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Update documentation if needed
4. Ensure the app builds and runs
5. Submit PR with clear description
6. Address review comments

---

## Testing

### Running Tests

```bash
# Run Rust tests
cd src-tauri && cargo test

# Run frontend tests
cd frontend && npm test
```

### Test Coverage

- Unit tests for utility functions
- Integration tests for database operations
- Component tests for React components
- End-to-end tests for critical workflows

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        let path = PathBuf::from("test_data/sample.eml");
        let hash = compute_sha256(&path).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    }
}
```

---

## Submitting Changes

### Before Submitting

- [ ] Code follows project standards
- [ ] All tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
- [ ] Commit messages are clear

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Testing
How you tested the changes

## Screenshots (if applicable)
Add screenshots for UI changes
```

---

## Code Review Process

1. All PRs require at least one review
2. Reviewers check for:
   - Code quality and standards
   - Potential bugs or issues
   - Documentation completeness
   - Test coverage
3. Address all review comments
4. Maintainer merges approved PRs

---

## Getting Help

- Check existing documentation in `docs/`
- Review open/closed issues
- Ask in discussions

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
