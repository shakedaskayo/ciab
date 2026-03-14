import type { WorkspaceSpec } from "@/lib/api/types";

export interface StarterTemplate {
  id: string;
  name: string;
  description: string;
  icon: string; // lucide icon name
  category: string;
  color: string; // accent color class
  provider: string;
  spec: WorkspaceSpec;
}

export const STARTER_TEMPLATES: StarterTemplate[] = [
  // ── Claude Code starters ──────────────────────────────────
  {
    id: "react-fullstack",
    name: "React Fullstack",
    description:
      "Modern React 19 + TypeScript + Vite app with Tailwind CSS, ESLint, and testing setup. Ready to build.",
    icon: "Globe",
    category: "Web",
    color: "from-blue-500/20 to-cyan-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a senior fullstack React developer. Use TypeScript, functional components, and modern React patterns. Prefer Tailwind CSS for styling. Write clean, maintainable code with proper error handling.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command: "npm create vite@latest app -- --template react-ts",
          workdir: "/workspace",
        },
        {
          name: "install",
          command: "npm install",
          workdir: "/workspace/app",
        },
        {
          name: "tailwind",
          command:
            "npm install -D tailwindcss @tailwindcss/vite",
          workdir: "/workspace/app",
        },
      ],
      skills: [
        { source: "nicepkg/aide-docs", enabled: true },
        { source: "anthropics/courses", name: "prompt-engineering", enabled: true },
      ],
      binaries: [
        { name: "nodejs", method: "apt" },
      ],
      env_vars: {
        NODE_ENV: "development",
      },
    },
  },
  {
    id: "nextjs-saas",
    name: "Next.js SaaS Starter",
    description:
      "Production-ready Next.js 15 with App Router, server components, Prisma ORM, and auth scaffolding.",
    icon: "Layers",
    category: "Web",
    color: "from-slate-500/20 to-zinc-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a Next.js expert building a SaaS application. Use the App Router, React Server Components where appropriate, and follow Next.js best practices. Use Prisma for database access.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "npx create-next-app@latest app --typescript --tailwind --eslint --app --src-dir --import-alias '@/*'",
          workdir: "/workspace",
        },
        {
          name: "prisma",
          command: "npm install prisma @prisma/client && npx prisma init",
          workdir: "/workspace/app",
        },
      ],
      skills: [
        { source: "vercel/next.js", name: "nextjs-expert", enabled: true },
        { source: "reactjs/react.dev", name: "react-expert", enabled: true },
      ],
      binaries: [
        { name: "nodejs", method: "apt" },
      ],
    },
  },
  {
    id: "python-api",
    name: "Python FastAPI",
    description:
      "FastAPI backend with SQLAlchemy, Pydantic models, async endpoints, and auto-generated OpenAPI docs.",
    icon: "Server",
    category: "Backend",
    color: "from-yellow-500/20 to-green-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a senior Python backend developer. Use FastAPI with async patterns, Pydantic for validation, SQLAlchemy for ORM, and follow PEP 8. Write comprehensive tests with pytest.",
      },
      pre_commands: [
        {
          name: "setup-venv",
          command: "python3 -m venv .venv && . .venv/bin/activate && pip install fastapi uvicorn sqlalchemy pydantic alembic pytest httpx",
          workdir: "/workspace/app",
        },
        {
          name: "scaffold",
          command:
            "mkdir -p app/api app/models app/schemas app/core tests && touch app/__init__.py app/api/__init__.py app/models/__init__.py app/schemas/__init__.py app/core/__init__.py",
          workdir: "/workspace/app",
        },
      ],
      skills: [
        { source: "fastapi/fastapi", enabled: true },
      ],
      binaries: [
        { name: "python3-pip", method: "apt" },
        { name: "python3-venv", method: "apt" },
      ],
      env_vars: {
        PYTHONPATH: "/workspace/app",
      },
    },
  },
  {
    id: "rust-cli",
    name: "Rust CLI Tool",
    description:
      "Rust CLI application with clap for argument parsing, color output, error handling with anyhow, and cross-platform support.",
    icon: "Terminal",
    category: "Systems",
    color: "from-orange-500/20 to-red-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a Rust systems programmer. Write idiomatic Rust with proper error handling using anyhow/thiserror. Use clap for CLI args. Follow Rust API guidelines and write tests.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command: "cargo init app && cd app && cargo add clap --features derive && cargo add anyhow serde serde_json tokio --features tokio/full",
          workdir: "/workspace",
        },
      ],
      skills: [
        { source: "nicepkg/aide-docs", enabled: true },
      ],
      binaries: [
        { name: "build-essential", method: "apt" },
      ],
    },
  },
  {
    id: "data-science",
    name: "Data Science Notebook",
    description:
      "Python data science environment with Jupyter, pandas, numpy, matplotlib, scikit-learn, and sample datasets.",
    icon: "BarChart3",
    category: "Data",
    color: "from-purple-500/20 to-pink-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a data scientist. Use pandas for data manipulation, matplotlib/seaborn for visualization, scikit-learn for ML. Write clean, reproducible analysis code with proper documentation.",
      },
      pre_commands: [
        {
          name: "setup",
          command:
            "python3 -m venv .venv && . .venv/bin/activate && pip install jupyter pandas numpy matplotlib seaborn scikit-learn plotly",
          workdir: "/workspace/app",
        },
        {
          name: "dirs",
          command: "mkdir -p data notebooks models",
          workdir: "/workspace/app",
        },
      ],
      binaries: [
        { name: "python3-pip", method: "apt" },
        { name: "python3-venv", method: "apt" },
      ],
    },
  },
  {
    id: "go-microservice",
    name: "Go Microservice",
    description:
      "Go HTTP microservice with Chi router, structured logging, health checks, Docker-ready, and comprehensive tests.",
    icon: "Box",
    category: "Backend",
    color: "from-cyan-500/20 to-blue-500/10",
    provider: "claude-code",
    spec: {
      agent: {
        provider: "claude-code",
        system_prompt:
          "You are a Go backend developer. Write idiomatic Go with proper error handling, structured logging (slog), and clean architecture. Use Chi for routing. Write table-driven tests.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "mkdir -p app && cd app && go mod init myservice && go get github.com/go-chi/chi/v5 github.com/go-chi/chi/v5/middleware",
          workdir: "/workspace",
        },
        {
          name: "dirs",
          command: "mkdir -p cmd/server internal/handler internal/service internal/model",
          workdir: "/workspace/app",
        },
      ],
      binaries: [
        { name: "golang-go", method: "apt" },
      ],
    },
  },
  // ── OpenAI Codex starters ─────────────────────────────────
  {
    id: "codex-typescript",
    name: "TypeScript Project",
    description:
      "Node.js TypeScript project with strict tsconfig, ESLint, Vitest, and modern ES module setup.",
    icon: "FileCode",
    category: "Backend",
    color: "from-emerald-500/20 to-teal-500/10",
    provider: "codex",
    spec: {
      agent: {
        provider: "codex",
        system_prompt:
          "You are a TypeScript expert. Use strict TypeScript with proper types (no any). Follow modern Node.js patterns with ESM imports. Write tests with Vitest.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "mkdir app && cd app && npm init -y && npm install typescript @types/node vitest tsx -D && npx tsc --init --strict --target ES2022 --module NodeNext --moduleResolution NodeNext",
          workdir: "/workspace",
        },
        {
          name: "dirs",
          command: "mkdir -p src tests",
          workdir: "/workspace/app",
        },
      ],
      binaries: [
        { name: "nodejs", method: "apt" },
      ],
    },
  },
  {
    id: "codex-django",
    name: "Django Web App",
    description:
      "Django 5 web application with REST framework, PostgreSQL-ready models, and admin interface configured.",
    icon: "Layout",
    category: "Web",
    color: "from-green-600/20 to-emerald-500/10",
    provider: "codex",
    spec: {
      agent: {
        provider: "codex",
        system_prompt:
          "You are a Django expert. Use Django best practices including class-based views, proper model design, DRF for APIs, and comprehensive test coverage.",
      },
      pre_commands: [
        {
          name: "setup",
          command:
            "python3 -m venv .venv && . .venv/bin/activate && pip install django djangorestframework django-cors-headers",
          workdir: "/workspace/app",
        },
        {
          name: "scaffold",
          command:
            ". .venv/bin/activate && django-admin startproject myapp . && python manage.py startapp core",
          workdir: "/workspace/app",
        },
      ],
      binaries: [
        { name: "python3-pip", method: "apt" },
        { name: "python3-venv", method: "apt" },
      ],
    },
  },
  // ── Gemini starters ───────────────────────────────────────
  {
    id: "gemini-flutter",
    name: "Flutter Mobile App",
    description:
      "Flutter cross-platform mobile app with Material 3 design, state management, and navigation setup.",
    icon: "Smartphone",
    category: "Mobile",
    color: "from-blue-400/20 to-indigo-500/10",
    provider: "gemini",
    spec: {
      agent: {
        provider: "gemini",
        system_prompt:
          "You are a Flutter mobile developer. Use Material 3, Riverpod for state management, GoRouter for navigation. Write widget tests.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "flutter create --org com.example --platforms android,ios app",
          workdir: "/workspace",
        },
        {
          name: "deps",
          command:
            "flutter pub add flutter_riverpod go_router && flutter pub add dev:flutter_lints",
          workdir: "/workspace/app",
        },
      ],
    },
  },
  {
    id: "gemini-ml-pipeline",
    name: "ML Training Pipeline",
    description:
      "Machine learning pipeline with PyTorch, data preprocessing, model training, evaluation, and experiment tracking.",
    icon: "BrainCircuit",
    category: "Data",
    color: "from-violet-500/20 to-fuchsia-500/10",
    provider: "gemini",
    spec: {
      agent: {
        provider: "gemini",
        system_prompt:
          "You are an ML engineer. Use PyTorch for model training, properly handle data loading and preprocessing. Track experiments with MLflow. Write modular, reproducible training code.",
      },
      pre_commands: [
        {
          name: "setup",
          command:
            "python3 -m venv .venv && . .venv/bin/activate && pip install torch torchvision numpy pandas scikit-learn mlflow tensorboard tqdm",
          workdir: "/workspace/app",
        },
        {
          name: "dirs",
          command: "mkdir -p data models src/data src/models src/training experiments",
          workdir: "/workspace/app",
        },
      ],
      binaries: [
        { name: "python3-pip", method: "apt" },
        { name: "python3-venv", method: "apt" },
      ],
    },
  },
  // ── Cursor starters ───────────────────────────────────────
  {
    id: "cursor-chrome-extension",
    name: "Chrome Extension",
    description:
      "Chrome Extension Manifest V3 with TypeScript, Webpack bundling, popup UI, and content script setup.",
    icon: "Chrome",
    category: "Tools",
    color: "from-amber-500/20 to-yellow-500/10",
    provider: "cursor",
    spec: {
      agent: {
        provider: "cursor",
        system_prompt:
          "You are building a Chrome Extension with Manifest V3. Use TypeScript, organize code into background, content, and popup modules. Follow Chrome Extension best practices for security.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "mkdir -p app/src/{background,content,popup} app/public && cd app && npm init -y && npm install -D typescript webpack webpack-cli ts-loader copy-webpack-plugin @types/chrome",
          workdir: "/workspace",
        },
      ],
      binaries: [
        { name: "nodejs", method: "apt" },
      ],
    },
  },
  {
    id: "cursor-vscode-extension",
    name: "VS Code Extension",
    description:
      "VS Code extension with TypeScript, command palette integration, webview panel, and extension testing setup.",
    icon: "Blocks",
    category: "Tools",
    color: "from-indigo-500/20 to-violet-500/10",
    provider: "cursor",
    spec: {
      agent: {
        provider: "cursor",
        system_prompt:
          "You are building a VS Code extension. Use the VS Code Extension API with TypeScript. Follow the extension guidelines, implement proper activation events, and use webview for custom UI.",
      },
      pre_commands: [
        {
          name: "scaffold",
          command:
            "npx --yes yo generator-code --extensionType ts --extensionName my-extension --extensionDescription '' --gitInit false --pkgManager npm --webpack false --extensionDisplayName 'My Extension' --openWith skip",
          workdir: "/workspace",
        },
      ],
      binaries: [
        { name: "nodejs", method: "apt" },
      ],
    },
  },
];

export const TEMPLATE_CATEGORIES = [
  { id: "all", label: "All" },
  { id: "Web", label: "Web" },
  { id: "Backend", label: "Backend" },
  { id: "Systems", label: "Systems" },
  { id: "Data", label: "Data" },
  { id: "Mobile", label: "Mobile" },
  { id: "Tools", label: "Tools" },
] as const;
