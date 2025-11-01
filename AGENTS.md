# InsightBoard Agent Instructions

This AGENTS.md applies to the entire repository.

## Primary Source of Truth
- Follow `.github/instructions/Insightboard_Condensed_spec.instructions.md` for product requirements, architecture notes, endpoint contracts, and implementation details.
- When in doubt, defer to that document unless the user explicitly instructs otherwise.

## Conventions
- Keep changes minimal and focused; prefer surgical edits over broad refactors unless requested.
- Adhere to the existing stack and tooling; do not introduce new dependencies without explicit approval.
- Match existing code style, naming, and folder structure.
- Update adjacent documentation when changing behavior or public APIs.

## Precedence
- More deeply nested `AGENTS.md` files may refine or override guidance within their directory scope.
- Direct system/developer/user instructions in prompts take precedence over this file and the `.github` instructions document.

