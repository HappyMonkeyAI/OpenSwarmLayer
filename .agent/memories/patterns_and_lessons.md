# Patterns and Lessons

- Preserve the application's existing documentation and ADR numbering when adopting an external protocol; add project-shaped records after the local ADR sequence.
- Treat worker summaries and focused tests as partial evidence; the owner verifies the acceptance contract independently.
- Keep portable MCP intent routing separate from machine-local mounted-server state; never store credentials in either.
- Use worktrees for parallel changes and stage only owned paths when the checkout already contains user work.
