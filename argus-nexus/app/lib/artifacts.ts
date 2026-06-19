import { Artifact } from './types';
const RE = /<argus-artifact\s+type="([^"]+)"(?:\s+title="([^"]*)")?>([\s\S]*?)<\/argus-artifact>/g;

export function parseArtifacts(content: string) {
  const artifacts: Artifact[] = [];
  const cleanText = content.replace(RE, (_m, type, title, body) => {
    artifacts.push({ id: 'a' + Date.now() + artifacts.length, type: type.toLowerCase(), title: title || type, content: body.trim() });
    return '';
  }).trim();
  return { cleanText, artifacts };
}
