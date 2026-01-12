'use client';

import { Play, Square, Clock, Bot } from 'lucide-react';

interface Task {
  id: string;
  title: string;
  description: string;
  status: string;
  agent_type: string;
  priority: number;
  labels: string[];
  created_at: string;
}

interface TaskCardProps {
  task: Task;
  onExecute: () => void;
  onStop: () => void;
}

export function TaskCard({ task, onExecute, onStop }: TaskCardProps) {
  const priorityColors: Record<number, string> = {
    1: 'bg-red-500',
    2: 'bg-orange-500',
    3: 'bg-yellow-500',
    4: 'bg-blue-500',
    5: 'bg-gray-400',
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="kanban-card">
      {/* Priority indicator */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          <div
            className={`w-2 h-2 rounded-full ${
              priorityColors[task.priority] || priorityColors[5]
            }`}
          />
          <span className="text-xs text-gray-500 font-mono">
            {task.id.slice(0, 8)}
          </span>
        </div>
        <StatusBadge status={task.status} />
      </div>

      {/* Title */}
      <h4 className="font-medium text-gray-900 mb-2">{task.title}</h4>

      {/* Description */}
      {task.description && (
        <p className="text-sm text-gray-500 mb-3 line-clamp-2">
          {task.description}
        </p>
      )}

      {/* Agent type */}
      {task.agent_type && (
        <div className="flex items-center space-x-1 text-xs text-gray-500 mb-3">
          <Bot className="w-3 h-3" />
          <span>{task.agent_type}</span>
        </div>
      )}

      {/* Labels */}
      {task.labels && task.labels.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-3">
          {task.labels.map((label) => (
            <span
              key={label}
              className="px-2 py-0.5 bg-gray-100 text-gray-600 text-xs rounded"
            >
              {label}
            </span>
          ))}
        </div>
      )}

      {/* Footer */}
      <div className="flex items-center justify-between pt-2 border-t border-gray-100">
        <div className="flex items-center space-x-1 text-xs text-gray-400">
          <Clock className="w-3 h-3" />
          <span>{formatDate(task.created_at)}</span>
        </div>

        {/* Actions */}
        <div className="flex space-x-1">
          {task.status === 'pending' && (
            <button
              onClick={onExecute}
              className="p-1 text-green-600 hover:bg-green-50 rounded"
              title="Execute task"
            >
              <Play className="w-4 h-4" />
            </button>
          )}
          {task.status === 'running' && (
            <button
              onClick={onStop}
              className="p-1 text-red-600 hover:bg-red-50 rounded"
              title="Stop task"
            >
              <Square className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const statusStyles: Record<string, string> = {
    pending: 'status-pending',
    assigned: 'status-pending',
    running: 'status-running',
    completed: 'status-completed',
    failed: 'status-failed',
    cancelled: 'status-cancelled',
  };

  return (
    <span
      className={`px-2 py-0.5 text-xs font-medium rounded ${
        statusStyles[status] || statusStyles.pending
      }`}
    >
      {status}
    </span>
  );
}
