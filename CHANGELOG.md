# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

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