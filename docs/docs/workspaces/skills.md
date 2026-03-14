# Skills

Skills are reusable agent capabilities that provide procedural knowledge and best practices. CIAB supports skills from [skills.sh](https://skills.sh/) and custom skill sources.

## Configuration

```toml
[[workspace.skills]]
source = "vercel-labs/ai-sdk-best-practices"
version = "v2.0"
enabled = true
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | string | Yes | — | Skill identifier (owner/repo format or URL) |
| `version` | string | No | latest | Specific version or tag |
| `name` | string | No | — | Override display name |
| `enabled` | boolean | No | `true` | Toggle without removing |
| `config` | object | No | `{}` | Skill-specific configuration |

## How Skills Work

During provisioning, enabled skills are installed via:

```bash
npx skillsadd <source>
```

This installs the skill's configuration files, prompts, and rules into the agent's environment, giving it access to specialized knowledge and procedures.

## Examples

```toml
# Popular community skills
[[workspace.skills]]
source = "vercel-labs/ai-sdk-best-practices"

[[workspace.skills]]
source = "community/typescript-patterns"

# Custom internal skills
[[workspace.skills]]
source = "shakedaskayo/internal-coding-standards"
version = "v3.1"

# Temporarily disabled skill
[[workspace.skills]]
source = "experimental/new-framework"
enabled = false
```

## Custom Skills

You can create your own skills by following the [skills.sh specification](https://skills.sh/). Skills are Git repositories with a standard structure that agents can consume.
