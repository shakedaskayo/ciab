FROM node:22-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    git curl ca-certificates openssh-client \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g @anthropic-ai/gemini-cli

RUN useradd -m -s /bin/bash agent
USER agent
WORKDIR /workspace

ENTRYPOINT ["gemini"]
