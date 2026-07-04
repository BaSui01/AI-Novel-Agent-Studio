import React from 'react';
import { useNovelStore } from '../stores/useNovelStore';
import { Activity, Cpu, DollarSign, Gauge, ShieldCheck, Zap } from 'lucide-react';

export const TelemetryBar: React.FC = () => {
  const { totalCostUsd, totalTokens, lastTps, lastTtftMs, isGenerating } = useNovelStore();

  return (
    <div className="h-7 bg-slate-950 border-t border-slate-800 px-4 flex items-center justify-between text-[11px] text-slate-400">
      {/* Left Gateway Status */}
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-1.5">
          <div className={`w-2 h-2 rounded-full ${isGenerating ? 'bg-amber-400 animate-ping' : 'bg-emerald-400'}`} />
          <span className="text-slate-300 font-medium">Rust API Gateway: 监听中 (localhost:8080)</span>
        </div>
        <div className="flex items-center space-x-1 text-slate-500">
          <ShieldCheck className="w-3.5 h-3.5 text-emerald-500" />
          <span>本地 SQLite + LanceDB 数据已加密保护</span>
        </div>
      </div>

      {/* Right Metrics Telemetry */}
      <div className="flex items-center space-x-5">
        <div className="flex items-center space-x-1 text-slate-300">
          <Gauge className="w-3.5 h-3.5 text-indigo-400" />
          <span>实时 TPS: <strong className="text-indigo-300">{lastTps > 0 ? lastTps.toFixed(1) : '-'} token/s</strong></span>
        </div>

        <div className="flex items-center space-x-1 text-slate-300">
          <Activity className="w-3.5 h-3.5 text-purple-400" />
          <span>TTFT: <strong className="text-purple-300">{lastTtftMs > 0 ? `${lastTtftMs}ms` : '-'}</strong></span>
        </div>

        <div className="flex items-center space-x-1 text-slate-300">
          <Zap className="w-3.5 h-3.5 text-amber-400" />
          <span>累计 Token: <strong className="text-amber-300">{totalTokens.toLocaleString()}</strong></span>
        </div>

        <div className="flex items-center space-x-1 text-slate-300">
          <DollarSign className="w-3.5 h-3.5 text-emerald-400" />
          <span>累计花费: <strong className="text-emerald-300">${totalCostUsd.toFixed(4)}</strong></span>
        </div>
      </div>
    </div>
  );
};
