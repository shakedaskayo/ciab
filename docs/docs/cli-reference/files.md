# ciab files

Manage files in sandboxes.

## list

List files in a directory.

```bash
ciab files list <sandbox-id> [--path <dir>]
```

Default path is `/`.

## upload

Upload a file to the sandbox.

```bash
ciab files upload <sandbox-id> --path <remote-path> --input <local-file>
```

## download

Download a file from the sandbox.

```bash
ciab files download <sandbox-id> --path <remote-path> [--output <local-file>]
```

If `--output` is omitted, content is written to stdout.

## delete

Delete a file in the sandbox.

```bash
ciab files delete <sandbox-id> --path <remote-path>
```
