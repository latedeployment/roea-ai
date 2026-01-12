'use client';

import useSWR from 'swr';
import { Bot, Settings, Play, Users } from 'lucide-react';

interface AgentDefinition {
  id: string;
  name: string;
  description: string;
  base_runtime: string;
  default_model: string;
  resource_limits?: {
    max_turns: number;
    timeout_minutes: number;
    max_cost_usd: number;
  };
}

interface AgentInstance {
  id: string;
  agent_type: string;
  task_id: string;
  status: string;
  started_at: string;
}

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export function AgentList() {
  const { data: agents, error: agentsError } = useSWR<AgentDefinition[]>(
    '/api/v1/agents',
    fetcher
  );
  const { data: instances } = useSWR<AgentInstance[]>(
    '/api/v1/agents/instances',
    fetcher,
    { refreshInterval: 3000 }
  );

  if (agentsError) {
    return (
      <div className="bg-red-50 text-red-700 p-4 rounded-lg">
        Failed to load agents. Make sure the Roea server is running.
      </div>
    );
  }

  const getInstanceCount = (agentId: string) => {
    if (!instances) return 0;
    return instances.filter((i) => i.agent_type === agentId).length;
  };

  return (
    <div className="space-y-6">
      {/* Running Instances */}
      {instances && instances.length > 0 && (
        <div className="bg-white rounded-lg shadow-sm p-6">
          <h2 className="text-lg font-semibold mb-4 flex items-center space-x-2">
            <Users className="w-5 h-5 text-green-500" />
            <span>Running Instances ({instances.length})</span>
          </h2>
          <div className="space-y-3">
            {instances.map((instance) => (
              <div
                key={instance.id}
                className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
              >
                <div className="flex items-center space-x-3">
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                  <div>
                    <span className="font-medium">{instance.agent_type}</span>
                    <span className="text-gray-500 text-sm ml-2">
                      Task: {instance.task_id.slice(0, 8)}
                    </span>
                  </div>
                </div>
                <span className="text-xs text-gray-400 font-mono">
                  {instance.id}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Agent Definitions */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h2 className="text-lg font-semibold mb-4">Agent Definitions</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {agents?.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              instanceCount={getInstanceCount(agent.id)}
            />
          ))}

          {!agents && (
            <div className="col-span-full text-center py-8 text-gray-500">
              Loading agents...
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function AgentCard({
  agent,
  instanceCount,
}: {
  agent: AgentDefinition;
  instanceCount: number;
}) {
  return (
    <div className="border border-gray-200 rounded-lg p-4 hover:border-primary-300 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 bg-primary-100 rounded-lg flex items-center justify-center">
            <Bot className="w-4 h-4 text-primary-600" />
          </div>
          <div>
            <h3 className="font-medium text-gray-900">{agent.name}</h3>
            <span className="text-xs text-gray-500 font-mono">{agent.id}</span>
          </div>
        </div>
        {instanceCount > 0 && (
          <span className="px-2 py-0.5 bg-green-100 text-green-700 text-xs rounded-full">
            {instanceCount} running
          </span>
        )}
      </div>

      <p className="text-sm text-gray-600 mb-3 line-clamp-2">
        {agent.description}
      </p>

      <div className="space-y-1 text-xs text-gray-500">
        <div className="flex items-center justify-between">
          <span>Runtime:</span>
          <span className="font-medium">{agent.base_runtime}</span>
        </div>
        <div className="flex items-center justify-between">
          <span>Model:</span>
          <span className="font-medium">{agent.default_model}</span>
        </div>
        {agent.resource_limits && (
          <>
            <div className="flex items-center justify-between">
              <span>Max turns:</span>
              <span className="font-medium">{agent.resource_limits.max_turns}</span>
            </div>
            <div className="flex items-center justify-between">
              <span>Timeout:</span>
              <span className="font-medium">
                {agent.resource_limits.timeout_minutes} min
              </span>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
