---
name: opsx-fix
description: Fix or amend an active OpenSpec change plan without losing current progress. Handles race conditions, Git constraints, and Zombie tasks.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.5.0"
---

Fix or amend an active OpenSpec change plan (tasks and design) without losing current progress.

**Store selection:** If the user names a store (a store is a standalone OpenSpec repo registered on this machine) or the work lives in one, run `openspec store list --json` to discover registered store ids, then pass `--store <id>` on the commands that read or write specs and changes (`new change`, `status`, `instructions`, `list`, `show`, `validate`, `archive`, `doctor`, `context`). Other commands do not take the flag. Hints printed by commands already carry the flag; keep it on follow-ups. Without a store, commands act on the nearest local `openspec/` root.

**Input**: Optionally specify a change name and the reason for the fix. If omitted, prompt the user for the change name and the fix description.

**Pre-flight Checks (CRITICAL)**
1. **Concurrency Check**: Verify that no other `/opsx:apply` agent or subagent is currently running. Ask the user to confirm if unsure.
2. **Git State Check**: Verify you are on the correct `feature/<change-name>` branch via `git status`. Do not proceed if on `main`.

**Steps**

1. **Select the change and the fix directive**
   - Identify the active change and the user's instructions for the pivot.
   - Always announce: "Preparing fix for change: <name>".

2. **Dynamically Locate Files**
   - Run `openspec status --change "<name>" --json`
   - Parse the JSON to get `artifactPaths` (this gives you the exact paths to `design.md`, `tasks.md`, `proposal.md`, etc., regardless of the schema).

3. **Read Rules and Context**
   - Run `openspec instructions tasks --change "<name>" --json` to load the mandatory formatting rules for tasks.
   - Read the current `design.md` and `tasks.md` from disk (do NOT rely on conversational memory to avoid token exhaustion).

4. **Apply the Fix (Safe Mutation)**
   - **Update Design**: Do NOT delete the original design. Append a block `### Corrección de Ruta (Fix)` explaining the pivot and the new strategy.
   - **Update Tasks**:
     - Keep completed tasks (`- [x]`) intact.
     - Discard obsolete pending tasks by marking them as complete with a note to avoid blocking the parser: `- [x] ~~(DESCARTADO) Reason~~` or completely convert them to plain text.
     - **Zombie Code Analysis**: If a completed task is invalidated by the fix, add a NEW pending task to revert or refactor its codebase (e.g., `- [ ] Refactor: undo previous implementation of discarded approach`) and also a task to remove any zombie unit tests.
     - Append the new tasks required by the fix.

5. **Post-Flight Validation**
   - Do NOT modify the YAML frontmatter of any artifact.
   - Run `openspec status --change "<name>"` to verify the artifact tree is healthy and alert the user if the new design requires generating brand new artifacts.

**Output**
Summarize the fix applied, listing the discarded tasks and the new tasks added. Prompt the user to resume work with `/opsx:apply`.
