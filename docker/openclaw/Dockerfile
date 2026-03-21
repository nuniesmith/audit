# syntax=docker/dockerfile:1.7
# =============================================================================
# RustAssistant — Custom OpenClaw Image
# =============================================================================
#
# Layers RustAssistant agent config, workspace defaults, and environment
# overrides on top of the upstream OpenClaw base image.
#
# The base image is built from source by build.sh (see Dockerfile.upstream)
# or pulled from a registry. Either way, this Dockerfile adds our customisation
# layer so the container starts pre-configured for the RA stack.
#
# Build (standalone — usually invoked by build.sh):
#   docker build -f docker/openclaw/Dockerfile \
#       --build-arg OPENCLAW_BASE_IMAGE=openclaw-base:local \
#       -t openclaw:local .
#
# Build args:
#   OPENCLAW_BASE_IMAGE  Base image tag (default: openclaw-base:local)
# =============================================================================

ARG OPENCLAW_BASE_IMAGE=openclaw-base:local
FROM ${OPENCLAW_BASE_IMAGE}

LABEL org.opencontainers.image.source="https://github.com/nuniesmith/rustassistant" \
      org.opencontainers.image.title="RustAssistant OpenClaw" \
      org.opencontainers.image.description="OpenClaw gateway customised for the RustAssistant stack"

# ── Bake in RustAssistant agent configuration ────────────────────────────────
# agents.yml defines the briefer, session-scribe, and default agent personas
# along with tool bindings (GitHub, Discord, RA semantic search, RA ingest).
USER root
RUN mkdir -p /home/node/.openclaw/config \
             /home/node/.openclaw/workspace/sessions \
             /home/node/.openclaw/workspace/notes \
    && chown -R node:node /home/node/.openclaw

COPY --chown=node:node openclaw/config/agents.yml /home/node/.openclaw/config/agents.yml

# ── RustAssistant-specific environment defaults ──────────────────────────────
# These are defaults — docker-compose environment{} and .env still override.
ENV OPENCLAW_CONFIG_DIR=/home/node/.openclaw \
    OPENCLAW_WORKSPACE_DIR=/home/node/.openclaw/workspace \
    NODE_ENV=production

# ── Drop back to non-root ───────────────────────────────────────────────────
USER node

# Inherit upstream HEALTHCHECK and CMD — no changes needed.
# CMD ["node", "openclaw.mjs", "gateway", "--allow-unconfigured"]
# (overridden by docker-compose command: stanza anyway)
