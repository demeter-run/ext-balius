# Instance

### Local development.

To develop locally, you need to do the following:

1. Create a local vault server:
   ```shell
   vault server -dev 

   ```
   On another shell:
   ```shell
   vault policy write -address http://127.0.0.1:8200 balius ../bootstrap/feature/policy.hcl
   vault secrets enable -address http://127.0.0.1:8200 transit
   vault token create -address http://127.0.0.1:8200 -policy="balius" -display-name="balius" -ttl="720h" -renewable=true -format json | jq -r '.auth.client_token'

   ```
   Save the output, it is the token for interacting with vault.
2. Create a local postgres:
   ```shell
   docker run -d --rm --name balius -e POSTGRES_USER=test -e POSTGRES_PASSWORD=test -e POSTGRES_DB=test -p 5432:5432 postgres
   PGPASSWORD=test psql -U test -h localhost test -f migrations/20250527.sql

   ```
3. Have a local running dolos instance.
4. Create a `config.toml` with the following:
   ```toml
   network = "cardano-preprod"
   connection = "host=localhost user=test password=test"
   namespace = "ext-balius-m1"
   pod = ""  # Replace with some random data.
   shard = "test-chainsync"
   prometheus_addr = "0.0.0.0:8002"
   vault_address = "http://127.0.0.1:8200"
   vault_token = ""  # Replace with you vault token
   vault_token_renew_seconds = 10

   [rpc]
   listen_address = "0.0.0.0:3001"

   [ledger]
   endpoint_url = "http://localhost:50051"

   [chainsync]
   endpoint_url = "http://localhost:50051"
   ```
5. Run `BALIUSD_CONFIG=config.toml cargo run`
6. To cleanup, `docker container stop balius` and stop the vault process.
