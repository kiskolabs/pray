# Drift sections and offline install

Continue the reference CLI toward the next spec-priority gap:

- keep `pray install --offline` limited to explicit local path packages
- make `pray drift` report grouped sections for lockfile, package, managed span, rendered file, and warning changes
- cover the new drift classifications with regression tests
