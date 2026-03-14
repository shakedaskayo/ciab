FROM node:22-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    git curl ca-certificates openssh-client \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://cursor.sh/install.sh | bash || true

RUN useradd -m -s /bin/bash agent
USER agent
WORKDIR /workspace

ENTRYPOINT ["cursor"]
