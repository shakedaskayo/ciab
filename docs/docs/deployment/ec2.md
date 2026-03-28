# AWS EC2

Run CIAB agent sandboxes on AWS EC2. Each sandbox gets a dedicated EC2 instance with SSH-based command execution and automatic cleanup.

## Overview

The EC2 backend (`ciab-sandbox-ec2`) creates one EC2 instance per sandbox. Each instance gets:

- A dedicated ephemeral Ed25519 SSH key pair injected via cloud-init user-data
- Instance tags for identification and lifecycle management
- Automatic termination on sandbox deletion
- Optional integration with Packer for pre-built AMIs

## Prerequisites

- AWS account with EC2 permissions
- AWS credentials configured (`~/.aws/credentials`, environment variables, or IAM role)
- A VPC with a subnet that has internet access
- A security group allowing inbound SSH (port 22) from the CIAB server
- An AMI with the target agent pre-installed (or use [Packer](packer.md) to build one)

## Configuration

### config.toml

Set the runtime backend to `ec2` and configure the `[runtime.ec2]` section:

```toml
[runtime]
backend = "ec2"

[runtime.ec2]
# AWS region for instances
region = "us-east-1"

# Default AMI for agent instances (build with `ciab image build`)
default_ami = "ami-0abcdef1234567890"

# Instance type
instance_type = "t3.medium"

# Networking
subnet_id = "subnet-0123456789abcdef0"
security_group_ids = ["sg-0123456789abcdef0"]

# SSH settings
ssh_user = "ubuntu"                    # SSH user on the AMI
ssh_port = 22                          # SSH port
ssh_timeout_secs = 120                 # Timeout waiting for SSH readiness

# Instance configuration
key_pair_name = ""                     # Leave empty to use ephemeral keys (recommended)
iam_instance_profile = ""              # Optional IAM instance profile ARN
root_volume_size_gb = 30               # Root EBS volume size
root_volume_type = "gp3"              # EBS volume type: gp3, gp2, io1, io2

# Lifecycle
terminate_on_delete = true             # Terminate instance when sandbox is deleted
stop_on_pause = true                   # Stop instance on sandbox pause (vs. keep running)
instance_ready_timeout_secs = 180      # Max time to wait for instance to reach "running"

# Tags applied to all instances
[runtime.ec2.tags]
"Environment" = "development"
"Team" = "platform"
```

## SSH Key Management

By default, CIAB generates an ephemeral Ed25519 key pair for each sandbox:

1. A fresh key pair is generated at sandbox creation time
2. The public key is injected into the instance via cloud-init user-data
3. The private key is stored in the CIAB database (encrypted)
4. On sandbox deletion, the key pair is discarded along with the instance

This approach avoids managing long-lived SSH keys. No AWS key pair resource is created.

!!! tip
    If you need to use a pre-existing AWS key pair instead (e.g., for debugging), set `key_pair_name` in the config. CIAB will use that key pair and expect the private key at `~/.ssh/<key_pair_name>.pem`.

## Instance Tagging

Every EC2 instance created by CIAB is tagged for identification:

| Tag | Value | Description |
|-----|-------|-------------|
| `ciab-sandbox-id` | UUID | The sandbox ID |
| `ciab-managed` | `true` | Marks the instance as CIAB-managed |
| `Name` | `ciab-<sandbox-name>` | Human-readable name |

Custom tags from `[runtime.ec2.tags]` are merged with these defaults.

!!! warning
    Do not remove the `ciab-managed` tag from running instances. CIAB uses this tag to identify and clean up instances on shutdown.

## Per-Workspace Overrides

Workspaces can override EC2 settings:

```toml
[runtime]
backend = "ec2"
ec2_region = "eu-west-1"
ec2_instance_type = "t3.large"
ec2_ami = "ami-0fedcba9876543210"
ec2_subnet_id = "subnet-0fedcba9876543210"

[runtime.ec2_tags]
"Team" = "frontend"
```

## Integration with Packer

The recommended workflow is to pre-build AMIs with Packer, then reference them in the EC2 config:

```bash
# 1. Build an AMI with claude-code pre-installed
ciab image build --provider claude-code --region us-east-1

# 2. Get the AMI ID from the build output
ciab image list

# 3. Set the AMI in config.toml
# default_ami = "ami-0abcdef1234567890"
```

See [Packer](packer.md) for full image building documentation.

!!! note
    If no `default_ami` is set, CIAB falls back to a base Ubuntu 22.04 AMI and provisions the agent at sandbox creation time. This is slower but works without Packer.

## Architecture

```
┌──────────────────────┐
│   CIAB Server         │
│   (ciab-api + CLI)    │
│   SQLite + SSH keys   │
└─────────┬────────────┘
          │ AWS API + SSH
          ▼
┌──────────────────────┐  ┌──────────────────────┐
│   EC2 Instance       │  │   EC2 Instance        │
│   (claude-code)      │  │   (codex)             │
│   Tag: ciab-managed  │  │   Tag: ciab-managed   │
└──────────────────────┘  └───────────────────────┘
```

CIAB creates, monitors, and terminates EC2 instances through the AWS API. Commands are executed over SSH.
