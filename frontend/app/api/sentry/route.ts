import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// ── Config ─────────────────────────────────────────────────────────────────
// In Docker: ARGUS_DAEMON_URL=http://argus-daemon:9000 (set in docker-compose)
// In local dev: falls back to localhost
const DAEMON_URL = process.env.ARGUS_DAEMON_URL ?? 'http://localhost:9000';
const PROXY_TIMEOUT_MS = 4_000;
const LOCAL_CMD_TIMEOUT_MS = 3_000;

// ── Main handler ───────────────────────────────────────────────────────────

export async function GET() {
  // Try the daemon endpoint first — it runs in the correct Linux environment
  // with Docker socket access and /proc visibility.
  try {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), PROXY_TIMEOUT_MS);
    try {
      const res = await fetch(`${DAEMON_URL}/sentry`, {
        signal: ctrl.signal,
        cache: 'no-store',
      });
      if (res.ok) {
        const data = await res.json();
        return Response.json(data);
      }
    } finally {
      clearTimeout(timer);
    }
  } catch {
    // Daemon unreachable — fall through to local fallback (dev mode / cold start)
  }

  // ── Local fallback (dev only — macOS / native Next.js) ─────────────────
  return Response.json(await localSentryData());
}

// ── Local data collection (macOS dev environment) ─────────────────────────

async function safe(cmd: string, timeoutMs = LOCAL_CMD_TIMEOUT_MS): Promise<string> {
  try {
    const { stdout } = await execAsync(cmd, { timeout: timeoutMs });
    return stdout.trim();
  } catch {
    return '';
  }
}

async function localSentryData() {
  const [memRaw, dockerRaw, psRaw] = await Promise.all([
    safe("top -l 1 -n 0 2>/dev/null | grep PhysMem"),
    safe('docker ps --format "{{.Names}}|{{.Status}}|{{.Ports}}" 2>/dev/null'),
    safe('ps aux 2>/dev/null | grep -E "argus|next" | grep -v grep'),
  ]);

  // macOS PhysMem: "PhysMem: 15G used (…), 113M unused."
  const memMatch = memRaw.match(/([\d.]+[GM])\s+used.*?([\d.]+[GM])\s+unused/);
  const memory = memMatch
    ? { used: memMatch[1], free: memMatch[2] }
    : { used: '?', free: '?' };

  const containers = dockerRaw
    ? dockerRaw.split('\n').filter(Boolean).map((line) => {
        const [name, status, ports] = line.split('|');
        const healthy   = (status ?? '').includes('healthy') && !(status ?? '').includes('unhealthy');
        const unhealthy = (status ?? '').includes('unhealthy');
        return { name: name ?? '', status: status ?? '', ports: ports ?? '', healthy, unhealthy };
      })
    : [];

  const processes: { name: string; pid: string; mem: string; uptime: string }[] = [];
  if (psRaw) {
    for (const line of psRaw.split('\n').filter(Boolean)) {
      const parts = line.trim().split(/\s+/);
      const pid   = parts[1] ?? '';
      const memMb = ((parseFloat(parts[5] ?? '0') || 0) / 1024).toFixed(0);
      const time  = parts[9] ?? '';
      const cmd   = parts.slice(10).join(' ');

      if (cmd.includes('argus'))     processes.push({ name: 'argus', pid, mem: `${memMb}MB`, uptime: time });
      else if (cmd.includes('next')) processes.push({ name: 'frontend (next)', pid, mem: `${memMb}MB`, uptime: time });
    }
  }

  return { memory, containers, processes, ts: Date.now() };
}
