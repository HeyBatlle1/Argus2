import { Artifact } from './types';

const ARTIFACT_RE = /<argus-artifact\s+type="([^"]+)"(?:\s+title="([^"]*)")?>([\s\S]*?)<\/argus-artifact>/g;

export interface ParsedContent {
  cleanText: string;
  artifacts: Artifact[];
}

/**
 * Extract <argus-artifact> blocks from agent response text.
 * Returns the text with artifact blocks removed + the parsed artifacts.
 */
export function parseArtifacts(content: string): ParsedContent {
  const artifacts: Artifact[] = [];
  let idx = 0;

  const cleanText = content.replace(ARTIFACT_RE, (_match, type, title, body) => {
    artifacts.push({
      id: `artifact-${Date.now()}-${idx++}`,
      type: type.trim().toLowerCase(),
      title: (title ?? type).trim() || type,
      content: body.trim(),
    });
    return '';
  }).trim();

  return { cleanText, artifacts };
}

/** Language label for display in the panel header */
export function artifactLabel(type: string): string {
  const map: Record<string, string> = {
    html: 'HTML',
    svg: 'SVG',
    markdown: 'Markdown',
    python: 'Python',
    javascript: 'JavaScript',
    js: 'JavaScript',
    typescript: 'TypeScript',
    ts: 'TypeScript',
    css: 'CSS',
    json: 'JSON',
    bash: 'Bash',
    sh: 'Shell',
    rust: 'Rust',
    go: 'Go',
  };
  return map[type] ?? type.toUpperCase();
}

/** True for types rendered visually (iframe / SVG), false for code types */
export function isVisualArtifact(type: string): boolean {
  return type === 'html' || type === 'svg' || type === 'markdown';
}
