# Security Policy

`grok-eliminator` removes local Grok CLI artifacts and local credential exports.
It does not rotate, revoke, validate, or transmit xAI credentials.

## Supported versions

Security fixes are accepted for the latest code on the `main` branch until the
project publishes versioned releases.

## Reporting a vulnerability

Please report suspected vulnerabilities through GitHub private vulnerability
reporting when it is enabled for this repository. If that is unavailable, contact
the repository owner privately and avoid publishing exploit details in a public
issue.

Useful reports include:

- affected operating system,
- command that exposed the issue,
- expected and actual behavior,
- whether secret values were printed, read, persisted, or transmitted,
- minimal reproduction steps that do not include real credentials.

## Security boundaries

The tool must not:

- print API-key values,
- contact xAI or any external provider,
- modify signed application bundles such as `/Applications/cmux.app`,
- delete files merely because they contain the word Grok,
- mutate state during `audit` or `remove` without `--apply`.

External xAI credentials should be rotated outside this tool after local cleanup.
