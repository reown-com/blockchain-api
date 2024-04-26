# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## 0.58.0 - 2024-04-26
#### Features
- **(providers)** adding Polygon Amoy testnet (#647) - (2159ebd) - Max Kalashnikoff

- - -

## 0.57.0 - 2024-04-24
#### Features
- **(1inch)** handling of the unsupported chain ID (#646) - (73f0ba4) - Max Kalashnikoff

- - -

## 0.56.0 - 2024-04-23
#### Features
- **(analytics)** adding analytics for balance lookups (#632) - (53d5d62) - Max Kalashnikoff

- - -

## 0.55.6 - 2024-04-22
#### Bug Fixes
- **(utils)** properly handling of malformed CAIP-2 and 10 (#643) - (29c4018) - Max Kalashnikoff

- - -

## 0.55.5 - 2024-04-22
#### Bug Fixes
- **(Zerion)** return only non-spam items (#642) - (e5d99ba) - Max Kalashnikoff

- - -

## 0.55.4 - 2024-04-19
#### Bug Fixes
- **(Zerion)** handling of null price response (#641) - (6706f1c) - Max Kalashnikoff

- - -

## 0.55.3 - 2024-04-19
#### Bug Fixes
- **(fungibles)** adding more exception to native tokens (#640) - (809e041) - Max Kalashnikoff

- - -

## 0.55.2 - 2024-04-19
#### Bug Fixes
- **(fungibles)** adding eth token address representation (#639) - (fc69496) - Max Kalashnikoff

- - -

## 0.55.1 - 2024-04-18
#### Bug Fixes
- **(conversion)** making `amount` optional in approval (#638) - (5f90aa4) - Max Kalashnikoff

- - -

## 0.55.0 - 2024-04-18
#### Features
- **(conversion)** allowance endpoint (#637) - (68fc5bf) - Max Kalashnikoff

- - -

## 0.54.0 - 2024-04-18
#### Features
- **(conversion)** implementing gas price endpoint (#636) - (e97c8e8) - Max Kalashnikoff

- - -

## 0.53.0 - 2024-04-17
#### Features
- **(conversion)** passing through error message in case of wrong parameter (#635) - (ff6f993) - Max Kalashnikoff

- - -

## 0.52.3 - 2024-04-17
#### Bug Fixes
- **(conversion)** changing amounts to be String (#634) - (16981a6) - Max Kalashnikoff

- - -

## 0.52.2 - 2024-04-17
#### Bug Fixes
- **(names)** removing of eth mainnet addresses only (#633) - (a1b18bb) - Max Kalashnikoff

- - -

## 0.52.1 - 2024-04-16
#### Bug Fixes
- **(balance)** respond with an empty balance for sdk version <= 4.1.8 (#631) - (2d5c73f) - Max Kalashnikoff

- - -

## 0.52.0 - 2024-04-15
#### Bug Fixes
- **(tests)** fixing error code in identity test with wrong project ID (#629) - (311a5eb) - Max Kalashnikoff
#### Features
- adding sdk type and version to CORS allowed headers (#630) - (7d52a4b) - Max Kalashnikoff

- - -

## 0.51.3 - 2024-04-15
#### Bug Fixes
- **(identity)** adding project ID validation first in the handler (#623) - (71a314f) - Max Kalashnikoff

- - -

## 0.51.2 - 2024-04-15
#### Bug Fixes
- **(providers)** removing zkSync Goerli testnet, adding Sepolia instead (#627) - (7c34fcc) - Max Kalashnikoff

- - -

## 0.51.1 - 2024-04-15
#### Bug Fixes
- **(weights)** excluding zero weight providers from iteration (#628) - (f9a219d) - Max Kalashnikoff

- - -

## 0.51.0 - 2024-04-15
#### Features
- **(rpc)** adding rpc call retrying (#624) - (f0a1f7e) - Max Kalashnikoff

- - -

## 0.50.0 - 2024-04-15
#### Features
- **(providers)** adding Linea mainnet to supported chains (#626) - (0ef85ae) - Max Kalashnikoff

- - -

## 0.49.8 - 2024-04-15
#### Bug Fixes
- **(providers)** removing Goerli and Mumbai chains (#625) - (dfd07c1) - Max Kalashnikoff

- - -

## 0.49.7 - 2024-04-11
#### Bug Fixes
- **(identity)** hardcode ethereum mainnet (#622) - (378b28c) - Max Kalashnikoff

- - -

## 0.49.6 - 2024-04-11
#### Bug Fixes
- disable cache for testing project ID (#619) - (4001ff5) - Chris Smith

- - -

## 0.49.5 - 2024-04-11
#### Bug Fixes
- **(identity)** proper rpc errors handling in avatar lookup (#621) - (3df2e41) - Max Kalashnikoff

- - -

## 0.49.4 - 2024-04-10
#### Bug Fixes
- **(identity)** handling properly errors in RPC call response (#620) - (7c41c6f) - Max Kalashnikoff

- - -

## 0.49.3 - 2024-04-10
#### Bug Fixes
- **(revert)** hardcode mainnet (#610) (#618) - (1bde8e5) - Max Kalashnikoff

- - -

## 0.49.2 - 2024-04-09
#### Bug Fixes
- **(Docker)** adding curl to the Docker runtime image (#615) - (c35246b) - Max Kalashnikoff
- hardcode mainnet (#610) - (d8ed82d) - Chris Smith

- - -

## 0.49.1 - 2024-04-09
#### Bug Fixes
- **(server)** enabling tcp_nodelay and tcp_sleep_on_accept_errors (#617) - (c875075) - Max Kalashnikoff

- - -

## 0.49.0 - 2024-04-09
#### Bug Fixes
- **(server)** decreasing keep-alive ping interval (#616) - (c938b92) - Max Kalashnikoff
#### Features
- **(monitoring)** using 3 minutes period for 5xx alerts (#614) - (f12cdda) - Max Kalashnikoff

- - -

## 0.48.0 - 2024-04-08
#### Features
- **(logging)** improving logging in the transactions history (#613) - (3aaca98) - Max Kalashnikoff

- - -

## 0.47.0 - 2024-04-08
#### Features
- **(logging)** adding additional tracing on project id validation (#612) - (dd7ee6c) - Max Kalashnikoff

- - -

## 0.46.0 - 2024-04-05
#### Features
- **(providers)** fungible price endpoint implementation (#608) - (33aadff) - Max Kalashnikoff

- - -

## 0.45.2 - 2024-04-04
#### Bug Fixes
- **(build)** adding assets as an exemption to gitignore (#607) - (38d9a3a) - Max Kalashnikoff

- - -

## 0.45.1 - 2024-04-04
#### Bug Fixes
- **(names)** decreasing minimum name length to `3` (#606) - (707ed22) - Max Kalashnikoff

- - -

## 0.44.2 - 2024-04-03
#### Bug Fixes
- **(providers)** varying priority for the GetBlock provider (#602) - (bdff75e) - Max Kalashnikoff

- - -

## 0.44.1 - 2024-04-02
#### Bug Fixes
- **(providers)** removing deprecated endpoints from Grove/Pokt (#603) - (264586a) - Max Kalashnikoff

- - -

## 0.44.0 - 2024-04-02
#### Bug Fixes
- **(monitoring)** adding rate-limiting alert and removing double healthy hosts (#601) - (5f069e9) - Max Kalashnikoff
#### Features
- **(analytics)** adding transaction ID to the onramp analytics (#604) - (0da4f7e) - Max Kalashnikoff

- - -

## 0.43.0 - 2024-04-01
#### Features
- rate limiting (#600) - (e0a9fe2) - Max Kalashnikoff

- - -

## 0.42.0 - 2024-03-29
#### Features
- **(analytics)** updating to  tag (#599) - (68c3a23) - Max Kalashnikoff

- - -

## 0.41.8 - 2024-03-26
#### Bug Fixes
- **(zerion)** handling properly `Polygon` native token address in balance (#597) - (1aa8d7f) - Max Kalashnikoff

- - -

## 0.41.7 - 2024-03-26
#### Bug Fixes
- **(monitoring)** decreasing system metrics sampling interval (#598) - (2efa413) - Max Kalashnikoff
- **(monitoring)** increasing the CPU alarm interval (#596) - (c80d10e) - Max Kalashnikoff
- alarm config (#595) - (c619f7e) - Chris Smith
#### Miscellaneous Chores
- bump version - (28e4854) - Chris Smith

- - -

## 0.41.6 - 2024-03-25
#### Bug Fixes
- top-level ELB error metric and logs, downscale to min capacities (#592) - (7cf125d) - Chris Smith

- - -

## 0.41.5 - 2024-03-22
#### Bug Fixes
- provider-returned non-200s should be 503s & fix autoscaling (#591) - (1dbfe4c) - Chris Smith

- - -

## 0.41.4 - 2024-03-21
#### Bug Fixes
- **(balance)** using the implementation chain address instead of first (#589) - (43c8eb8) - Max Kalashnikoff

- - -

## 0.41.3 - 2024-03-21
#### Bug Fixes
- **(tests)** fixing address to be undefined and not null (#588) - (b484258) - Max Kalashnikoff
- removing unwraps for proper handling and error context (#583) - (1b69020) - Max Kalashnikoff

- - -

## 0.41.2 - 2024-03-21
#### Bug Fixes
- **(balance)** adding token contract address (#587) - (abe23d5) - Max Kalashnikoff
#### Miscellaneous Chores
- downgrade runner (#584) - (3d004b6) - Chris Smith

- - -

## 0.41.1 - 2024-03-14
#### Bug Fixes
- **(identity)** handling `0x` RPC response for identity (#582) - (7a59595) - Max Kalashnikoff

- - -

## 0.41.0 - 2024-03-14
#### Features
- **(dev)** updating default env files and .gitignore (#581) - (dddc908) - Max Kalashnikoff
- **(providers)** adding GetBlock provider (#577) - (55ced19) - Max Kalashnikoff

- - -

## 0.40.4 - 2024-03-14
#### Bug Fixes
- **(providers)** updating providers supported chains (#580) - (f9390be) - Max Kalashnikoff

- - -

## 0.40.3 - 2024-03-13
#### Bug Fixes
- removing `unwrap` in updating weights (#579) - (08c5b24) - Max Kalashnikoff

- - -

## 0.40.2 - 2024-03-13
#### Bug Fixes
- **(providers)** changing the Base testnet to Sepolia (#578) - (cd95039) - Max Kalashnikoff

- - -

## 0.40.1 - 2024-03-12
#### Bug Fixes
- **(providers)** removing Omnia provider (#575) - (9d42f41) - Max Kalashnikoff

- - -

## 0.40.0 - 2024-03-11
#### Bug Fixes
- supported chains endpoint (#571) - (8d0888d) - Chris Smith
#### Features
- **(docs)** adding the footnote for not guaranteed RPC chains (#574) - (f24d882) - Max Kalashnikoff

- - -

## 0.39.0 - 2024-03-08
#### Features
- **(conversion)** convert transaction builder endpoint implementation (#572) - (ddec3bb) - Max Kalashnikoff

- - -

## 0.38.0 - 2024-03-08
#### Features
- **(conversion)** approve transaction endpoint implementation (#570) - (b78f4fb) - Max Kalashnikoff

- - -

## 0.37.0 - 2024-03-08
#### Features
- **(conversion)** conversion quotes endpoint implementation (#568) - (1b798b2) - Max Kalashnikoff

- - -

## 0.36.0 - 2024-03-07
#### Features
- **(conversion)** available tokens list endpoint implementation (#567) - (e78f77a) - Max Kalashnikoff

- - -

## 0.35.1 - 2024-03-06
#### Bug Fixes
- **(zerion)** allowing HTTP 202 return code to pass (#564) - (8e7dc7b) - Max Kalashnikoff

- - -

## 0.35.0 - 2024-03-05
#### Features
- account balance endpoint (#563) - (ef11f08) - Max Kalashnikoff

- - -

## 0.34.0 - 2024-03-05
#### Bug Fixes
- **(monitoring)** fixing typo in excluding 503 from availability (#550) - (bfc8e85) - Max Kalashnikoff
- correct Solana chain ID (#565) - (5bfe565) - Chris Smith
#### Features
- sorting supported chains list by the chainid (#562) - (378304e) - Max Kalashnikoff

- - -

## 0.33.1 - 2024-03-01
#### Bug Fixes
- **(providers)** changing Near chain id to be CAIP-2 compatible (#547) - (b96bec0) - Max Kalashnikoff

- - -

## 0.33.0 - 2024-03-01
#### Bug Fixes
- **(grafana)** adding missed Near provider panels (#542) - (3ea1840) - Max Kalashnikoff
- **(monitoring)** removing 503 from non-providers errors (#546) - (6d760ec) - Max Kalashnikoff
- **(monitoring)** increasing CPU alarm threshold (#539) - (9481d93) - Max Kalashnikoff
- **(monitoring)** using 3 minutes CPU average (#536) - (e3cb34e) - Max Kalashnikoff
#### Features
- **(monitoring)** changing to use Prometheus data for the Memory usage (#529) - (6105312) - Max Kalashnikoff
- **(monitoring)** changing to use Prometheus data for the CPU usage (#528) - (cd66e9c) - Max Kalashnikoff
- **(providers)** adding Mantle to supported chains (#541) - (a1c15d3) - Max Kalashnikoff

- - -

## 0.32.0 - 2024-02-22
#### Features
- adding Holesky to the supported chains (#534) - (8c5fd5d) - Max Kalashnikoff

- - -

## 0.31.3 - 2024-02-22
#### Bug Fixes
- **(providers)** adding more Near protocol support (#535) - (4ed8520) - Max Kalashnikoff

- - -

## 0.31.2 - 2024-02-21
#### Bug Fixes
- **(errors)** unifying provider reachability errors to the 503 HTTP Error (#531) - (e66ff12) - Max Kalashnikoff

- - -

## 0.31.1 - 2024-02-20
#### Bug Fixes
- increasing keep-alive timeouts (#527) - (b2b4a36) - Max Kalashnikoff

- - -

## 0.31.0 - 2024-02-20
#### Features
- **(onramp)** adding onramp buy quotes endpoint (#525) - (8058278) - Max Kalashnikoff

- - -

## 0.30.0 - 2024-02-20
#### Features
- **(onramp)** adding onramp buy options endpoint (#499) - (2c98a73) - Max Kalashnikoff

- - -

## 0.29.0 - 2024-02-19
#### Features
- **(config)** config parameter to disable project id check (#526) - (04cd050) - Max Kalashnikoff

- - -

## 0.28.0 - 2024-02-16
#### Features
- **(history)** using the shared reqwest http client (#516) - (99b63bc) - Max Kalashnikoff

- - -

## 0.27.0 - 2024-02-15
#### Features
- **(metrics)** exporting CPU and Memory usage metrics to Prometheus (#514) - (2abe3a6) - Max Kalashnikoff

- - -

## 0.26.0 - 2024-02-15
#### Features
- **(rpc)** adding Klaytn mainnet to supported chains (#513) - (e54b603) - Max Kalashnikoff

- - -

## 0.25.0 - 2024-02-13
#### Features
- **(ens)** unifying errors (#512) - (e360409) - Max Kalashnikoff

- - -

## 0.24.0 - 2024-02-12
#### Features
- **(ens)** checks for allowed zones and name format (#511) - (446956d) - Max Kalashnikoff

- - -

## 0.23.0 - 2024-02-09
#### Features
- **(ens)** updating name address handler (#510) - (1ff2211) - Max Kalashnikoff

- - -

## 0.22.0 - 2024-02-09
#### Features
- **(ens)** updating name attributes handler (#509) - (725961b) - Max Kalashnikoff

- - -

## 0.21.2 - 2024-02-09
#### Bug Fixes
- **(ens)** register name endpoint cleanup (#505) - (fc4c07b) - Max Kalashnikoff

- - -

## 0.21.1 - 2024-02-07
#### Bug Fixes
- removing unwraps from the WeightedIndex (#508) - (e1228ff) - Max Kalashnikoff

- - -

## 0.21.0 - 2024-02-07
#### Features
- **(providers)** adding Base for Pokt and Publicnode providers (#506) - (d0acd3c) - Max Kalashnikoff

- - -

## 0.20.0 - 2024-02-07
#### Features
- **(analytics)** extracting onramp transactions history analytics (#498) - (7b58903) - Max Kalashnikoff

- - -

## 0.19.1 - 2024-02-07
#### Bug Fixes
- respond with temporary unavailable when no working providers found (#507) - (f0f905f) - Max Kalashnikoff

- - -

## 0.19.0 - 2024-02-07
#### Bug Fixes
- **(ci)** bump ci_workflows version (#504) - (a46cdfa) - Max Kalashnikoff
#### Features
- **(ens)** checking for supported attributes (#497) - (a46130f) - Max Kalashnikoff
- **(grafana)** adding Aurora and Quicknode providers dashboards (#501) - (f060240) - Max Kalashnikoff
- **(testsing)** splitting integration tests (#503) - (3b0c4dd) - Max Kalashnikoff

- - -

## 0.18.0 - 2024-02-05
#### Features
- **(providers)** adding the Quicknode provider for zkSync (#500) - (d1ca64c) - Max Kalashnikoff

- - -

## 0.17.1 - 2024-02-02
#### Bug Fixes
- **(ci)** fixing the new integration test error from 400 to 401 (#495) - (763323b) - Max Kalashnikoff
- link to supported chains from error message (#496) - (845d7aa) - Chris Smith

- - -

## 0.17.0 - 2024-02-01
#### Features
- **(providers)** proxy request for a certain provider (#487) - (fcbb608) - Max Kalashnikoff

- - -

## 0.16.0 - 2024-01-31
#### Features
- **(history)** exposing dapp and chain info (#494) - (a3bf46e) - Max Kalashnikoff

- - -

## 0.15.1 - 2024-01-26
#### Bug Fixes
- **(tests)** fixing database test for the recent ENSIP-11 update (#493) - (eb5efba) - Max Kalashnikoff

- - -

## 0.15.0 - 2024-01-25
#### Features
- **(ens)** changes to SLIP-44 and ENSIP-11 compatible responses (#489) - (1b78a00) - Max Kalashnikoff

- - -

## 0.14.0 - 2024-01-25
#### Features
- **(ens)** adding timestamp threshold check (#491) - (58bed27) - Max Kalashnikoff

- - -

## 0.13.3 - 2024-01-24
#### Bug Fixes
- **(providers)** removing Aurora websocket from the Infura (#492) - (3b3dc64) - Max Kalashnikoff

- - -

## 0.13.2 - 2024-01-24
#### Bug Fixes
- **(providers)** adding Aurora to the init providers (#488) - (ae26198) - Max Kalashnikoff

- - -

## 0.13.1 - 2024-01-23
#### Bug Fixes
- **(ci)** ignoring pokt solana test (#485) - (dcddb88) - Max Kalashnikoff
- **(ci)** using the `current`, `latest` or `manual` image tag in the manual deploy (#483) - (12f813a) - Max Kalashnikoff
- **(providers)** unifying Solana chain_id (#486) - (9100ab7) - Max Kalashnikoff

- - -

## 0.13.0 - 2024-01-22
#### Features
- **(providers)** updating Pokt to the new endpoint (#479) - (6fa8672) - Max Kalashnikoff

- - -

## 0.12.1 - 2024-01-18
#### Bug Fixes
- **(providers)** removing Aurora from the Infura provider (#482) - (0d9887e) - Max Kalashnikoff

- - -

## 0.12.0 - 2024-01-18
#### Features
- **(providers)** adding the Aurora native mainnet and testnet RPC (#469) - (48b18c7) - Max Kalashnikoff

- - -

## 0.11.4 - 2024-01-18
#### Bug Fixes
- **(ci)** fixes to actions workflow files according to the `actionlint` (#480) - (9ddb398) - Max Kalashnikoff
- **(noop)** checking providers CI workflow (#481) - (949592e) - Max Kalashnikoff

- - -

## 0.11.3 - 2024-01-16
#### Bug Fixes
- **(ci)** inherit secrets to providers validation (#477) - (15ff3db) - Max Kalashnikoff
- **(noop)** checking the providers CI workflow (#478) - (b32f1c2) - Max Kalashnikoff

- - -

## 0.11.2 - 2024-01-16
#### Bug Fixes
- **(ci)** removing write permissions from the workflow (#475) - (d78711f) - Max Kalashnikoff
- **(ci)** noop change in providers code to test the CI (#474) - (ae88560) - Max Kalashnikoff
- **(ci)** changing to call providers workflow as a job (#473) - (eaaf79e) - Max Kalashnikoff
- **(noop)** checking the providers CI workflow (#476) - (71916af) - Max Kalashnikoff

- - -

## 0.11.1 - 2024-01-15
#### Bug Fixes
- **(ci)** adding the branch tag for the sub workflow (#470) - (846045c) - Max Kalashnikoff
- **(noop)** this is noop commit to check the provider tests new CI (#471) - (da2d7da) - Max Kalashnikoff

- - -

## 0.11.0 - 2024-01-15
#### Bug Fixes
- **(ci)** removnge the trailing slash from rpc url (#467) - (c2184ba) - Max Kalashnikoff
- **(noop)** this is noop commit to check the provider tests new CI (#468) - (e8813d9) - Max Kalashnikoff
#### Features
- **(ci)** run provider integration tests from sub-validate (#466) - (5fc1df8) - Max Kalashnikoff
- **(ci)** per provider integration tests (#465) - (36e3652) - Max Kalashnikoff

- - -

## 0.10.3 - 2024-01-12
#### Bug Fixes
- adding temporary instrument logging for the avatar lookup (#462) - (f039e28) - Max Kalashnikoff
- correct name of dynamic terraform variables (#458) - (07e9250) - Xavier Basty

- - -

## 0.10.2 - 2024-01-10
#### Bug Fixes
- changing to `validate_project_access_and_quota` in history and portfolio handlers (#456) - (456cb6a) - Max Kalashnikoff

- - -

## 0.10.1 - 2024-01-10
#### Bug Fixes
- not returning quota limit error (#457) - (22517f3) - xDarksome
- ignoring Infura tests until resolution (#455) - (5a6ad9a) - Max Kalashnikoff
- passing infura project id to the tests (#454) - (05a74d5) - Max Kalashnikoff

- - -

## 0.10.0 - 2024-01-09
#### Features
- implementing on ramp url generator (#453) - (2ce2e8e) - Max Kalashnikoff

- - -

## 0.9.2 - 2024-01-05
#### Bug Fixes
- **(logging)** removing Coinbase response logging (#449) - (079f510) - Max Kalashnikoff
#### Miscellaneous Chores
- **(logging)** reducing logging for proxy requests (#447) - (470c0cb) - Max Kalashnikoff
- **(monitoring)** adding history metrics per provider (#446) - (0e6df79) - Max Kalashnikoff
- reducing logging for the identity requests (#448) - (44b45ea) - Max Kalashnikoff
- bumping the ethers version (#435) - (f6b6cd3) - Max Kalashnikoff
- Bump version for release - (4e9e57a) - geekbrother

- - -

## 0.9.0 - 2024-01-02
#### Features
- **(onramp)** exposing name and quantity from Coinbase (#442) - (a7a55bc) - Max Kalashnikoff
#### Miscellaneous Chores
- **(refactor)** making history structures unite, arranging structures (#441) - (ea71dd3) - Max Kalashnikoff

- - -

## 0.8.0 - 2023-12-25
#### Bug Fixes
- **(providers)** changing the mainnet URL for zksync (#443) - (a77749b) - Max Kalashnikoff
#### Features
- **(ci)** implement integration tests for hexless accounts (#419) - (7be75d3) - Max Kalashnikoff

- - -

## 0.7.1 - 2023-12-22
#### Bug Fixes
- revert back to string-based address in transactions (#439) - (68cd876) - Max Kalashnikoff

- - -

## 0.7.0 - 2023-12-22
#### Bug Fixes
- **(ci)** add postgres to the sub-validate (#438) - (cc8641e) - Max Kalashnikoff
- **(ci)** removing /health from the sub-cd validation (#437) - (e9a2f9e) - Max Kalashnikoff
#### Features
- **(onramp)** Coinbase transaction status (#388) - (ea4c371) - Derek

- - -

## 0.6.1 - 2023-12-21
#### Bug Fixes
- use task name instead of image name in ECS deploy on `staging` (#436) - (83f8020) - Xavier Basty
#### Miscellaneous Chores
- removing ens_allowlist (#431) - (5c6d644) - Max Kalashnikoff

- - -

## 0.6.0 - 2023-12-20
#### Features
- implement register handler (#418) - (3c06126) - Max Kalashnikoff

- - -

## 0.5.0 - 2023-12-20
#### Features
- adding lookup and reverse lookup handlers (#417) - (7fc110f) - Max Kalashnikoff

- - -

## 0.4.2 - 2023-12-20
#### Bug Fixes
- **(ci)** fixing the stage-url variable for the validation (#432) - (3b34ddd) - Max Kalashnikoff
#### Miscellaneous Chores
- **(docker)** Revert: moving COPY before the cargo chef cook (#434) - (fd11cfa) - Max Kalashnikoff

- - -

## 0.4.1 - 2023-12-20
#### Bug Fixes
- update shared flows to `0.1.3` to fix ECS task names (#433) - (b66330e) - Xavier Basty
#### Miscellaneous Chores
- **(ci)** fixing prod validation url (#425) - (8f651f9) - Max Kalashnikoff
- adding node_modules to the .gitignore (#424) - (d59b6b9) - Max Kalashnikoff

- - -

## 0.4.0 - 2023-12-19
#### Features
- implement helpers (#413) - (389e70e) - Max Kalashnikoff
#### Miscellaneous Chores
- **(docker)** moving COPY before the cargo chef cook (#429) - (d540595) - Max Kalashnikoff

- - -

## 0.3.1 - 2023-12-19
#### Bug Fixes
- use the `X-Forwarded-For` header from the ALB to retrieve the client IP (#428) - (306ad07) - Xavier Basty
#### Miscellaneous Chores
- **(docker)** remove migrations directory from dockerignore (#427) - (c438c14) - Max Kalashnikoff

- - -

## 0.3.0 - 2023-12-19
#### Bug Fixes
- **(o11y)** dashboards broken (#420) - (777166c) - Derek
- ECS task name in version retrieval (#423) - (97440c5) - Xavier Basty
- query URL (#421) - (8e7e99f) - Chris Smith
- change ECS role Prometheus permission to read+write access (#422) - (aaac458) - Xavier Basty
#### Features
- scaffold sqlx (#412) - (2ef48ec) - Max Kalashnikoff
- add sql schema and migrations (#411) - (4e2e19a) - Max Kalashnikoff
- add Postgres 16 to the docker-compose for the CI tests (#410) - (e60d6f1) - Max Kalashnikoff
#### Miscellaneous Chores
- **(terraform)** downgrade aurora version (#426) - (6c8e67b) - Max Kalashnikoff
- **(terraform)** adding Postgres (#415) - (4c899d6) - Max Kalashnikoff

- - -

## 0.2.0 - 2023-12-15
#### Features
- **(identity)** remove ENS demo (#404) - (590da3f) - Derek
#### Miscellaneous Chores
- migrate CI, AWS account and alerting (#382) - (50f30e9) - Xavier Basty

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).