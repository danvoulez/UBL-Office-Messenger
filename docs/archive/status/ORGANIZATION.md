# Codebase Organization

**Status**: ✅ Organized into three independent systems with NO code duplication

## Structure Overview

The codebase has been reorganized according to the Flagship Trinity specification with strict separation and no code duplication:

```
OFFICE-main/
├── README.md                    # Root overview
├── ARCHITECTURE.md              # Complete architecture documentation
├── ORGANIZATION.md              # This file
│
├── ubl-messenger/              # System 1: UBL Messenger
│   ├── README.md
│   ├── frontend/                # React/TypeScript UI
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── context/
│   │   │   ├── services/
│   │   │   └── ...
│   │   ├── package.json
│   │   └── vite.config.ts
│   │
│   ├── backend-node/            # Node.js backend (temporary)
│   │   ├── server/
│   │   │   └── index.mjs
│   │   └── package.json
│   │
│   └── backend/                 # Rust backend (target)
│       ├── src/
│       │   ├── conversation/
│       │   ├── message/
│       │   ├── office_client/    # References ../../office/office
│       │   ├── ubl_client/
│       │   └── ...
│       ├── Cargo.toml            # office = { path = "../../office/office" }
│       └── Dockerfile
│
├── office/                      # System 2: Office
│   ├── office/                  # Office Rust code (single source)
│   ├── docker-compose.yml       # Orchestration
│   └── README.md
│       ├── README.md
│       ├── src/
│       │   ├── entity/
│       │   ├── session/
│       │   ├── context/
│       │   ├── governance/
│       │   └── ...
│       └── Cargo.toml
│
└── ubl/                         # System 3: UBL (single source)
    ├── README.md
    ├── kernel/                  # Core Rust implementation
    ├── containers/               # Container logic
    │   ├── C.Messenger/
    │   ├── C.Office/
    │   └── ...
    └── specs/                   # Specifications
```

## No Code Duplication

### ✅ Single Source of Truth

1. **Office**: Only one implementation at `office/office/`
   - Referenced by UBL Messenger backend via path dependency: `office = { path = "../../office/office" }`
   - No duplicate OFFICE code

2. **UBL**: Only one implementation at `ubl/`
   - Used by both OFFICE and Messenger
   - No duplicate UBL code

3. **UBL Messenger Backend**: Only one Rust implementation at `ubl-messenger/backend/`
   - No duplicates exist
   - Single source of truth

### ✅ Type Definitions

- **Frontend types** (`messenger/frontend/types.ts`): TypeScript definitions for UI
- **Backend types** (`ubl-messenger/backend/src/`): Rust structs for API
- **No duplication**: Each language has its own type definitions, but they represent the same contracts

### ✅ Shared Dependencies

- OFFICE is referenced, not copied
- UBL Ledger is referenced, not copied
- Each system is independent but uses shared dependencies via path references

## What Changed

### Before
- No duplicates exist
- Unclear which was the source of truth
- Potential for code drift

### After
- ✅ Single Rust messenger backend at `ubl-messenger/backend/`
- ✅ No duplicates exist
- ✅ OFFICE referenced via path dependency (no duplication)
- ✅ Clear separation: each system has one implementation

## Migration Notes

### Frontend Paths
- All React code in `messenger/frontend/`
- Import paths remain the same (using `@/` alias)
- Vite config updated to reflect new structure

### Backend Paths
- Node.js server in `ubl-messenger/backend-node/`
- Rust backend in `ubl-messenger/backend/`
- Data path updated: `./data/db.json` (was `./server/data/db.json`)
- Package.json scripts updated

### Configuration
- `vite.config.ts`: Updated comments about backend location
- `package.json`: Removed server scripts (backend has its own)
- `Cargo.toml`: Updated Office path to `../../office/office`
- `docker-compose.yml`: Updated to reflect new structure

## Running the Systems

### Development

```bash
# Terminal 1: UBL
cd ubl
cargo run

# Terminal 2: Office
cd office/office
cargo run

# Terminal 3: UBL Messenger Backend (Node.js)
cd ubl-messenger/backend-node
npm install
npm run server

# Terminal 4: UBL Messenger Frontend
cd ubl-messenger/frontend
npm install
npm run dev

# Terminal 5: UBL Messenger Backend (Rust)
cd ubl-messenger/backend
cargo run  # Uses Office from ../../office/office
```

### Production (Docker)

```bash
cd ubl-ecosystem
docker-compose up
```

## File Locations Reference

| Component | Location | Duplication Status |
|-----------|----------|-------------------|
| UBL Messenger Frontend | `ubl-messenger/frontend/` | ✅ Single source |
| UBL Messenger Backend (Node.js) | `ubl-messenger/backend-node/` | ✅ Single source |
| UBL Messenger Backend (Rust) | `ubl-messenger/backend/` | ✅ Single source (duplicate removed) |
| Office | `office/office/` | ✅ Single source (referenced, not copied) |
| UBL | `ubl/` | ✅ Single source (referenced, not copied) |
| Docker Compose | `office/docker-compose.yml` | ✅ Single source |
| Architecture Docs | `ARCHITECTURE.md` | ✅ Single source |

## Dependency Graph

```
ubl-messenger/backend (Rust)
  └── office = { path = "../../office/office" }
      └── (uses UBL via HTTP)

ubl-messenger/frontend (React)
  └── (calls UBL Messenger backend via HTTP)

ubl-messenger/backend-node (Node.js)
  └── (temporary, will be replaced)
```

## Notes

- ✅ **No code duplication**: Each system has a single implementation
- ✅ **Path dependencies**: OFFICE referenced, not copied
- ✅ **Independent deployment**: Each system can be deployed separately
- ✅ **Clear ownership**: Each directory has one purpose
- ✅ **Type safety**: TypeScript types and Rust types represent same contracts but are language-specific

## Verification

To verify no duplication:

```bash
# Check only one OFFICE implementation
ls -la office/office/

# Check only one UBL Messenger Rust backend
ls -la ubl-messenger/backend/

# Verify no duplicate messenger
ls -la office/messenger/  # Should not exist

# Check Cargo.toml references
grep -r "path.*office" ubl-messenger/backend/Cargo.toml
```
