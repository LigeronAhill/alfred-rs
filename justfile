# https://just.systems

set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
set dotenv-load := true
set quiet := true

# View available commands
default:
	just --list


# <---------- DATABASE ---------->
# Start database
db-start:
	docker-compose up -d

# Stop database
db-stop:
	docker-compose down

# Create DB
[working-directory: "pgdb"]
db-create:
	sqlx database create

# Drop DB
[working-directory: "pgdb"]
db-drop:
	sqlx database drop

# Add migration
[working-directory: "pgdb"]
add-migration MIGRATION:
	sqlx migrate add -r {{MIGRATION}}

# Run migrations
[working-directory: "pgdb"]
migrate:
	sqlx migrate run

# Revert migration
[working-directory: "pgdb"]
revert:
	sqlx migrate revert

# Offline
[working-directory: "pgdb"]
offline:
	cargo sqlx prepare

# <---------- TESTS ---------->
# Run tests
test:
	cargo test

# Test coverage
cover:
	cargo llvm-cov

# <---------- DEPENDENCIES ---------->
# Install dependencies
deps-install:
	rustup update
	cargo install cargo-udeps --locked
	cargo install cargo-audit --features=fix
	cargo +stable install cargo-llvm-cov --locked
	cargo install --locked cargo-deny
	cargo install sqlx-cli --no-default-features --features postgres,rustls
	cargo deny init

# <---------- LINTING ---------->
# Check unused crates
unused:
	cargo +nightly udeps --all-targets

# Format code
fmt:
	cargo fmt -- --emit=files

# Run linter
lint:
	cargo clippy --fix --allow-dirty --allow-staged

# Run security audit
sec:
	cargo audit --ignore RUSTSEC-2023-0071

# Check dependencies
deps-check:
	cargo deny check

# <---------- PREPARE FOR PUSH ---------->
# Prepare for push
prepare: test fmt lint unused sec deps-check
