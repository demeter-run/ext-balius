# ed25519-signer-policy.hcl
path "transit/keys/*" {
  capabilities = ["create", "update", "list"]
}

path "transit/export/public-key/*" {
  capabilities = ["read"]
}

path "transit/sign/*" {
  capabilities = ["update"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
