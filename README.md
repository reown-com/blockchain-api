# rpc-proxy

`cargo run` to run and then `curl -v localhost:3000/health`

### Docker

`docker build . --tag rpc-proxy:`

`docker run -p 3000:3000 -e INFURA_PROJECT_ID=<some id> --name rpc -it rpc-proxy`
