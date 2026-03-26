# ciab image

Manage machine images for the EC2 runtime backend.

## build

Start a new image build using Packer.

```bash
ciab image build [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--provider` | `claude-code` | Agent provider to install in the image |
| `--region` | Config default | AWS region for the AMI |
| `--instance-type` | Config default | EC2 instance type for the build |
| `--template` | Config default | Packer template source override |
| `--var` | -- | Extra Packer variable, `key=value` (repeatable) |
| `--wait` | `false` | Block until the build completes |

```bash
# Build a Claude Code AMI and wait for completion
ciab image build --provider claude-code --wait

# Build a Codex AMI in eu-west-1
ciab image build --provider codex --region eu-west-1

# Pass extra variables to the Packer template
ciab image build --provider claude-code --var node_version=22 --var extra_packages=ripgrep
```

## list

List available machine images.

```bash
ciab image list [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--region` | Config default | Filter by AWS region |
| `--provider` | -- | Filter by agent provider |

```bash
ciab image list
ciab image list --provider claude-code --region us-east-1
```

Example output:

```
IMAGE ID                  PROVIDER      REGION      CREATED              STATUS
ami-0abcdef1234567890     claude-code   us-east-1   2026-03-25 14:30:00  available
ami-0fedcba9876543210     codex         us-east-1   2026-03-24 10:15:00  available
```

## status

Check the status of an image build.

```bash
ciab image status <build-id>
```

```bash
ciab image status 550e8400-e29b-41d4-a716-446655440000
```

Example output:

```
Build ID:   550e8400-e29b-41d4-a716-446655440000
Provider:   claude-code
Region:     us-east-1
Status:     building
Started:    2026-03-25 14:30:00
AMI ID:     --
```

Build states: `queued`, `building`, `succeeded`, `failed`.

## delete

Delete a machine image (deregisters the AMI and deletes the EBS snapshot).

```bash
ciab image delete <image-id>
```

```bash
ciab image delete ami-0abcdef1234567890
```

!!! warning
    This is irreversible. Running instances launched from this AMI are not affected, but new sandboxes cannot use the deleted image.
