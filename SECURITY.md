# Security

## Reporting a vulnerability

Please report security problems privately, not in a public issue: use **Report a vulnerability**
on this repository's **Security** tab (GitHub's private vulnerability reporting). Include what is
affected, how to reproduce it, and the TuxBridge version.

We'll reply as soon as we can, keep you updated while we work on a fix, and credit you in the
release notes if you'd like.

## Supported versions

TuxBridge is in pre-release; fixes go into the newest version, which installed copies can get from
Settings → **Update TuxBridge**.

## How TuxBridge protects you

- Updates are signed with Rookery Software's key, and TuxBridge installs only updates whose
  signature matches.
- Linux systems, Docker Engine, Ollama and AI models are downloaded from their official sources.
- TuxBridge's Docker engine listens only on this PC (`127.0.0.1`) and accepts only clients with a
  certificate from its own authority.
- Ask Pip's AI runs on this PC; nothing you ask leaves it.
- Feedback reports are sent only when you choose to, and their diagnostic report hides your
  Windows user name.
