'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { ScheduledTask, ModelId } from '@/lib/types';
import { MODEL_CONFIG } from '@/lib/models';

function groupByPeriod(tasks: ScheduledTask[]) {
  const now = new Date();
  const startOfWeek = new Date(now);
  startOfWeek.setDate(now.getDate() - now.getDay());
  startOfWeek.setHours(0, 0, 0, 0);

  const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
  const startOfLastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);

  const thisWeek: ScheduledTask[] = [];
  const thisMonth: ScheduledTask[] = [];
  const lastMonth: ScheduledTask[] = [];
  const older: ScheduledTask[] = [];
  const upcoming: ScheduledTask[] = [];

  for (const task of tasks) {
    const ts = task.runAt ? new Date(task.runAt) : new Date(task.createdAt);
    const isUpcoming = task.status === 'pending' && task.runAt && new Date(task.runAt) > now;

    if (isUpcoming) {
      upcoming.push(task);
    } else if (ts >= startOfWeek) {
      thisWeek.push(task);
    } else if (ts >= startOfMonth) {
      thisMonth.push(task);
    } else if (ts >= startOfLastMonth) {
      lastMonth.push(task);
    } else {
      older.push(task);
    }
  }

  return { upcoming, thisWeek, thisMonth, lastMonth, older };
}

const STATUS_COLOR: Record<string, string> = {
  pending:  '#f5b800',
  running:  '#67f6ff',
  done:     '#39d353',
  failed:   '#ff5577',
};

const STATUS_LABEL: Record<string, string> = {
  pending:  '○',
  running:  '◉',
  done:     '✓',
  failed:   '✗',
};

function TaskRow({ task }: { task: ScheduledTask }) {
  const cfg = MODEL_CONFIG[task.agent as ModelId] ?? { name: task.agent, color: '#9a9aa8' };
  const ts = task.runAt ? new Date(task.runAt) : new Date(task.createdAt);
  const statusCol = STATUS_COLOR[task.status] ?? '#5a5a68';

  return (
    <div className="flex gap-2 items-start py-1.5 border-b border-white/5 last:border-0">
      <span className="flex-shrink-0 font-mono text-[10px] mt-px" style={{ color: statusCol }}>
        {STATUS_LABEL[task.status] ?? '·'}
      </span>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5">
          <span
            className="text-[8px] font-mono px-1 py-px rounded"
            style={{ background: `${cfg.color}18`, color: cfg.color, border: `1px solid ${cfg.color}30` }}
          >
            {cfg.name}
          </span>
          <span className="text-[9px] font-mono text-[#3a3a48]">
            {ts.toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
          </span>
        </div>
        <p className="text-[10px] text-[#9a9aa8] leading-tight line-clamp-2">
          {task.description}
        </p>
      </div>
    </div>
  );
}

function Section({ label, tasks }: { label: string; tasks: ScheduledTask[] }) {
  if (tasks.length === 0) return null;
  return (
    <div className="mb-1">
      <div
        className="flex items-center gap-2 px-3 py-1.5 sticky top-0"
        style={{ background: '#070710' }}
      >
        <div className="text-[8px] font-mono tracking-[2px] uppercase text-[#c084fc]/60">{label}</div>
        <div className="flex-1 h-px" style={{ background: 'rgba(192,132,252,0.12)' }} />
        <span className="text-[8px] font-mono text-[#3a3a48]">{tasks.length}</span>
      </div>
      <div className="px-3">
        {tasks.map((t) => <TaskRow key={t.id} task={t} />)}
      </div>
    </div>
  );
}

// Model activity summary — how many tasks per model per period
function ModelSummary({ tasks, period }: { tasks: ScheduledTask[]; period: string }) {
  if (tasks.length === 0) return null;

  const counts: Record<string, number> = {};
  for (const t of tasks) {
    counts[t.agent] = (counts[t.agent] ?? 0) + 1;
  }

  return (
    <div className="mx-3 mb-3 p-2 rounded-lg" style={{ background: 'rgba(192,132,252,0.04)', border: '1px solid rgba(192,132,252,0.1)' }}>
      <div className="text-[8px] font-mono tracking-[1.5px] text-[#c084fc]/50 mb-1.5 uppercase">{period} summary</div>
      <div className="flex flex-wrap gap-1.5">
        {Object.entries(counts).map(([model, count]) => {
          const cfg = MODEL_CONFIG[model as ModelId] ?? { name: model, color: '#9a9aa8' };
          return (
            <div key={model} className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full flex-shrink-0" style={{ background: cfg.color }} />
              <span className="text-[9px] font-mono text-[#5a5a68]">{cfg.name}</span>
              <span className="text-[9px] font-mono text-[#9a9aa8]">×{count}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function ScheduleChronicle() {
  const scheduledTasks = useAgentStore((s) => s.scheduledTasks);

  if (scheduledTasks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center px-4">
        <div className="text-2xl mb-2 opacity-20">◎</div>
        <div className="font-mono text-[10px] text-[#3a3a48] tracking-[1px]">
          NO SCHEDULED TASKS
        </div>
        <div className="text-[9px] text-[#3a3a48] mt-1">
          deploy work via the TASKS button
        </div>
      </div>
    );
  }

  const { upcoming, thisWeek, thisMonth, lastMonth, older } = groupByPeriod(scheduledTasks);
  const allThisMonth = [...thisWeek, ...thisMonth];

  return (
    <div className="h-full overflow-y-auto">
      <div className="pt-2">
        <ModelSummary tasks={allThisMonth} period="this month" />
        <Section label="Upcoming" tasks={upcoming} />
        <Section label="This Week" tasks={thisWeek} />
        <Section label="This Month" tasks={thisMonth} />
        <Section label="Last Month" tasks={lastMonth} />
        <Section label="Older" tasks={older} />
      </div>
    </div>
  );
}
