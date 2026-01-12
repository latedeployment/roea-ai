'use client';

import { useState } from 'react';
import { KanbanBoard } from '@/components/kanban/KanbanBoard';
import { AgentList } from '@/components/agents/AgentList';
import { ExecutionMonitor } from '@/components/monitor/ExecutionMonitor';
import { ProcessGraph } from '@/components/process/ProcessGraph';
import { Layout, Bot, Activity, Settings, GitBranch } from 'lucide-react';

type Tab = 'kanban' | 'agents' | 'monitor' | 'processes' | 'settings';

export default function Home() {
  const [activeTab, setActiveTab] = useState<Tab>('kanban');

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="w-10 h-10 bg-primary-600 rounded-lg flex items-center justify-center">
                <Bot className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-gray-900">Roea AI</h1>
                <p className="text-sm text-gray-500">Agent Orchestrator</p>
              </div>
            </div>

            {/* Navigation */}
            <nav className="flex space-x-1">
              <TabButton
                icon={<Layout className="w-4 h-4" />}
                label="Tasks"
                active={activeTab === 'kanban'}
                onClick={() => setActiveTab('kanban')}
              />
              <TabButton
                icon={<Bot className="w-4 h-4" />}
                label="Agents"
                active={activeTab === 'agents'}
                onClick={() => setActiveTab('agents')}
              />
              <TabButton
                icon={<Activity className="w-4 h-4" />}
                label="Monitor"
                active={activeTab === 'monitor'}
                onClick={() => setActiveTab('monitor')}
              />
              <TabButton
                icon={<GitBranch className="w-4 h-4" />}
                label="Processes"
                active={activeTab === 'processes'}
                onClick={() => setActiveTab('processes')}
              />
              <TabButton
                icon={<Settings className="w-4 h-4" />}
                label="Settings"
                active={activeTab === 'settings'}
                onClick={() => setActiveTab('settings')}
              />
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-6">
        {activeTab === 'kanban' && <KanbanBoard />}
        {activeTab === 'agents' && <AgentList />}
        {activeTab === 'monitor' && <ExecutionMonitor />}
        {activeTab === 'processes' && <ProcessGraph />}
        {activeTab === 'settings' && <SettingsPanel />}
      </main>
    </div>
  );
}

function TabButton({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${
        active
          ? 'bg-primary-100 text-primary-700'
          : 'text-gray-600 hover:bg-gray-100'
      }`}
    >
      {icon}
      <span className="font-medium">{label}</span>
    </button>
  );
}

function SettingsPanel() {
  return (
    <div className="bg-white rounded-lg shadow-sm p-6">
      <h2 className="text-lg font-semibold mb-4">Settings</h2>
      <p className="text-gray-500">Settings panel coming soon...</p>
    </div>
  );
}
