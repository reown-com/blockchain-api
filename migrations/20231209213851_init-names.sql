-- Creating the hstore extension for the attributes column
CREATE EXTENSION hstore;

-- Initializing the names table
CREATE TABLE names (
  name VARCHAR(255) PRIMARY KEY,
  registered_at TIMESTAMPTZ NOT NULL  DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  -- We are using the hstore as a key-value for the extensible attributes list
  attributes hstore,

  -- Check for the standartized name format
  CONSTRAINT ens_name_standard CHECK (name ~ '^[a-z0-9.-]*$')
);
