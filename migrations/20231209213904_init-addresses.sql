-- List of supported blockchain namespaces
CREATE TYPE namespaces AS ENUM (
  'eip155' -- Ethereum
);

-- Initializing the addresses table
CREATE TABLE addresses (
  -- Breakdown of the CAP-10 address format into namespace:chain_id:address
  namespace namespaces,
  /*
  chain_id can represent a chain id e.g. (Cosmos and cosmoshub-3 chain):
    cosmos:cosmoshub-3:cosmos1t2uflqwqe0fsj0shcfkrvpukewcw40yjj6hdc0
  chain_id can be empty e.g. (Litecoin mainnet):
    bip122:12a765e31ffd4059bada1e25190f6e98
  */
  chain_id VARCHAR(255) NOT NULL,
  address VARCHAR(255) NOT NULL,

  name  VARCHAR(255) REFERENCES  names (name) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  PRIMARY KEY (name, namespace, chain_id, address)
);

-- Creating indexes for the address lookups
CREATE INDEX index_cap_10_format_address
  ON addresses (namespace, chain_id, address);
CREATE INDEX index_namespace_address
  ON addresses (namespace, address);
CREATE INDEX index_address
  ON addresses (address);
CREATE INDEX index_name
  ON addresses (name);
