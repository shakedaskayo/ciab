# Packer Image Builder

Build machine images (AMIs) with coding agents pre-installed using HashiCorp Packer. Pre-built images dramatically reduce sandbox startup time on EC2.

## Overview

The Packer integration (`ciab-packer`) automates building machine images:

- Build AMIs with agents, dependencies, and tools pre-installed
- Use builtin templates or provide your own
- Fetch templates from local files, HTTP URLs, or git repos
- Track build status through the API or CLI
- Automatically clean up intermediate resources

## Prerequisites

- [HashiCorp Packer](https://developer.hashicorp.com/packer/install) installed (or set `auto_install = true`)
- AWS credentials configured with permissions to create AMIs
- A VPC/subnet where Packer can launch a temporary build instance

## Configuration

### config.toml

```toml
[packer]
# Path to the packer binary (default: "packer", found via PATH)
binary_path = "packer"

# Automatically install Packer if not found
auto_install = true

# Working directory for Packer builds
work_dir = "/tmp/ciab-packer"

# Default AWS region for AMI builds
default_region = "us-east-1"

# Default instance type for the build instance
build_instance_type = "t3.medium"

# Default VPC/subnet for the build instance (optional, uses default VPC if omitted)
build_subnet_id = ""

# Timeout for the entire build process
build_timeout_secs = 1800

# Template source (see Template Sources below)
template_source = "builtin://default-ec2"
```

## Template Sources

Packer templates can be loaded from multiple sources:

### Builtin Templates

The default template installs common agent dependencies on Ubuntu 22.04:

```toml
[packer]
template_source = "builtin://default-ec2"
```

The builtin `default-ec2` template:

- Starts from the latest Ubuntu 22.04 LTS AMI
- Installs Node.js 20, Python 3.11, Rust toolchain, Go 1.22, git, curl, jq
- Installs agent binaries based on the `provider` variable
- Configures a non-root `ciab` user with SSH access
- Accepts variables: `provider`, `region`, `instance_type`, `subnet_id`

### Local Files

```toml
[packer]
template_source = "/path/to/my-template.pkr.hcl"
```

### HTTP URLs

```toml
[packer]
template_source = "https://example.com/templates/ciab-agent.pkr.hcl"
```

### Git Repositories

Use Packer-style git source syntax:

```toml
[packer]
template_source = "git::https://github.com/myorg/infra.git//packer/ciab?ref=v1.0.0"
```

Format: `git::<repo-url>//<path-within-repo>?ref=<tag-or-branch>`

## CLI Usage

### Build an Image

```bash
ciab image build [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--provider` | `claude-code` | Agent provider to install |
| `--region` | Config default | AWS region for the AMI |
| `--instance-type` | Config default | Build instance type |
| `--template` | Config default | Template source override |
| `--var` | -- | Extra Packer variable (repeatable, `key=value`) |
| `--wait` | `false` | Wait for the build to complete |

```bash
# Build with defaults
ciab image build --provider claude-code --wait

# Build with custom region and instance type
ciab image build --provider codex --region eu-west-1 --instance-type t3.large

# Build with extra variables
ciab image build --provider claude-code --var node_version=22 --var extra_packages=ripgrep
```

### List Images

```bash
ciab image list [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--region` | Config default | Filter by region |
| `--provider` | -- | Filter by agent provider |

```bash
ciab image list
ciab image list --provider claude-code --region us-east-1
```

### Check Build Status

```bash
ciab image status <build-id>
```

Shows the current state of an image build: `queued`, `building`, `succeeded`, or `failed`.

### Delete an Image

```bash
ciab image delete <image-id>
```

Deregisters the AMI and deletes the associated EBS snapshot.

!!! warning
    Deleting an image does not affect running instances that were launched from it, but new sandboxes cannot use it.

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/images/build` | Start an image build |
| `GET` | `/api/v1/images` | List available images |
| `GET` | `/api/v1/images/builds/{build_id}` | Get build status |
| `DELETE` | `/api/v1/images/{image_id}` | Delete an image |

See the [Images API Reference](../api-reference/images.md) for full request/response schemas.

## Integration with EC2

### Workflow 1: Pre-build AMIs (Recommended)

Build images ahead of time for fast sandbox startup:

```bash
# Build once
ciab image build --provider claude-code --wait

# Reference in config.toml
# [runtime.ec2]
# default_ami = "ami-0abcdef1234567890"

# All new sandboxes use the pre-built AMI
ciab sandbox create --provider claude-code
```

Sandbox startup is typically under 60 seconds with a pre-built AMI.

### Workflow 2: On-demand provisioning

Without a pre-built AMI, CIAB uses a base Ubuntu AMI and provisions the agent at sandbox creation time. This adds 3-5 minutes to startup but requires no upfront image builds.

!!! tip
    Pre-building images is strongly recommended for production use. On-demand provisioning is useful for development and testing.
