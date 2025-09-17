# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the WalletConnect Blockchain API (formerly RPC Proxy), a Rust-based service that acts as an RPC proxy for interacting with multiple EVM and non-EVM blockchains. The service provides blockchain RPC proxying, load balancing across providers, and higher-level functions like ENS resolution and transaction history.

## Development Commands

The project uses `just` as the primary command runner. All commands are defined in the `justfile`:

### Primary Development Commands
- `just run` - Build and run the service locally (requires `.env` file)
- `just devloop` - Run linting and tests (primary development workflow)
- `just build` - Build the project (`cargo build`)
- `just test all` - Run all tests with full features (`cargo test --features=full`)
- `just lint` - Run all linting (clippy, fmt check, udeps)

### Rust-Specific Commands
- `just cargo-clippy` - Run clippy with full features and tests
- `just cargo-fmt` - Format code with rustfmt
- `just cargo-check` - Fast compile check
- `just cargo-udeps` - Check for unused dependencies (requires nightly)

### Integration Testing
- `yarn integration` - Run integration tests (requires `RPC_URL` and `PROJECT_ID` env vars)
- Use `RPC_URL=http://localhost:3000` for local testing
- Integration tests are in the `integration/` directory

### Infrastructure
- `just docker` - Start PostgreSQL and Redis containers
- `just tf <command>` - Terraform commands for infrastructure management

## Code Architecture

### Core Components
- **`src/lib.rs`** - Main application bootstrap and service setup
- **`src/main.rs`** - Entry point with configuration loading
- **`src/handlers/`** - HTTP request handlers for different API endpoints
- **`src/providers/`** - Blockchain RPC provider implementations (Pokt, Quicknode, etc.)
- **`src/storage/`** - Storage backends (Redis, IRN, PostgreSQL)
- **`src/project/`** - Project/registry management and metrics
- **`src/analytics/`** - Analytics event tracking and data collection
- **`src/names/`** - Name resolution services (ENS, etc.)

### Key Architectural Patterns
- Uses Axum for HTTP server with middleware layers
- Provider pattern for different RPC backends with load balancing
- Redis-based caching and rate limiting
- PostgreSQL for persistent storage
- Prometheus metrics integration
- Async/await throughout with Tokio runtime

### Configuration
- Environment variables defined in `.env.example`
- Provider API keys required for full functionality
- PostgreSQL connection string required
- See `src/env/` for configuration structure

### Provider System
The service supports multiple blockchain providers:
- Pokt Network, Quicknode, Coinbase, Zerion, Allnodes
- Provider-specific configurations in `src/env/`
- Load balancing and failover handling in `src/providers/`

### Testing Strategy
- Unit tests alongside source files (`#[cfg(test)]`)
- Integration tests in `integration/` directory using Jest/TypeScript
- Functional tests in `tests/` directory
- Use `--features=full` for complete test coverage

## Environment Setup

1. Copy `.env.example` to `.env`
2. Configure required provider API keys
3. Set PostgreSQL connection string
4. Use `just docker` to start local dependencies
5. Run `just run` to start the service

## Chain Support

Supported chains are documented in `SUPPORTED_CHAINS.md`. The service supports both EVM and non-EVM chains including Ethereum, Polygon, Solana, and others.