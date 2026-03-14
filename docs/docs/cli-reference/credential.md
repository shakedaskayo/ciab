# ciab credential

Manage encrypted credentials.

## create

Store a new credential.

```bash
ciab credential create --name <name> --type <type> --data <json>
```

**Types:** `api_key`, `env_vars`, `git_token`, `oauth_token`, `ssh_key`, `file`

```bash
ciab credential create \
  --name anthropic-key \
  --type api_key \
  --data '{"ANTHROPIC_API_KEY": "sk-ant-..."}'
```

## list

List all credentials (metadata only).

```bash
ciab credential list
```

## get

Get credential metadata.

```bash
ciab credential get <credential-id>
```

## delete

Delete a credential.

```bash
ciab credential delete <credential-id>
```
