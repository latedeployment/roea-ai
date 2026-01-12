'use client';

import { useState } from 'react';
import useSWR, { mutate } from 'swr';
import { Plus, Play, Square, RefreshCw } from 'lucide-react';
import { TaskCard } from './TaskCard';
import { CreateTaskModal } from './CreateTaskModal';

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

const fetcher = (url: string) => fetch(url).then((res) => res.json());

const columns = [
  { id: 'pending', title: 'Pending', status: 'pending' },
  { id: 'running', title: 'Running', status: 'running' },
  { id: 'completed', title: 'Completed', status: 'completed' },
  { id: 'failed', title: 'Failed', status: 'failed' },
];

export function KanbanBoard() {
  const { data: tasks, error, isLoading } = useSWR<Task[]>('/api/v1/tasks', fetcher, {
    refreshInterval: 5000,
  });
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  const refreshTasks = () => {
    mutate('/api/v1/tasks');
  };

  const executeTask = async (taskId: string) => {
    try {
      await fetch(`/api/v1/tasks/${taskId}/execute?async=true`, {
        method: 'POST',
      });
      refreshTasks();
    } catch (err) {
      console.error('Failed to execute task:', err);
    }
  };

  const stopTask = async (taskId: string) => {
    try {
      await fetch(`/api/v1/tasks/${taskId}/stop`, {
        method: 'POST',
      });
      refreshTasks();
    } catch (err) {
      console.error('Failed to stop task:', err);
    }
  };

  const getTasksByStatus = (status: string) => {
    if (!tasks) return [];
    return tasks.filter((task) => task.status === status);
  };

  if (error) {
    return (
      <div className="bg-red-50 text-red-700 p-4 rounded-lg">
        Failed to load tasks. Make sure the Roea server is running.
      </div>
    );
  }

  return (
    <div>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-4">
          <h2 className="text-lg font-semibold">Task Board</h2>
          <button
            onClick={refreshTasks}
            className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </button>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center space-x-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          <span>New Task</span>
        </button>
      </div>

      {/* Kanban Columns */}
      <div className="flex space-x-4 overflow-x-auto pb-4">
        {columns.map((column) => (
          <div key={column.id} className="kanban-column flex-shrink-0">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold text-gray-700">{column.title}</h3>
              <span className="text-sm text-gray-500">
                {getTasksByStatus(column.status).length}
              </span>
            </div>

            <div className="space-y-3">
              {getTasksByStatus(column.status).map((task) => (
                <TaskCard
                  key={task.id}
                  task={task}
                  onExecute={() => executeTask(task.id)}
                  onStop={() => stopTask(task.id)}
                />
              ))}

              {getTasksByStatus(column.status).length === 0 && (
                <div className="text-center text-gray-400 py-8">
                  No tasks
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Create Task Modal */}
      {isCreateModalOpen && (
        <CreateTaskModal
          onClose={() => setIsCreateModalOpen(false)}
          onCreated={() => {
            setIsCreateModalOpen(false);
            refreshTasks();
          }}
        />
      )}
    </div>
  );
}
