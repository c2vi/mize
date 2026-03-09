# Mize

Mize is a strongly typed "filesystem" for the age of connectivity, elevating the Unix file philosophy into modern distributed computing.

## Codebase Structure

```
packages/
├── mize/          # Core Mize framework (Rust)
├── marts/         # Mize parts collection (Rust/TypeScript)
├── mme/           # Mize Module Environment (Rust)
├── ppc/           # Platform-specific components
├── vic/           # Victor CLI tool
└── ac_mize_macros/# Mize macros
```

## Key Components

To understand the main components of the architecture you canr efer to `architecture.md`.

## Require clarification and plan approval before making code changes

Before making any code changes other than the changelog, you must follow this two-step process:

### Step 1: Ask Clarifying Questions

- Always ask at least one clarifying question about the user's request
- Understand the full scope and context of what they're asking for
- Clarify any ambiguous requirements or edge cases
- Ask about preferred approaches if multiple solutions exist
- Confirm the expected behavior and user experience

### Step 2: Present Implementation Plan

- After receiving clarification, present a detailed implementation plan
- Break down the work into specific, actionable steps
- Identify which files will be created, modified, or deleted
- Explain the technical approach and any architectural decisions
- Highlight any potential risks, trade-offs, or dependencies
- Estimate the complexity and scope of changes
- **Wait for explicit user approval** before proceeding with any code changes

### Approval Requirements

- User must explicitly approve the plan with words like "yes", "approved", "proceed", "go ahead", or similar
- If the user suggests modifications to the plan, incorporate them and seek re-approval
- Do not assume silence or ambiguous responses mean approval

### Exceptions

- This process may be skipped only for trivial changes like fixing obvious typos or formatting
- When in doubt, always follow the full process rather than assuming an exception applies

### Example Workflow

1. User: "Add a login form to the app"
2. Assistant: "I'd like to clarify a few things about the login form: [questions]"
3. User: [provides answers]
4. Assistant: "Based on your requirements, here's my implementation plan: [detailed plan]. Does this approach look good to you?"
5. User: "Yes, that looks good"
6. Assistant: [writes plan to file].
7. Assistant: [proceeds with implementation].
