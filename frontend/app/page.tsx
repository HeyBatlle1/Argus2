'use client';

import { Header } from '@/components/header/Header';
import { EyesPanel } from '@/components/eyes/EyesPanel';
import { ConversationPanel } from '@/components/conversation/ConversationPanel';
import { MindPanel } from '@/components/mind/MindPanel';

export default function Home() {
  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ background: '#0a0a0f' }}>
      <Header />
      <main className="flex flex-1 overflow-hidden" style={{ paddingTop: '56px' }}>
        <EyesPanel />
        <ConversationPanel />
        <MindPanel />
      </main>
    </div>
  );
}
