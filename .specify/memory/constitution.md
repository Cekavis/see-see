<!--
Sync Impact Report
- Version change: unratified template -> 1.0.0
- Modified principles:
  - Placeholder 1 -> I. Maintainable Quality
  - Placeholder 2 -> II. Proportionate Testing (NON-NEGOTIABLE)
  - Placeholder 3 -> III. Consistent User Experience
  - Placeholder 4 -> IV. Accessible and Beautiful UI
- Added sections: Quality Gates; Development Workflow
- Removed sections: unused fifth principle placeholder
- Templates:
  - ✅ .specify/templates/plan-template.md
  - ✅ .specify/templates/spec-template.md
  - ✅ .specify/templates/tasks-template.md
- Follow-up TODOs: none
-->
# See See Constitution

## Core Principles

### I. Maintainable Quality
Code MUST be correct, readable, and no more complex than the current requirement demands.
Implementations MUST reuse established project patterns before adding abstractions or dependencies.
Public behavior, failure handling, and trust-boundary validation MUST be explicit. Complexity and new
dependencies require a written justification showing why a simpler option is insufficient.

Rationale: the smallest maintainable solution reduces defects, review cost, and long-term ownership.

### II. Proportionate Testing (NON-NEGOTIABLE)
Every behavioral change MUST include automated evidence proportionate to its risk. Logic requires focused
unit tests; boundaries, contracts, and critical user journeys require integration tests. A regression fix
MUST include a check that fails without the fix. Documentation-only and visual-copy-only changes MAY use
targeted manual verification when no executable behavior changes, and the verification MUST be recorded.
Existing tests MUST remain enabled and MUST NOT be weakened to obtain a passing result.

Rationale: verification must catch meaningful regressions without creating ceremonial test suites.

### III. Consistent User Experience
Equivalent actions and states MUST behave consistently across the product. Features MUST define loading,
empty, success, error, disabled, and recovery states where applicable. Language, navigation, feedback,
keyboard behavior, and platform conventions MUST follow existing product patterns. Intentional deviations
MUST be documented in the specification and approved during review.

Rationale: predictable behavior reduces user effort and makes the product feel coherent.

### IV. Accessible and Beautiful UI
User-facing work MUST reuse shared design tokens and components before introducing one-off styling. It MUST
meet applicable accessibility requirements, support intended viewport sizes, preserve clear visual hierarchy,
and define interaction states. Completion requires objective checks for accessibility, spacing, typography,
responsiveness, and consistency, plus human review of hierarchy, polish, and visual coherence.

Rationale: beauty is durable when it is systematic, inclusive, and verified rather than subjective alone.

## Quality Gates

- Formatting, static analysis, and the narrowest relevant automated tests MUST pass before completion.
- User-facing changes MUST include acceptance criteria for consistency, accessibility, responsive behavior,
  and required interface states.
- User-facing changes MUST receive human visual review at representative viewport sizes.
- Known unrelated failures MUST be recorded; new failures MUST be resolved before completion.
- Security, accessibility, error handling, or data-integrity safeguards MUST NOT be removed for simplicity.

## Development Workflow

1. Specifications MUST describe independently testable user outcomes and measurable success criteria.
2. Plans MUST document constitution compliance and justify deviations, complexity, and new dependencies.
3. Tasks MUST include required automated tests and, for user-facing work, accessibility and visual review.
4. Reviews MUST verify behavior, regression coverage, user experience consistency, and UI quality.
5. Work is complete only when required checks pass and verification evidence is recorded.

## Governance

This constitution supersedes conflicting project practices and templates. Amendments require a documented
reason, review of affected templates and guidance, and an explicit semantic version change. MAJOR versions
remove or redefine governing commitments; MINOR versions add or materially expand them; PATCH versions clarify
wording without changing obligations. Every specification, plan, task list, and code review MUST check relevant
principles. Any exception MUST be narrow, documented with its risk and simpler alternatives, and approved before
implementation.

**Version**: 1.0.0 | **Ratified**: 2026-07-23 | **Last Amended**: 2026-07-23
