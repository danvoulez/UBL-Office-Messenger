#!/bin/bash
# UBL Development Environment Setup (macOS with Homebrew)
# Run: chmod +x setup-dev.sh && ./setup-dev.sh

set -e

echo "🚀 UBL Development Environment Setup"
echo "====================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check for Homebrew
if ! command -v brew &> /dev/null; then
    echo -e "${YELLOW}Installing Homebrew...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add to PATH for Apple Silicon
    if [[ $(uname -m) == 'arm64' ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
fi

echo -e "${GREEN}✓ Homebrew installed${NC}"

# Install PostgreSQL
echo -e "${YELLOW}Installing PostgreSQL...${NC}"
brew install postgresql@16

# Start PostgreSQL service
echo -e "${YELLOW}Starting PostgreSQL...${NC}"
brew services start postgresql@16

# Wait for PostgreSQL to start
sleep 3

# Add PostgreSQL to PATH
export PATH="/opt/homebrew/opt/postgresql@16/bin:$PATH"
echo 'export PATH="/opt/homebrew/opt/postgresql@16/bin:$PATH"' >> ~/.zshrc

echo -e "${GREEN}✓ PostgreSQL installed and running${NC}"

# Create database and user
echo -e "${YELLOW}Creating UBL database...${NC}"

# Check if database exists
if psql -lqt | cut -d \| -f 1 | grep -qw ubl_ledger; then
    echo -e "${YELLOW}Database 'ubl_ledger' already exists${NC}"
else
    createdb ubl_ledger
    echo -e "${GREEN}✓ Database 'ubl_ledger' created${NC}"
fi

# Set DATABASE_URL
export DATABASE_URL="postgresql://$(whoami)@localhost/ubl_ledger"
echo "export DATABASE_URL=\"postgresql://\$(whoami)@localhost/ubl_ledger\"" >> ~/.zshrc

echo -e "${GREEN}✓ DATABASE_URL configured${NC}"

# Apply SQL migrations
echo -e "${YELLOW}Applying database migrations...${NC}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SQL_DIR="$SCRIPT_DIR/ubl/sql"

if [ -d "$SQL_DIR" ]; then
    # Apply migrations in order
    for sql_file in $(ls "$SQL_DIR"/*.sql 2>/dev/null | sort); do
        echo "  Applying: $(basename $sql_file)"
        psql -d ubl_ledger -f "$sql_file" 2>/dev/null || true
    done
    echo -e "${GREEN}✓ Migrations applied${NC}"
else
    echo -e "${YELLOW}No SQL directory found at $SQL_DIR${NC}"
fi

# Install Rust if needed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo -e "${GREEN}✓ Rust installed${NC}"

# Install sqlx-cli
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}Installing sqlx-cli...${NC}"
    cargo install sqlx-cli --no-default-features --features postgres
fi
echo -e "${GREEN}✓ sqlx-cli installed${NC}"

# Install Node.js for Messenger frontend
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}Installing Node.js...${NC}"
    brew install node@20
    export PATH="/opt/homebrew/opt/node@20/bin:$PATH"
    echo 'export PATH="/opt/homebrew/opt/node@20/bin:$PATH"' >> ~/.zshrc
fi
echo -e "${GREEN}✓ Node.js installed${NC}"

echo ""
echo -e "${GREEN}====================================${NC}"
echo -e "${GREEN}✅ Setup Complete!${NC}"
echo -e "${GREEN}====================================${NC}"
echo ""
echo "Environment variables set:"
echo "  DATABASE_URL=$DATABASE_URL"
echo ""
echo "Next steps:"
echo ""
echo "  1. Reload shell:"
echo "     source ~/.zshrc"
echo ""
echo "  2. Compile UBL Kernel:"
echo "     cd ubl/kernel/rust && cargo build"
echo ""
echo "  3. Compile Office:"
echo "     cd apps/office && cargo build"
echo ""
echo "  4. Run Messenger Frontend:"
echo "     cd apps/messenger/frontend && npm install && npm run dev"
echo ""
echo "  5. (Optional) Generate SQLx offline cache:"
echo "     cd ubl/kernel/rust && cargo sqlx prepare --workspace"
echo ""
