# Kubernetes

Run CIAB and its agent sandboxes on Kubernetes. Each sandbox becomes a Pod, with optional microVM isolation via [Kata Containers](https://katacontainers.io/).

## Overview

The Kubernetes backend (`ciab-sandbox-k8s`) creates one Pod per sandbox. Each Pod gets:

- A dedicated PVC for workspace files
- Network isolation via NetworkPolicy
- Optional RuntimeClass for Kata Containers / microVM isolation
- Configurable resource limits and node scheduling

## Prerequisites

- Kubernetes cluster (v1.26+)
- `kubectl` configured with cluster access
- Helm 3 (for the Helm chart)
- Optional: Kata Containers installed and a RuntimeClass registered

## Quick Start with Helm

The Helm chart deploys the CIAB server itself on Kubernetes, configured to use the Kubernetes runtime backend.

```bash
# Add the namespace for agent Pods
kubectl create namespace ciab-agents

# Install CIAB
helm install ciab ./helm/ciab \
  --set secrets.anthropicApiKey=$ANTHROPIC_API_KEY \
  --set secrets.encryptionKey=$(openssl rand -hex 32)
```

## Configuration

### config.toml

Set the runtime backend to `kubernetes` and configure the `[runtime.kubernetes]` section:

```toml
[runtime]
backend = "kubernetes"

[runtime.kubernetes]
namespace = "ciab-agents"
agent_image = "ghcr.io/shakedaskayo/ciab-claude:latest"

# Storage
storage_class = "standard"
workspace_pvc_size = "10Gi"

# Security (all default to true)
create_network_policy = true
run_as_non_root = true
drop_all_capabilities = true

# Resource defaults
default_cpu_request = "500m"
default_cpu_limit = "2"
default_memory_request = "256Mi"
default_memory_limit = "2Gi"
```

### Kata Containers (microVM isolation)

For hardware-level isolation, set a RuntimeClass that maps to Kata Containers:

```toml
[runtime.kubernetes]
runtime_class = "kata-containers"
```

The cluster operator must install Kata Containers and register the RuntimeClass:

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: kata-containers
handler: kata-qemu
```

### Node Scheduling

Pin agent Pods to dedicated nodes:

```toml
[runtime.kubernetes.node_selector]
"ciab/agent-node" = "true"
```

### Kubeconfig

When running CIAB outside the cluster, specify a kubeconfig:

```toml
[runtime.kubernetes]
kubeconfig = "/home/user/.kube/config"
context = "my-cluster"
```

When running inside the cluster (e.g., via the Helm chart), omit these fields to use in-cluster config automatically.

## Per-Workspace Overrides

Workspaces can override Kubernetes settings:

```toml
[runtime]
backend = "kubernetes"
kubernetes_namespace = "team-frontend"
kubernetes_runtime_class = "kata-containers"
kubernetes_image = "ghcr.io/shakedaskayo/ciab-gemini:latest"

[runtime.kubernetes_node_selector]
"team" = "frontend"
```

## Helm Chart Reference

The chart is located at `helm/ciab/`. Key values:

| Value | Default | Description |
|-------|---------|-------------|
| `replicaCount` | `1` | CIAB server replicas |
| `image.repository` | `ghcr.io/shakedaskayo/ciab` | Server image |
| `service.port` | `9090` | API port |
| `persistence.enabled` | `true` | SQLite PVC |
| `persistence.size` | `5Gi` | SQLite PVC size |
| `runtime.backend` | `kubernetes` | Runtime backend |
| `runtime.kubernetes.namespace` | `ciab-agents` | Agent Pod namespace |
| `runtime.kubernetes.runtimeClass` | `""` | RuntimeClass (Kata) |
| `runtime.kubernetes.storageClass` | `standard` | Workspace PVC storage class |
| `runtime.kubernetes.workspacePvcSize` | `10Gi` | Workspace PVC size |
| `runtime.kubernetes.createNetworkPolicy` | `true` | Isolate agent Pods |
| `runtime.kubernetes.runAsNonRoot` | `true` | Non-root containers |
| `runtime.kubernetes.dropAllCapabilities` | `true` | Drop all caps |
| `secrets.encryptionKey` | `""` | Credential vault key |
| `secrets.anthropicApiKey` | `""` | Anthropic API key |
| `ingress.enabled` | `false` | Enable Ingress |

See `helm/ciab/values.yaml` for the full reference.

## Architecture

```
┌──────────────────────┐
│   CIAB Server Pod    │
│   (ciab-api + CLI)   │
│   PVC: sqlite.db     │
└─────────┬────────────┘
          │ kube API
          ▼
┌──────────────────────┐  ┌──────────────────────┐
│   Agent Pod (claude) │  │   Agent Pod (codex)   │
│   PVC: workspace     │  │   PVC: workspace      │
│   NetworkPolicy ✓    │  │   NetworkPolicy ✓     │
└──────────────────────┘  └───────────────────────┘
```

Each agent Pod is created, monitored, and cleaned up by the CIAB server through the Kubernetes API.
