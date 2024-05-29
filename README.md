# Blockchain API

[📚 Documentation](https://docs.walletconnect.com/cloud/blockchain-api)

Blockchain RPC proxy for interacting with multiple EVM and non-EVM blockchains. Also offers higher-levels functions such as ENS resolution and transaction history. We do not run our own blockchain RPC nodes but instead proxy to and load balance across other popular RPC providers.

Not to be confused with the [WalletConnect Relay](https://docs.walletconnect.com/cloud/relay).

## Usage

Endpoint: `https://rpc.walletconnect.com/v1?chainId=eip155:1&projectId=<your-project-id>`

For example:

```bash
curl -X POST "https://rpc.walletconnect.com/v1?chainId=eip155:1&projectId=<your-project-id>" --data '{"id":"1","jsonrpc":"2.0","method":"eth_chainId","params":[]}'
```

Obtain a `projectId` from <https://cloud.walletconnect.com>

See [SUPPORTED_CHAINS.md](./SUPPORTED_CHAINS.md) for which chains we support and which `chainId` to use.

## Development

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) >= 1.56 version,
- [Just](https://github.com/casey/just#packages) as a command runner,
- [Docker](https://www.docker.com/) for building and running Docker containers.

### Building and running

```bash
cp .env.example .env
nano .env
```

```bash
just run
```

```bash
# projectId is not validated under default .env.example configuration
curl -X POST "http://localhost:3000/v1?chainId=eip155:1&projectId=someid" --data '{"id":"1","jsonrpc":"2.0","method":"eth_chainId","params":[]}'
```

## Testing

```bash
just devloop
```

### Docker

```console
$ docker build . --tag rpc-proxy:
$ docker run -p 3000:3000 \
    -e RPC_PROXY_POKT_PROJECT_ID=<some_id> \
    -e RPC_PROXY_INFURA_PROJECT_ID=<some_id> \
    -e RPC_PROXY_QUICKNODE_API_TOKEN=<some_id> \
    -e RPC_PROXY_REGISTRY_API_URL=<registry_url> \
    -e RPC_PROXY_REGISTRY_API_AUTH_TOKEN=<token> \
    --name rpc -it rpc-proxy
```

### Docker Compose

If you need to test with registry caching activated, you can use `docker-compose` to spawn a redis instance for the proxy:

```console
$ RPC_PROXY_POKT_PROJECT_ID=<some_id> \
  RPC_PROXY_INFURA_PROJECT_ID=<some_id> \
  RPC_PROXY_QUICKNODE_API_TOKEN=<some_id> \
  RPC_PROXY_REGISTRY_API_AUTH_TOKEN=<token> \
  docker-compose up
```
