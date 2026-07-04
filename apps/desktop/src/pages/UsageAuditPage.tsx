import React, { useState } from 'react';
import { Activity, DollarSign, Zap, Gauge, History, Filter, RefreshCw } from 'lucide-react';
import { useNovelStore } from '../stores/useNovelStore';

export const UsageAuditPage: React.FC = () => {
  const { totalCostUsd, totalTokens } = useNovelStore();

  const [logs] = useState([
    {
      id: 'req_101',
      agent: 'Writer',
      provider: 'Anthropic Claude',
      model: 'claude-3-5-sonnet-20241022',
      inputTokens: 1420,
      outputTokens: 1850,
      reasoningTokens: 0,
      costUsd: 0.0320,
      latencyMs: 1420,
      ttftMs: 280,
      tps: 62.4,
      createdAt: '2026-07-03 23:25:10',
    },
    {
      id: 'req_102',
      agent: 'Editor',
      provider: 'OpenAI Cloud',
      model: 'gpt-4o',
      inputTokens: 850,
      outputTokens: 620,
      reasoningTokens: 0,
      costUsd: 0.0083,
      latencyMs: 890,
      ttftMs: 220,
      tps: 58.1,
      createdAt: '2026-07-03 23:20:45',
    },
    {
      id: 'req_103',
      agent: 'Reviewer',
      provider: 'OpenAI Cloud',
      model: 'o3-mini',
      inputTokens: 2100,
      outputTokens: 980,
      reasoningTokens: 420,
      costUsd: 0.0066,
      latencyMs: 1100,
      ttftMs: 310,
      tps: 45.2,
      createdAt: '2026-07-03 23:15:30',
    },
    {
      id: 'req_104',
      agent: 'Summarizer',
      provider: 'Ollama Local',
      model: 'qwen2.5:32b',
      inputTokens: 3200,
      outputTokens: 450,
      reasoningTokens: 0,
      costUsd: 0.0,
      latencyMs: 650,
      ttftMs: 140,
      tps: 84.2,
      createdAt: '2026-07-03 23:10:00',
    },
  ]);

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Overview Cards */}
      <div className="grid grid-cols-4 gap-4">
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <div className="text-xs text-slate-400 flex items-center space-x-1">
            <DollarSign className="w-3.5 h-3.5 text-emerald-400" />
            <span>项目总美金消费</span>
          </div>
          <div className="text-xl font-bold text-emerald-400 font-mono">${totalCostUsd.toFixed(4)}</div>
        </div>

        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <div className="text-xs text-slate-400 flex items-center space-x-1">
            <Zap className="w-3.5 h-3.5 text-amber-400" />
            <span>累计 Token 消耗</span>
          </div>
          <div className="text-xl font-bold text-amber-400 font-mono">{totalTokens.toLocaleString()}</div>
        </div>

        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <div className="text-xs text-slate-400 flex items-center space-x-1">
            <Gauge className="w-3.5 h-3.5 text-indigo-400" />
            <span>平均生成速度 (TPS)</span>
          </div>
          <div className="text-xl font-bold text-indigo-400 font-mono">62.4 token/s</div>
        </div>

        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <div className="text-xs text-slate-400 flex items-center space-x-1">
            <Activity className="w-3.5 h-3.5 text-purple-400" />
            <span>平均首 Token 延迟 (TTFT)</span>
          </div>
          <div className="text-xl font-bold text-purple-400 font-mono">238 ms</div>
        </div>
      </div>

      {/* Audit Logs Table Header */}
      <div className="flex items-center justify-between">
        <h3 className="font-bold text-sm text-slate-100 flex items-center space-x-2">
          <History className="w-4 h-4 text-indigo-400" />
          <span>LLM API 请求审计与消费日志 (Request Logs Audit)</span>
        </h3>
        <button className="p-1.5 rounded bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200">
          <Filter className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Audit Logs Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900 overflow-hidden">
        <table className="w-full text-left border-collapse text-xs">
          <thead>
            <tr className="bg-slate-950 border-b border-slate-800 text-slate-400">
              <th className="p-3 font-semibold">请求 ID & 时间</th>
              <th className="p-3 font-semibold">Agent 角色</th>
              <th className="p-3 font-semibold">调度 Provider & 模型</th>
              <th className="p-3 font-semibold">Input / Output Tokens</th>
              <th className="p-3 font-semibold">Reasoning Tokens</th>
              <th className="p-3 font-semibold">延迟 / TTFT</th>
              <th className="p-3 font-semibold">生成速度 (TPS)</th>
              <th className="p-3 font-semibold text-right">消费金额</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60">
            {logs.map((log) => (
              <tr key={log.id} className="hover:bg-slate-800/40 transition-colors">
                <td className="p-3">
                  <span className="font-mono font-medium text-slate-200">{log.id}</span>
                  <span className="block text-[10px] text-slate-500">{log.createdAt}</span>
                </td>
                <td className="p-3">
                  <span className="px-2 py-0.5 rounded bg-purple-950/60 border border-purple-800/40 text-purple-300 text-[10px]">
                    {log.agent} Agent
                  </span>
                </td>
                <td className="p-3">
                  <span className="font-medium text-slate-200">{log.model}</span>
                  <span className="block text-[10px] text-slate-500">{log.provider}</span>
                </td>
                <td className="p-3 font-mono text-slate-300">
                  {log.inputTokens} / {log.outputTokens}
                </td>
                <td className="p-3 font-mono text-slate-400">
                  {log.reasoningTokens > 0 ? `${log.reasoningTokens} tokens` : '-'}
                </td>
                <td className="p-3 font-mono text-slate-300">
                  {log.latencyMs}ms <span className="text-[10px] text-slate-500">({log.ttftMs}ms)</span>
                </td>
                <td className="p-3 font-mono text-indigo-400 font-semibold">{log.tps} t/s</td>
                <td className="p-3 text-right font-mono font-bold text-emerald-400">
                  ${log.costUsd.toFixed(4)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
