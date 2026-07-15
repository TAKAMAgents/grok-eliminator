# Design notes

The cleanup chain is modeled as a composition of local morphisms:

```text
CLI input -> validated operation -> local artifact audit -> guarded mutation -> report
```

The important invariant is that a Grok command must not remain reachable from
the user's shell while unrelated terminal capabilities and unrelated source
data remain intact. Platform-specific package roots and environment stores are
boundary adapters. The signed cmux bundle is a macOS-only external boundary:
the tool changes the user's shell path and guard, never the application bundle.
