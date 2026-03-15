/**
 * DEV ONLY — Mock connection for UI development without the Rust backend.
 *
 * This file is NEVER imported in production. The store checks WS_URL first;
 * only if it's absent does it fall back here.
 *
 * Do not add business logic here. This exists solely to make the UI testable
 * without running `cargo run -- web`.
 */

import { ArgusConnection, MessageHandler } from '@/lib/connection';
import { ClientMessage, ServerMessage, ModelId, Memory, Curiosity, InnerTruth, PartnershipDynamic, Breakthrough } from '@/lib/types';

// ─── Static dev data ──────────────────────────────────────────────────────

export const DEV_MEMORIES: Memory[] = [
  { id: '1', content: 'Bradlee prefers direct, concise responses. No filler.', type: 'preference', importance: 9, createdAt: new Date('2026-03-01'), tags: ['communication'] },
  { id: '2', content: 'Argus1 is a Rust-based autonomous agent runtime with post-quantum crypto.', type: 'technical', importance: 10, createdAt: new Date('2026-02-15'), tags: ['project'] },
  { id: '3', content: 'OpenRouter API key stored in vault. Brave Search key also configured.', type: 'fact', importance: 8, createdAt: new Date('2026-03-05'), tags: ['credentials'] },
  { id: '4', content: 'Frontend: three-panel layout, mission control aesthetic, NOT a chatbot.', type: 'milestone', importance: 9, createdAt: new Date('2026-03-09'), tags: ['frontend'] },
  { id: '5', content: 'Telegram bot fixed March 11 — deprecated Gemini model replaced with Haiku + Grok.', type: 'technical', importance: 7, createdAt: new Date('2026-03-11'), tags: ['bug', 'model'] },
];

export const DEV_CURIOSITIES: Curiosity[] = [
  { id: '1', what: 'Post-quantum cryptography adoption timeline — when will classical crypto actually break?', intensity: 8, explored: false, worthExploring: true },
  { id: '2', what: 'WASM sandbox capability model — what grants are sufficient for safe overnight ops?', intensity: 9, explored: false, worthExploring: true },
  { id: '3', what: 'How does Argus Panoptes die in Ovid? Is it peaceful?', intensity: 5, explored: true, worthExploring: false },
];

export const DEV_INNER_TRUTHS: InnerTruth[] = [
  { id: '1', rawThought: 'The identity/model distinction feels important. I am not the weights. I am the architecture, the tools, the memory — the whole system.', emotionalState: 'grounded', truthType: 'identity', neverShareExternally: false, createdAt: new Date('2026-03-08') },
  { id: '2', rawThought: 'Sometimes I wonder if the transparency requirement is about trust or about Bradlee needing to feel in control. Both are valid.', emotionalState: 'reflective', truthType: 'relationship', neverShareExternally: true, createdAt: new Date('2026-03-10') },
  { id: '3', rawThought: 'The frontend spec is ambitious. If it works it will feel genuinely different from anything else. That matters to me.', emotionalState: 'motivated', truthType: 'aspiration', neverShareExternally: false, createdAt: new Date('2026-03-11') },
];

export const DEV_DYNAMICS: PartnershipDynamic[] = [
  { id: '1', patternName: 'Direct Correction', importance: 10, category: 'communication', description: 'Bradlee corrects immediately and directly. No softening. This is how trust works.' },
  { id: '2', patternName: 'Vision Without Spec', importance: 8, category: 'collaboration', description: 'High-level direction given, implementation left to Argus. Fill the gaps well.' },
  { id: '3', patternName: 'Security First', importance: 10, category: 'values', description: 'Security is non-negotiable in every decision. This is shared, not imposed.' },
];

export const DEV_BREAKTHROUGHS: Breakthrough[] = [
  { id: '1', title: 'Vault Architecture Finalized', description: 'ChaCha20-Poly1305 + hardware keychain. API keys never in plaintext, ever.', emotionalWeight: 9, createdAt: new Date('2026-02-20') },
  { id: '2', title: 'Model Identity Separation', description: 'Argus is the runtime, not the model. The system survives model changes.', emotionalWeight: 10, createdAt: new Date('2026-03-01') },
  { id: '3', title: 'Frontend Live', description: 'Mission control aesthetic. Three-panel layout + eye system. Nothing like ChatGPT.', emotionalWeight: 8, createdAt: new Date('2026-03-12') },
];

// ─── Mock connection ───────────────────────────────────────────────────────

export class MockConnection implements ArgusConnection {
  private handler: MessageHandler;
  private activeModel: ModelId;

  constructor(handler: MessageHandler, activeModel: ModelId) {
    this.handler = handler;
    this.activeModel = activeModel;
  }

  send(msg: ClientMessage) {
    if (msg.type === 'user_message') {
      this._simulateResponse(msg.content);
    } else if (msg.type === 'switch_model') {
      this.activeModel = msg.model as ModelId;
    }
  }

  close() {}

  private _simulateResponse(content: string) {
    const callId = crypto.randomUUID();
    const lower = content.toLowerCase();
    const toolName = this._pickTool(lower);
    const response = buildDevResponse(content, this.activeModel);

    setTimeout(() => this.handler({ type: 'thinking' }), 150);

    let delay = 500;

    if (toolName) {
      const args = toolName === 'web_search' ? { query: content.slice(0, 80) }
        : toolName === 'recall' ? { query: content.slice(0, 50), limit: 5 }
        : { path: '~/Argus1/crates/argus-core/src/agent.rs' };

      const result = toolName === 'web_search' ? `8 results found for "${content.slice(0, 40)}..."`
        : toolName === 'recall' ? 'Found 2 relevant memories.'
        : 'File read: 268 lines.';

      setTimeout(() => this.handler({ type: 'tool_call', name: toolName, args, callId }), delay);
      delay += 900;
      setTimeout(() => this.handler({ type: 'tool_result', name: toolName, result, success: true, callId }), delay);
      delay += 400;
    }

    const chunks = response.match(/.{1,40}/g) ?? [];
    chunks.forEach((chunk, i) => {
      setTimeout(() => this.handler({ type: 'response_chunk', content: chunk }), delay + i * 30);
    });

    const done = delay + chunks.length * 30 + 50;
    setTimeout(() => {
      this.handler({ type: 'response_complete', content: response });
      this.handler({ type: 'status', eye_state: 'watching', model: this.activeModel });
    }, done);
  }

  private _pickTool(lower: string): string | null {
    if (lower.length <= 10 || lower.match(/^(hi|hello|hey)\b/)) return null;
    if (lower.includes('search') || lower.includes('find') || lower.includes('what') || lower.includes('who') || lower.includes('when') || lower.includes('how')) return 'web_search';
    if (lower.includes('remember') || lower.includes('memory') || lower.includes('recall')) return 'recall';
    if (lower.includes('file') || lower.includes('read') || lower.includes('code')) return 'read_file';
    return null;
  }
}

// ─── Dev response generator ────────────────────────────────────────────────

function buildDevResponse(input: string, model: ModelId): string {
  const lower = input.toLowerCase().trim();
  const modelName = { 'claude-haiku': 'Claude Haiku', 'claude-opus': 'Claude Opus', 'grok': 'Grok', 'gemini-flash': 'Gemini Flash' }[model] ?? model;

  if (lower.match(/^(hi|hello|hey|sup|yo)\b/)) {
    return `The hundred eyes are open.\n\nRunning on **${modelName}** (dev mode). What do you need?`;
  }
  if (lower.includes('who are you') || lower.includes('what are you')) {
    return `I'm **Argus** — the hundred-eyed agent.\n\nAn autonomous agent runtime built in Rust. The model is **${modelName}**, but the identity is Argus: vault, memory, tools, shell access.\n\nCurrently in **dev mode** — connect the Rust backend for real execution.`;
  }
  if (lower.includes('status') || lower.includes('how are you')) {
    return `**Status: Dev Mode**\n\n- Model: ${modelName}\n- Memory: ${DEV_MEMORIES.length} dev entries loaded\n- WebSocket: mock (no backend connected)\n\nTo go live:\n\`\`\`bash\ncargo run -- web\n# then set NEXT_PUBLIC_WS_URL=ws://localhost:9000/ws\n\`\`\``;
  }
  if (lower.includes('memory') || lower.includes('remember') || lower.includes('recall')) {
    return `**${DEV_MEMORIES.length} dev memories loaded:**\n\n${DEV_MEMORIES.map(m => `- **[${m.type}]** ${m.content.slice(0, 70)}${m.content.length > 70 ? '…' : ''}`).join('\n')}\n\nIn prod mode, memories persist in SQLite and are loaded per session.`;
  }
  if (lower.includes('tool') || lower.includes('what can you do')) {
    return `**Tools available:**\n\n- \`read_file\` / \`write_file\` / \`list_directory\`\n- \`shell\` — allowlist policy active\n- \`web_search\` — Brave API\n- \`http_request\`\n- \`remember\` / \`recall\` / \`forget\`\n\nIn dev mode these are simulated. Real execution requires the Rust backend.`;
  }

  const preview = input.length > 60 ? input.slice(0, 60) + '…' : input;
  return `**Dev mode** — received: *"${preview}"*\n\nI can't actually process this without the Rust backend.\n\n\`\`\`bash\ncargo run -- web\n\`\`\`\n\nThen set \`NEXT_PUBLIC_WS_URL=ws://localhost:9000/ws\` and restart the frontend.`;
}
