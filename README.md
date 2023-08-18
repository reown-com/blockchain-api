# Blockchain API

WalletConnect's Blockchain API. We do not run our own RPC nodes but instead proxy to and load balance across other popular RPC providers.

## Usage

Endpoint: `https://rpc.walletconnecct.com/v1?chainId=eip155:1&projectId=<your-project-id>`

Obtain a `projectId` from <https://cloud.walletconnect.com>

See [SUPPORTED_CHAINS.md](./SUPPORTED_CHAINS.md) for which chains we support and which `chainId` to use.

## Development

```bash
cp .env.example .env
nano .env
```

```bash
just run
```

```bash
curl -X POST "http://localhost:3000/v1?chainId=eip155:5&projectId=someid" -d '{"id":"1660887896683","jsonrpc":"2.0","method":"eth_chainId","params":[]}'
```

## Testing

```bash
just amigood
```

### Docker

```console
$ docker build . --tag rpc-proxy:
$ docker run -p 3000:3000 \
    -e RPC_PROXY_POKT_PROJECT_ID=<some_id> \
    -e RPC_PROXY_INFURA_PROJECT_ID=<some_id> \
    -e RPC_PROXY_REGISTRY_API_URL=<registry_url> \
    -e RPC_PROXY_REGISTRY_API_AUTH_TOKEN=<token> \
    --name rpc -it rpc-proxy
```

### Docker Compose

If you need to test with registry caching activated, you can use `docker-compose` to spawn a redis instance for the proxy:

```console
$ RPC_PROXY_POKT_PROJECT_ID=<some_id> \
  RPC_PROXY_INFURA_PROJECT_ID=<some_id> \
  RPC_PROXY_REGISTRY_API_AUTH_TOKEN=<token> \
  docker-compose up
```
