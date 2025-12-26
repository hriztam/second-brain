# second-brain

## AI powered hyper-personalised agent designed to increase the productivity of the user and promote hands-free control over system

## Idea

- It is a voice-first agent that acts as a virtual second brain for the user
- Thinking of keeping three layers/ways to talk to the agent (Dictation Mode, Intelligent Mode, Agent Mode)
- Dictation Mode: Transcribers whatever user says with ultra fast latency
- Intelligent Mode: Generates a LLM response for user's request (eg. "Write an application to aply for an internship at Google")
- Agent Mode: This is where the agentic abilities of second brain lies
- - Integrations: Users can connect their apps/platforms and the agent can perform actions on it (Slack, Calendar, etc) (Can be done using Composio)
- - Browser Use: The agent can interact with the browser and perform actions like open a tab, search, etc
- - System control: Designed for macos, it will use AppleScript to execute users request

## Technolgies

### My initial thought process is to go for Rust for the core architecture and Swift for the UI Layer, I will try to use and integrate other softwares to have a smooth build environment while being swift (no pun intended lol)

### Things that I would need along the way

- Transcriber/ASR (local or streaming)
- LLM (Either local or streaming)
- For the agentic capabilties:-
- - Integrations: Composio MCP (Rube)
- - Browser use: browser use tool (Doesn't know how it works yet so just a placeholder)
- - System control: AppleScript

## UI Layer

- Dictation Mode: Only a indicator which expands where the user presses its hot-key (like Whispr Flow)
- Intelligent Mode: Same as Dictation mode but with a slightly different colour wave form (like Ito.ai)
- Agent Mode: A glass-like overlay UI which appears in blocks rather than whole screen (Assisting the user without breaking the workflow) (A jarvis-like look)
