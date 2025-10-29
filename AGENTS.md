# AGENTS:

AI code agents instructions.

## How to add a new Provider

This guide explains how to add a new RPC provider to the blockchain-api. It is intended for AI code agents and contributors. Follow the checklist to implement, register, and ship a provider safely.

References in repo:
- Provider traits and registry: `src/providers/mod.rs`
- Provider env configs: `src/env/` modules and `src/env/mod.rs`
- Provider registration: `src/lib.rs::init_providers`
- Supported chains doc: `SUPPORTED_CHAINS.md`
- Monitoring: `terraform/monitoring/`

Note on consistency: The Base provider shows the standard pattern for HTTP RPC providers. See `src/env/base.rs` and `src/providers/base.rs` for how to define supported chains, implement proxying, and register providers. Keep content-type handling consistent with existing providers and document WS support status explicitly.

### 1) Choose provider scope and kind
- Decide categories supported by this provider:
  - HTTP RPC proxy: implement `RpcProvider`
  - WebSocket RPC proxy: implement `RpcWsProvider` (optional)
- Add or reuse a `ProviderKind` variant:
  - If a new concrete provider, add a new variant in `src/providers/mod.rs` enum `ProviderKind` with `Display` and `from_str` updates.
  - If chain-specific but using generic config, you may not need a new variant (see `GenericProvider`).

### 2) Create env config for the provider
Add a file in `src/env/` named after the provider (e.g., `base.rs` or `acme.rs`). The config must implement `ProviderConfig` and expose supported HTTP and WS chains with weights.

Use `src/env/base.rs` as a reference implementation for:
- Implementing `ProviderConfig`
- Declaring `supported_chains` and `supported_ws_chains`
- Assigning `Weight::new(Priority::...)` per chain

Export the config from `src/env/mod.rs` by adding a `mod <provider>;` and re-export line.
Keep env config defaults in-sync with `SUPPORTED_CHAINS.md`.

### 3) Implement the provider(s)
Add files in `src/providers/` for HTTP and optional WS. Use these references:
- HTTP provider: `src/providers/base.rs` (simple proxy pattern)
- Error and header handling: `src/providers/pokt.rs`
- WebSocket provider: `src/providers/quicknode.rs` (implements `RpcWsProvider`)

WS support is optional; if supported, ensure `supported_ws_chains` is populated in the env config.

Important response handling considerations:
- Preserve or normalize content-type to `application/json` for JSON-RPC envelopes.
- If upstream returns non-JSON bodies for JSON-RPC wrappers, adapt as done in Quicknode TON/Tron helpers.
- Map upstream rate limiting and server errors consistently. Use `is_rate_limited` and interpret JSON-RPC error envelopes similar to `pokt.rs` and `quicknode.rs`.

### 4) Register the provider
Add your provider in `src/providers/mod.rs`:
- Add `mod acme;` and export in the `pub use` section if it’s a concrete provider.
- Ensure `ProviderKind` has your variant if not using `Generic(String)`.

Register in `src/lib.rs::init_providers` similarly to how Base is registered. See `src/lib.rs` for existing calls to `providers.add_rpc_provider::<BaseProvider, BaseConfig>(BaseConfig::default())` and to `add_ws_provider` for providers with WS support.

If your provider requires secrets or API keys, add them to `ProvidersConfig` in `src/providers/mod.rs` and thread them into your config constructor (see Quicknode/Syndica examples). Add corresponding env variables to `src/env/mod.rs` tests and to your deployment environment.

### 5) Update SUPPORTED_CHAINS.md
Add the new chains supported by your provider, including chain IDs and any notable details. Keep it in sync with your env config defaults. Use CAIP identifiers consistently (e.g., `eip155:<id>`, `solana:<genesis_hash>`, `tron:<hex>`).

### 6) Monitoring and weights
- Add weights via `Weight::new(Priority::...)` in your env config for each chain. Higher priority means more traffic selection initially.
- Monitoring auto-adjusts weights from Prometheus. Ensure your provider’s metrics are correctly emitted via the normal proxy path. If new monitoring dashboards are needed, update:
  - `terraform/monitoring/chain_config.json`
  - `terraform/monitoring/dashboard.jsonnet`

### 7) Tests and canaries
- Build and run the server; ensure provider appears in `/v1/supported-chains` and traffic proxies correctly.
- Add or ensure “canary tests” cover your chains per project conventions.
- Validate rate-limit and error-mapping behavior manually for at least one failing request.

### 8) Documentation and PR notes
- Mention chains added and IDs in the PR description.
- Call out whether WS is supported; if not, note that explicitly.
- Ensure env variables and README snippets are updated if new secrets are required.

### 9) Minimal checklist for AI agents
- [ ] Add env config file in `src/env/` implementing `ProviderConfig`
- [ ] Export config in `src/env/mod.rs`
- [ ] Add provider implementation in `src/providers/` implementing required trait(s)
- [ ] Export provider in `src/providers/mod.rs` and update `ProviderKind` if needed
- [ ] Register provider(s) in `src/lib.rs::init_providers`
- [ ] Update `SUPPORTED_CHAINS.md`
- [ ] If needed, extend `ProvidersConfig` and environment variables
- [ ] Verify `/v1/supported-chains` includes new chains (HTTP and WS)
- [ ] Smoke test proxying, including error and rate-limit handling
- [ ] Update monitoring config if required
