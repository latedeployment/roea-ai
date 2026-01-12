'use client';

import { useEffect, useState } from 'react';
import useSWR from 'swr';
import { Activity, CheckCircle, XCircle, Clock, TrendingUp } from 'lucide-react';

interface TaskStats {
  pending: number;
  running: number;
  completed: number;
  failed: number;
  total: number;
}

interface AgentInstance {
  id: string;
  agent_type: string;
  task_id: string;
  status: string;
  started_at: string;
}

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export function ExecutionMonitor() {
  const { data: stats } = useSWR<TaskStats>('/api/v1/tasks/stats', fetcher, {
    refreshInterval: 3000,
  });
  const { data: instances } = useSWR<AgentInstance[]>(
    '/api/v1/agents/instances',
    fetcher,
    { refreshInterval: 2000 }
  );

  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Pending"
          value={stats?.pending || 0}
          icon={<Clock className="w-5 h-5" />}
          color="yellow"
        />
        <StatCard
          title="Running"
          value={stats?.running || 0}
          icon={<Activity className="w-5 h-5" />}
          color="blue"
        />
        <StatCard
          title="Completed"
          value={stats?.completed || 0}
          icon={<CheckCircle className="w-5 h-5" />}
          color="green"
        />
        <StatCard
          title="Failed"
          value={stats?.failed || 0}
          icon={<XCircle className="w-5 h-5" />}
          color="red"
        />
      </div>

      {/* Active Executions */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center space-x-2">
          <Activity className="w-5 h-5 text-blue-500" />
          <span>Active Executions</span>
        </h2>

        {instances && instances.length > 0 ? (
          <div className="space-y-4">
            {instances.map((instance) => (
              <ExecutionRow key={instance.id} instance={instance} />
            ))}
          </div>
        ) : (
          <div className="text-center py-12 text-gray-500">
            <Activity className="w-12 h-12 mx-auto mb-3 text-gray-300" />
            <p>No active executions</p>
            <p className="text-sm">Start a task to see it here</p>
          </div>
        )}
      </div>

      {/* System Status */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center space-x-2">
          <TrendingUp className="w-5 h-5 text-green-500" />
          <span>System Status</span>
        </h2>
        <div className="grid gap-4 md:grid-cols-3">
          <StatusItem
            label="Total Tasks"
            value={stats?.total || 0}
            status="healthy"
          />
          <StatusItem
            label="Active Agents"
            value={instances?.length || 0}
            status="healthy"
          />
          <StatusItem
            label="Success Rate"
            value={
              stats && stats.total > 0
                ? `${Math.round(
                    (stats.completed / (stats.completed + stats.failed || 1)) *
                      100
                  )}%`
                : 'N/A'
            }
            status={
              stats && stats.failed > stats.completed ? 'warning' : 'healthy'
            }
          />
        </div>
      </div>
    </div>
  );
}

function StatCard({
  title,
  value,
  icon,
  color,
}: {
  title: string;
  value: number;
  icon: React.ReactNode;
  color: 'yellow' | 'blue' | 'green' | 'red';
}) {
  const colors = {
    yellow: 'bg-yellow-50 text-yellow-600',
    blue: 'bg-blue-50 text-blue-600',
    green: 'bg-green-50 text-green-600',
    red: 'bg-red-50 text-red-600',
  };

  return (
    <div className="bg-white rounded-lg shadow-sm p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-500">{title}</p>
          <p className="text-2xl font-bold mt-1">{value}</p>
        </div>
        <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${colors[color]}`}>
          {icon}
        </div>
      </div>
    </div>
  );
}

function ExecutionRow({ instance }: { instance: AgentInstance }) {
  const [elapsedTime, setElapsedTime] = useState('');

  useEffect(() => {
    const updateElapsed = () => {
      const start = new Date(instance.started_at);
      const now = new Date();
      const diff = Math.floor((now.getTime() - start.getTime()) / 1000);
      const mins = Math.floor(diff / 60);
      const secs = diff % 60;
      setElapsedTime(`${mins}:${secs.toString().padStart(2, '0')}`);
    };

    updateElapsed();
    const interval = setInterval(updateElapsed, 1000);
    return () => clearInterval(interval);
  }, [instance.started_at]);

  return (
    <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
      <div className="flex items-center space-x-4">
        <div className="w-3 h-3 bg-blue-500 rounded-full animate-pulse" />
        <div>
          <div className="font-medium">{instance.agent_type}</div>
          <div className="text-sm text-gray-500">
            Task: {instance.task_id.slice(0, 8)}...
          </div>
        </div>
      </div>
      <div className="text-right">
        <div className="font-mono text-sm">{elapsedTime}</div>
        <div className="text-xs text-gray-400">{instance.status}</div>
      </div>
    </div>
  );
}

function StatusItem({
  label,
  value,
  status,
}: {
  label: string;
  value: number | string;
  status: 'healthy' | 'warning' | 'error';
}) {
  const statusColors = {
    healthy: 'bg-green-500',
    warning: 'bg-yellow-500',
    error: 'bg-red-500',
  };

  return (
    <div className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
      <div className="flex items-center space-x-2">
        <div className={`w-2 h-2 rounded-full ${statusColors[status]}`} />
        <span className="text-gray-600">{label}</span>
      </div>
      <span className="font-medium">{value}</span>
    </div>
  );
}
