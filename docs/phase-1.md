# Phase 1: Product definition and UX flow

## Scope
Build the v1 product spec and UX flow for a macOS-only, push-to-talk, voice-first agent with three modes. Cloud ASR/LLM is allowed. Target persona is an engineer. Hands-free means hotkey-driven voice with explicit confirmation steps for agent actions.

## Goals
- Define the three modes with clear purpose, entry/exit rules, and boundaries.
- Establish a command taxonomy and intent categories.
- Specify hands-free behavior and action confirmation rules.
- Set success metrics and latency budgets per mode.
- Draft UX flow: onboarding, triggers, error states, and confirmations.

## Non-goals
- Implement any ASR/LLM/tooling.
- Design visual UI details beyond high-level UX flow.
- Build integrations.

## Mode definitions

### Dictation mode
- Purpose: Low-latency transcription.
- Entry: Press-and-hold hotkey.
- Exit: Release hotkey.
- Output: Text inserted at cursor or copied to clipboard.
- Constraints: No tool calls or system actions.

### Intelligent mode
- Purpose: LLM response generation for user requests.
- Entry: Press-and-hold hotkey + mode toggle or voice prefix.
- Exit: Release hotkey; response appears in overlay.
- Output: Drafted content, suggestions, summaries.
- Constraints: No external tool calls by default.

### Agent mode
- Purpose: Execute multi-step tasks via tools and system control.
- Entry: Toggle into agent mode, press-and-hold hotkey to speak.
- Exit: Mode toggle off or inactivity timeout.
- Output: Action plan + execution with confirmations.
- Constraints: Requires explicit confirmations for high-risk actions.

## Command taxonomy
- Dictation: “write/type/dictate” instructions, no intent parsing beyond text.
- Knowledge: “explain/summarize/compare,” no actions.
- Drafting: “write/email/outline,” response text only.
- Planning: “plan/steps/checklist,” response text only.
- Actions: “open/create/send/delete/modify,” requires agent mode.
- System: “mute/volume/brightness/open app,” agent mode and explicit confirm.

## Hands-free behavior
- Hotkey is the primary trigger for speaking.
- Agent actions require a confirm step using “enter/confirm/yes.”
- High-risk actions require a two-step confirmation (e.g., “confirm delete”).
- No background listening; mic is active only during hotkey press.

## UX flow (text)

### Onboarding
1. Welcome screen: select modes enabled (default: Dictation + Intelligent).
2. Permissions: mic access, accessibility permissions for system control (optional).
3. Hotkey setup and test (push-to-talk).
4. Cloud settings: enable ASR/LLM, data retention preference.

### Dictation flow
1. User holds hotkey.
2. Waveform indicator appears.
3. Streaming transcript shown.
4. Release hotkey ends capture and inserts text.
5. Errors: mic/ASR failure -> inline toast with retry.

### Intelligent flow
1. User holds hotkey + intelligent mode.
2. Streaming transcript shown.
3. Release hotkey sends request.
4. Response in overlay; user can copy or insert.
5. Errors: LLM failure -> retry or fallback message.

### Agent flow
1. User toggles agent mode.
2. User holds hotkey and speaks task.
3. Planner returns steps; show summary.
4. Ask for confirmation on actions.
5. Execute; show status per step.
6. Errors: action failure -> show reason, ask to retry or adjust.

## Safety and confirmation policy
- High-risk actions: delete, send, purchase, system settings.
- Require explicit confirmation phrase.
- For system control, prompt with “Confirm [action]?” and require “confirm.”
- Always show a brief action summary before execution.

## Metrics and latency budgets
- Dictation: first partial text < 300ms; final text < 700ms after release.
- Intelligent: response start < 1.5s; total < 6s for typical request.
- Agent: plan summary < 3s; action step start < 2s after confirm.
- Reliability: 99% hotkey capture success; 95% task completion for common actions.

## Acceptance criteria
- Clear mode boundaries and activation rules documented.
- Command taxonomy covers at least 90% of expected requests.
- Confirmation policy defined for high-risk actions.
- Latency targets defined per mode.
- Text-based UX flow for onboarding and each mode.
