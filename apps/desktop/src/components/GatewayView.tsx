import React, { useState, useEffect } from 'react';
import { Cpu, Key, Server, Save, CheckCircle2, AlertCircle, Eye, EyeOff, Plus, Trash2, Shield, RefreshCw } from 'lucide-react';

interface ProviderItem {
  name: string;
  display_name: string;
  base_url: string;
  chat_completions_url?: string;
  responses_url?: string;
  openai_compat_url?: string;
  api_key: string;
  is_enabled: boolean;
  models: string[];
}

interface GatewayConfig {
  active_provider: string;
  fallback_chain: string[];
  providers: ProviderItem[];
}

export const GatewayView: React.FC = () => {
  const [config, setConfig] = useState<GatewayConfig | null>(null);
  const [showKeys, setShowKeys] = useState<{ [key: string]: boolean }>({});
  const [isSaved, setIsSaved] = useState(false);
  const [newModelInputs, setNewModelInputs] = useState<{ [key: string]: string }>({});

  useEffect(() => {
    fetchConfig();
  }, []);

  const fetchConfig = async () => {
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/gateway/config');
      if (res.ok) {
        const data = await res.json();
        setConfig(data);
      }
    } catch (err) {
      console.error('Failed to load gateway config:', err);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/gateway/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      if (res.ok) {
        setIsSaved(true);
        setTimeout(() => setIsSaved(false), 2500);
      }
    } catch (err) {
      console.error('Failed to save config:', err);
    }
  };

  const updateProvider = (index: number, field: keyof ProviderItem, value: any) => {
    if (!config) return;
    const updated = { ...config };
    updated.providers[index] = { ...updated.providers[index], [field]: value };
    setConfig(updated);
  };

  const toggleShowKey = (name: string) => {
    setShowKeys((prev) => ({ ...prev, [name]: !prev[name] }));
  };

  const addModel = (provIndex: number, provName: string) => {
    const modelName = newModelInputs[provName]?.trim();
    if (!modelName || !config) return;

    const updated = { ...config };
    if (!updated.providers[provIndex].models.includes(modelName)) {
      updated.providers[provIndex].models.push(modelName);
      setConfig(updated);
    }
    setNewModelInputs((prev) => ({ ...prev, [provName]: '' }));
  };

  const removeModel = (provIndex: number, modelIndex: number) => {
    if (!config) return;
    const updated = { ...config };
    updated.providers[provIndex].models.splice(modelIndex, 1);
    setConfig(updated);
  };

  if (!config) {
    return (
      <div className="flex-1 bg-slate-950 p-8 flex items-center justify-center text-slate-500">
        <RefreshCw className="w-5 h-5 animate-spin mr-2" />
        正在连接 Rust 原生 AI 网关服务...
      </div>
    );
  }

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Banner & Global Save Bar */}
      <div className="p-5 rounded-xl bg-gradient-to-r from-indigo-950/80 to-purple-950/80 border border-indigo-800/40 flex items-center justify-between shadow-lg">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center space-x-2">
            <Cpu className="w-5 h-5 text-indigo-400" />
            <span>AI 网关 4 大 Provider 端点卡片配置 (OpenAI/Claude/Gemini/Ollama)</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            原生支持 OpenAI (<code className="text-indigo-300">/v1/chat/completions</code> 与 <code className="text-indigo-300">/v1/responses</code> 双端点)，自动同步写入 <code className="bg-slate-900 px-1.5 py-0.5 rounded text-indigo-300">config/providers.json</code>
          </p>
        </div>
        <button
          onClick={handleSaveConfig}
          className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center space-x-1.5 transition-all shadow-lg shadow-indigo-900/40"
        >
          {isSaved ? (
            <>
              <CheckCircle2 className="w-4 h-4 text-emerald-300" />
              <span>已保存并生效！</span>
            </>
          ) : (
            <>
              <Save className="w-4 h-4" />
              <span>保存卡片配置</span>
            </>
          )}
        </button>
      </div>

      {/* 4 Core Provider Cards Grid */}
      <div className="grid grid-cols-2 gap-6">
        {config.providers.map((prov, idx) => (
          <div
            key={prov.name}
            className={`p-5 rounded-xl border transition-all space-y-4 ${
              prov.is_enabled
                ? 'bg-slate-900 border-slate-800 shadow-md'
                : 'bg-slate-950/50 border-slate-900 opacity-60'
            }`}
          >
            {/* Card Header & Switch */}
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <div className="flex items-center space-x-2">
                <Server className="w-4 h-4 text-indigo-400" />
                <h3 className="font-bold text-sm text-slate-100">{prov.display_name}</h3>
              </div>
              <label className="flex items-center space-x-2 cursor-pointer">
                <span className="text-xs text-slate-400">{prov.is_enabled ? '已启用' : '已禁用'}</span>
                <input
                  type="checkbox"
                  checked={prov.is_enabled}
                  onChange={(e) => updateProvider(idx, 'is_enabled', e.target.checked)}
                  className="rounded bg-slate-950 border-slate-800 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </label>
            </div>

            {/* Base URL Input */}
            <div>
              <label className="block text-[11px] font-medium text-slate-400 mb-1">Base API URL</label>
              <input
                type="text"
                value={prov.base_url}
                onChange={(e) => updateProvider(idx, 'base_url', e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 px-3 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>

            {/* OpenAI Dual Endpoints fields if OpenAI */}
            {prov.chat_completions_url !== undefined && (
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">Chat Completions Endpoint (/v1/chat/completions)</label>
                <input
                  type="text"
                  value={prov.chat_completions_url}
                  onChange={(e) => updateProvider(idx, 'chat_completions_url', e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 px-3 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            )}
            {prov.responses_url !== undefined && (
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">Responses API Endpoint (/v1/responses - 2025/2026 Agent 推荐)</label>
                <input
                  type="text"
                  value={prov.responses_url}
                  onChange={(e) => updateProvider(idx, 'responses_url', e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 px-3 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            )}

            {/* OpenAI Compat URL if Gemini */}
            {prov.openai_compat_url !== undefined && (
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">OpenAI 兼容 REST Endpoint</label>
                <input
                  type="text"
                  value={prov.openai_compat_url}
                  onChange={(e) => updateProvider(idx, 'openai_compat_url', e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 px-3 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            )}

            {/* API Key Input */}
            <div>
              <label className="block text-[11px] font-medium text-slate-400 mb-1">API Key 密钥</label>
              <div className="relative">
                <input
                  type={showKeys[prov.name] ? 'text' : 'password'}
                  value={prov.api_key}
                  onChange={(e) => updateProvider(idx, 'api_key', e.target.value)}
                  placeholder={prov.name === 'ollama' ? '本地模型无需 API Key' : '输入密钥 sk-xxxx'}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 pl-3 pr-9 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
                <button
                  type="button"
                  onClick={() => toggleShowKey(prov.name)}
                  className="absolute right-2.5 top-2 text-slate-500 hover:text-slate-300"
                >
                  {showKeys[prov.name] ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                </button>
              </div>
            </div>

            {/* Supported Models List */}
            <div>
              <label className="block text-[11px] font-medium text-slate-400 mb-1">支持的模型列表</label>
              <div className="flex flex-wrap gap-1.5 mb-2">
                {prov.models.map((m, mIdx) => (
                  <span
                    key={m}
                    className="inline-flex items-center space-x-1 px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-[11px] font-mono text-indigo-300"
                  >
                    <span>{m}</span>
                    <button
                      onClick={() => removeModel(idx, mIdx)}
                      className="text-slate-500 hover:text-red-400"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </span>
                ))}
              </div>
              <div className="flex space-x-2">
                <input
                  type="text"
                  placeholder="添加模型名 (如 o1 / o3-mini)"
                  value={newModelInputs[prov.name] || ''}
                  onChange={(e) => setNewModelInputs((prev) => ({ ...prev, [prov.name]: e.target.value }))}
                  onKeyDown={(e) => e.key === 'Enter' && addModel(idx, prov.name)}
                  className="flex-1 bg-slate-950 border border-slate-800 rounded py-1 px-2.5 text-[11px] text-slate-200 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onClick={() => addModel(idx, prov.name)}
                  className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs flex items-center"
                >
                  <Plus className="w-3 h-3 mr-1" />
                  添加
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
