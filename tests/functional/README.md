# Functional integration tests

The following functional integration tests are presented:

* Database tests
* Providers tests
  * Providers functional tests should be `#[ignore]` by default, because they will run by 
    the CI workflow specifically when the providers code is changed in the `src/provider`
    directory.
  * Providers test names should be in the format `{provider_name}_provider` and 
    `{provider_name}_provider_*` aligning with the provider name file in the 
    `src/providers` directory.
  * Example for the `coinbase` provider:
    * Implementation source file: `src/provider/coinbase.rs`
      Tests for the `coinbase` provider will run only if this file is changed.
    * Tests implementation for the `coinbase` provider can be in any files but should be
      `#[ignore]` by default and the test names must starts with the 
      `coinbase_provider`.
