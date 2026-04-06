import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function safe(cmd: string): Promise<string> {
  try {
    const { stdout } = await execAsync(cmd);
    return stdout.trim();
  } catch {
    return '';
  }
}

export async function GET() {
  const [memRaw, dockerRaw, psRaw] = await Promise.all([
    safe("top -l 1 -n 0 | grep PhysMem"),
    safe('docker ps --format "{{.Names}}|{{.Status}}|{{.Ports}}" 2>/dev/null'),
    safe('ps aux | grep -E "argus|next" | grep -v grep'),
  ]);

  // Parse PhysMem line: "PhysMem: 15G used (3263M wired, 1117M compressor), 113M unused."
  const memMatch = memRaw.match(/(\d+\.?\d*[GM])\s+used.*?(\d+\.?\d*[GM])\s+unused/);
  const memory = memMatch
    ? { used: memMatch[1], free: memMatch[2], raw: memRaw }
    : { used: '?', free: '?', raw: memRaw };

  // Parse Docker containers
  const containers = dockerRaw
    ? dockerRaw.split('\n').filter(Boolean).map((line) => {
        const [name, status, ports] = line.split('|');
        const healthy = status?.includes('healthy') && !status?.includes('unhealthy');
        const unhealthy = status?.includes('unhealthy');
        return { name, status, ports: ports || '', healthy, unhealthy };
      })
    : [];

  // Parse key native processes
  const processes: { name: string; pid: string; mem: string; uptime: string }[] = [];
  if (psRaw) {
    for (const line of psRaw.split('\n').filter(Boolean)) {
      const parts = line.trim().split(/\s+/);
      const pid = parts[1];
      const memMb = ((parseFloat(parts[5]) || 0) / 1024).toFixed(0);
      const time = parts[9] || '';
      const cmd = parts.slice(10).join(' ');

      if (cmd.includes('argus telegram')) {
        processes.push({ name: 'telegram daemon', pid, mem: `${memMb}MB`, uptime: time });
      } else if (cmd.includes('argus web')) {
        processes.push({ name: 'argus web', pid, mem: `${memMb}MB`, uptime: time });
      } else if (cmd.includes('next dev')) {
        processes.push({ name: 'frontend (next)', pid, mem: `${memMb}MB`, uptime: time });
      }
    }
  }

  return Response.json({ memory, containers, processes, ts: Date.now() });
}
