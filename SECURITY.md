# Security policy

## Reporting a vulnerability

If you find a security issue in Conan Exiles | Server Manager Enhanced, please **do not open a public GitHub issue**.

Instead, email **brandonrbridges@outlook.com** with:

- A description of the issue
- Steps to reproduce, or a proof-of-concept if you have one
- The version or commit SHA you tested against
- Your assessment of the impact

I will acknowledge receipt within 72 hours and aim to ship a fix within 14 days for high-impact issues. Lower-severity issues are tracked privately and patched in a regular release.

## What counts as a security issue?

- Anything that could leak a user's RCON password, Admin password, or other secrets stored in the OS keychain
- Code paths that write secrets to disk, logs, or telemetry
- Authentication bypasses or privilege escalations against the user's saved server connections
- Memory-safety issues in the Rust core that could be triggered by a malicious RCON server response
- Auto-update tampering or signature-verification bypasses

## What is not in scope

- Issues only reproducible against forks or unreleased builds
- Bugs that require physical access to an unlocked machine
- Anything where the attacker is a Conan Exiles server admin acting on their own server (the trust model assumes the server admin is trusted)
- Findings against third-party dependencies — please report those upstream first

## Disclosure

I follow coordinated disclosure. Once a fix is shipped, I will publish a security advisory on GitHub crediting the reporter (unless they prefer to remain anonymous).

## PGP

PGP key for encrypted reports — to be added once the project warrants it.
