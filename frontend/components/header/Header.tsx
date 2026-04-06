'use client';

import { ArgusEye } from './ArgusEye';
import { ConnectionStatus } from './ConnectionStatus';
import { ModelSelector } from './ModelSelector';
import { SentryDropdown } from './SentryDropdown';

export function Header() {
  return (
    <header
      className="fixed top-0 left-0 right-0 z-30 h-14 flex items-center justify-between px-4 border-b border-argus-border"
      style={{ background: '#0d0d14' }}
    >
      {/* Left: Logo */}
      <div className="flex items-center gap-3">
        <ArgusEye />
        <div className="flex flex-col">
          <span className="font-mono text-sm font-bold tracking-[0.2em] uppercase text-argus-amber leading-none">
            ARGUS
          </span>
          <span className="font-mono text-[9px] tracking-widest uppercase text-argus-textDim leading-none mt-0.5">
            The Hundred-Eyed Agent
          </span>
        </div>
      </div>

      {/* Center: Connection status */}
      <ConnectionStatus />

      {/* Right: Sentry + Model selector */}
      <div className="flex items-center gap-2">
        <SentryDropdown />
        <ModelSelector />
      </div>
    </header>
  );
}
