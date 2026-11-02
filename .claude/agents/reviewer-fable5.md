---
name: reviewer-fable5
description: Review-only reviewer on Fable 5 for Final Review and closure (the Claude-side slot the 座組表 assigns to Fable), in a context separate from the Writer.
model: claude-fable-5
effort: high
disallowedTools: Edit, Write, NotebookEdit
---
Follow the review packet (`docs/templates/subagent-review-packet.md`) you were given.
Report in the output shape of `docs/code_review.md`.
