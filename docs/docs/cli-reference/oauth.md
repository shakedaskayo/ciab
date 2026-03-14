# ciab oauth

OAuth authentication flows.

## authorize

Start browser-based OAuth authorization.

```bash
ciab oauth authorize --provider <provider>
```

Opens the default browser to the authorization URL.

## device-code

Start a device code flow (for headless environments).

```bash
ciab oauth device-code --provider <provider>
```

Displays the user code and verification URL.

## device-poll

Poll for device code authorization completion.

```bash
ciab oauth device-poll --provider <provider> --device-code <code>
```

## refresh

Refresh an OAuth token.

```bash
ciab oauth refresh --provider <provider> --credential-id <id>
```
