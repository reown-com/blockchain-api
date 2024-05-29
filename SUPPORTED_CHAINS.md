# List of supported chains

Chain name with associated `chainId` query param to use.

## HTTP RPC

| Network                                                  | Chain ID             |
|----------------------------------------------------------|----------------------|
| Ethereum                                                 | eip155:1             |
| Optimism                                                 | eip155:10            |
| Binance Smart Chain                                      | eip155:56            |
| Binance Smart Chain Testnet <sup>[1](#footnote1)</sup>   | eip155:97            |
| Gnosis Chain                                             | eip155:100           |
| Polygon                                                  | eip155:137           |
| zkSync Era Sepolia Testnet <sup>[1](#footnote1)</sup>    | eip155:300           |
| zkSync Era                                               | eip155:324           |
| Polygon Zkevm                                            | eip155:1101          |
| Mantle <sup>[1](#footnote1)</sup>                        | eip155:5000          |
| Mantle Testnet <sup>[1](#footnote1)</sup>                | eip155:5001          |
| Klaytn Mainnet                                           | eip155:8217          |
| Base                                                     | eip155:8453          |
| Ethereum Holesky                                         | eip155:17000         |
| Arbitrum                                                 | eip155:42161         |
| Celo                                                     | eip155:42220         |
| Avalanche Fuji Testnet <sup>[1](#footnote1)</sup>        | eip155:43113         |
| Avalanche C-Chain                                        | eip155:43114         |
| Linea <sup>[1](#footnote1)</sup>                         | eip155:59144         |
| Polygon Amoy <sup>[1](#footnote1)</sup>                  | eip155:80002         |
| Base Sepolia                                             | eip155:84532         |
| Arbitrum Sepolia                                         | eip155:421614        |
| Zora <sup>[1](#footnote1)</sup>                          | eip155:7777777       |
| Ethereum Sepolia                                         | eip155:11155111      |
| Optimism Sepolia                                         | eip155:11155420      |
| Zora Sepolia <sup>[1](#footnote1)</sup>                  | eip155:999999999     |
| Aurora <sup>[1](#footnote1)</sup>                        | eip155:1313161554    |
| Aurora Testnet <sup>[1](#footnote1)</sup>                | eip155:1313161555    |
| Near Mainnet                                             | near:mainnet         |
| Solana Mainnet                                           | solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp |

<a id="footnote1"><sup>1</sup></a> The availability of this chain in our RPC is not guaranteed.

## WebSocket RPC

WebSocket RPC **is not recommended for production use**, and may be removed in the future.

| Network            | Chain ID        |
|--------------------|-----------------|
| Ethereum           | eip155:1        |
| Optimism           | eip155:10       |
| Arbitrum           | eip155:42161    |
| Arbitrum Sepolia   | eip155:421614   |
| Zora               | eip155:7777777  |
| Ethereum Sepolia   | eip155:11155111 |
| Optimism Sepolia   | eip155:11155420 |
