# CIAB Default EC2 AMI Template
# Builds an Ubuntu-based AMI with a coding agent pre-installed.

packer {
  required_plugins {
    amazon = {
      source  = "github.com/hashicorp/amazon"
      version = ">= 1.3.0"
    }
  }
}

variable "region" {
  type    = string
  default = "us-east-1"
}

variable "base_ami" {
  type        = string
  description = "Base AMI ID (Ubuntu 22.04 recommended)"
}

variable "instance_type" {
  type    = string
  default = "t3.medium"
}

variable "agent_provider" {
  type        = string
  default     = "claude-code"
  description = "Agent CLI to install: claude-code, codex, gemini, cursor"
}

variable "ssh_user" {
  type    = string
  default = "ubuntu"
}

variable "ami_name_prefix" {
  type    = string
  default = "ciab-agent"
}

variable "volume_size" {
  type    = number
  default = 20
}

source "amazon-ebs" "agent" {
  region        = var.region
  source_ami    = var.base_ami
  instance_type = var.instance_type
  ssh_username  = var.ssh_user
  ami_name      = "${var.ami_name_prefix}-${var.agent_provider}-{{timestamp}}"

  launch_block_device_mappings {
    device_name           = "/dev/sda1"
    volume_size           = var.volume_size
    volume_type           = "gp3"
    delete_on_termination = true
  }

  tags = {
    Name       = "${var.ami_name_prefix}-${var.agent_provider}"
    ManagedBy  = "ciab-packer"
    Agent      = var.agent_provider
    BaseAMI    = var.base_ami
    BuiltAt    = "{{timestamp}}"
  }
}

build {
  sources = ["source.amazon-ebs.agent"]

  provisioner "shell" {
    inline = [
      "sudo apt-get update -y",
      "sudo apt-get install -y git curl wget build-essential unzip jq",
      "sudo apt-get clean",
    ]
  }

  provisioner "shell" {
    inline = [
      "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -",
      "sudo apt-get install -y nodejs",
    ]
  }

  provisioner "shell" {
    inline = [
      "case '${var.agent_provider}' in",
      "  claude-code)",
      "    sudo npm install -g @anthropic-ai/claude-code",
      "    ;;",
      "  codex)",
      "    sudo npm install -g @openai/codex",
      "    ;;",
      "  gemini)",
      "    sudo npm install -g @google/gemini-cli",
      "    ;;",
      "  cursor)",
      "    echo 'Cursor CLI requires manual installation'",
      "    ;;",
      "  *)",
      "    echo 'Unknown agent provider: ${var.agent_provider}'",
      "    exit 1",
      "    ;;",
      "esac",
    ]
  }

  provisioner "shell" {
    inline = [
      "sudo mkdir -p /home/${var.ssh_user}/workspace",
      "sudo chown -R ${var.ssh_user}:${var.ssh_user} /home/${var.ssh_user}/workspace",
    ]
  }

  provisioner "shell" {
    inline = [
      "sudo sed -i 's/^PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config",
      "sudo sed -i 's/^#PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config",
      "sudo sed -i 's/^PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config",
    ]
  }

  provisioner "shell" {
    inline = [
      "echo '{\"agent_provider\":\"${var.agent_provider}\",\"built_at\":\"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'\"}' | sudo tee /etc/ciab-image.json > /dev/null",
    ]
  }
}
