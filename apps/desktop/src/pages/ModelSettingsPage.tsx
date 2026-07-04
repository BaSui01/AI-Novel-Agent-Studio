import React, { useState, useEffect } from 'react';
import { Cpu, DollarSign, Plus, Check, Trash2, Eye, Shield, Layers, RefreshCw, Zap, Image as ImageIcon, Brain, Wrench, Save } from 'lucide-react';

interface ModelRegistryItem {
  id: string;
  provider: string;
  display_name: string;
  api_model_name: string;
  endpoint_type: string;
  context_window: number;
  max_output_tokens: number;
  supports_streaming: boolean;
  supports_tools: boolean;
  supports_vision: boolean;
  supports_reasoning: boolean;
  input_price_per_1m: number;
  output_price_per_1m: number;
  cached_input_price_per_1m: number;
  reasoning_price_per_1m: number;
  currency: string;
  enabled: boolean;
  is_custom: boolean;
}

export const ModelSettingsPage: React.FC = () => {
  const [models, setModels] = useState<ModelRegistryItem[]>([]);
  const [isAdding, setIsAdding] = useState(false);
  const [isSaved, setIsSaved] = useState(false);
  const [isFetchingUpstream, setIsFetchingUpstream] = useState(false);
  const [fetchBaseUrl, setFetchBaseUrl] = useState('https://api.openai.com/v1');
  const [fetchApiKey, setFetchApiKey] = useState('');
  const [upstreamStatus, setUpstreamStatus] = useState<string | null>(null);

  const [newModel, setNewModel] = useState<Partial<ModelRegistryItem>>({
    id: '',
    display_name: '',
    provider: 'custom',
    api_model_name: '',
    endpoint_type: 'chat_completions',
    context_window: 128000,
    max_output_tokens: 8192,
    supports_streaming: true,
    supports_tools: true,
    supports_vision: true,
    supports_reasoning: false,
    input_price_per_1m: 0.14,
    output_price_per_1m: 0.28,
    cached_input_price_per_1m: 0.014,
    reasoning_price_per_1m: 0.28,
  });

  useEffect(() => {
    fetchModels();
  }, []);

  const fetchModels = async () => {
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/models/registry');
      if (res.ok) {
        const data = await res.json();
        setModels(data);
      }
    } catch (err) {
      console.error('Failed to load model registry:', err);
    }
  };

  const saveModelRegistry = async (updatedModels: ModelRegistryItem[]) => {
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/models/registry', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updatedModels),
      });
      if (res.ok) {
        setIsSaved(true);
        setTimeout(() => setIsSaved(false), 2500);
      }
    } catch (err) {
      console.error('Failed to save model registry:', err);
    }
  };

  const handleFetchUpstream = async () => {
    setIsFetchingUpstream(true);
    setUpstreamStatus(null);
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/gateway/fetch-upstream-models', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base_url: fetchBaseUrl, api_key: fetchApiKey }),
      });
      const data = await res.json();
      if (data.status === 'ok' && data.models) {
        // Automatically add fetched models into registry!
        const newItems: ModelRegistryItem[] = data.models.map((mId: string) => ({
          id: mId,
          provider: 'upstream',
          display_name: mId,
          api_model_name: mId,
          endpoint_type: 'chat_completions',
          context_window: 128000,
          max_output_tokens: 8192,
          supports_streaming: true,
          supports_tools: true,
          supports_vision: false,
          supports_reasoning: false,
          input_price_per_1m: 0.5,
          output_price_per_1m: 1.5,
          cached_input_price_per_1m: 0.1,
          reasoning_price_per_1m: 1.5,
          currency: 'USD',
          enabled: true,
          is_custom: true,
        }));

        const combined = [...models];
        for (const item of newItems) {
          if (!combined.some((m) => m.id === item.id)) {
            combined.push(item);
          }
        }
        setModels(combined);
        await saveModelRegistry(combined);
        setUpstreamStatus(`成功从上游拉取并同步 ${data.models.length} 个模型至模型注册表！`);
      } else {
        setUpstreamStatus(data.message || '从上游拉取失败，请检查 Base URL 与 API Key');
      }
    } catch {
      setUpstreamStatus('网络错误或无法连接中转站 Base URL');
    } finally {
      setIsFetchingUpstream(false);
    }
  };

  const handleTestModel = async (modelId: string, supportsVision: boolean) => {
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/gateway/test-model', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model_id: modelId,
          base_url: fetchBaseUrl,
          api_key: fetchApiKey,
          test_vision: supportsVision,
        }),
      });
      const data = await res.json();
      alert(data.message || '模型连接正常');
    } catch {
      alert('连通性测试失败');
    }
  };

  const handleAddModel = () => {
    if (!newModel.id || !newModel.display_name) return;
    const created: ModelRegistryItem = {
      id: newModel.id,
      provider: newModel.provider || 'custom',
      display_name: newModel.display_name,
      api_model_name: newModel.api_model_name || newModel.id,
      endpoint_type: newModel.endpoint_type || 'chat_completions',
      context_window: newModel.context_window || 128000,
      max_output_tokens: newModel.max_output_tokens || 8192,
      supports_streaming: newModel.supports_streaming ?? true,
      supports_tools: newModel.supports_tools ?? true,
      supports_vision: newModel.supports_vision ?? false,
      supports_reasoning: newModel.supports_reasoning ?? false,
      input_price_per_1m: newModel.input_price_per_1m || 0,
      output_price_per_1m: newModel.output_price_per_1m || 0,
      cached_input_price_per_1m: newModel.cached_input_price_per_1m || 0,
      reasoning_price_per_1m: newModel.reasoning_price_per_1m || 0,
      currency: 'USD',
      enabled: true,
      is_custom: true,
    };

    const updated = [...models, created];
    setModels(updated);
    saveModelRegistry(updated);
    setIsAdding(false);
  };

  const handleDeleteModel = (id: string) => {
    const updated = models.filter((m) => m.id !== id);
    setModels(updated);
    saveModelRegistry(updated);
  };

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Header Banner */}
      <div className="p-5 rounded-xl bg-gradient-to-r from-purple-950/80 to-indigo-950/80 border border-purple-800/40 flex items-center justify-between shadow-lg">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center space-x-2">
            <DollarSign className="w-5 h-5 text-emerald-400" />
            <span>Model Registry 动态模型注册表 (完全无硬编码 / 后端同步)</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            动态读取与写入 <code className="bg-slate-900 px-1.5 py-0.5 rounded text-purple-300">config/models.json</code>，支持上游拉取、自定义参数与多模态识图设置。
          </p>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={() => saveModelRegistry(models)}
            className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold flex items-center space-x-1.5 transition-all shadow-lg"
          >
            {isSaved ? <Check className="w-4 h-4" /> : <Save className="w-4 h-4" />}
            <span>{isSaved ? '已保存！' : '保存模型配置'}</span>
          </button>
          <button
            onClick={() => setIsAdding(!isAdding)}
            className="px-4 py-2 rounded-lg bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold flex items-center space-x-1.5 transition-all shadow-lg shadow-purple-900/40"
          >
            <Plus className="w-4 h-4" />
            <span>手动添加自定义模型</span>
          </button>
        </div>
      </div>

      {/* Upstream Auto-Fetch Bar */}
      <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
        <h3 className="text-xs font-bold text-indigo-300 flex items-center space-x-1.5">
          <RefreshCw className="w-4 h-4" />
          <span>从上游 API 中转站 Base URL 自动拉取所有模型列表 (GET /v1/models)</span>
        </h3>
        <div className="flex space-x-3 text-xs">
          <input
            type="text"
            placeholder="中转站 Base URL (如 https://api.openai.com/v1)"
            value={fetchBaseUrl}
            onChange={(e) => setFetchBaseUrl(e.target.value)}
            className="flex-1 bg-slate-950 border border-slate-800 rounded px-3 py-2 text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
          />
          <input
            type="password"
            placeholder="API Key 密钥 (sk-xxxx)"
            value={fetchApiKey}
            onChange={(e) => setFetchApiKey(e.target.value)}
            className="w-64 bg-slate-950 border border-slate-800 rounded px-3 py-2 text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
          />
          <button
            onClick={handleFetchUpstream}
            disabled={isFetchingUpstream}
            className="px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-semibold flex items-center space-x-1"
          >
            {isFetchingUpstream ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Zap className="w-4 h-4" />}
            <span>自动拉取上游模型</span>
          </button>
        </div>
        {upstreamStatus && (
          <p className="text-xs font-mono text-emerald-400 bg-slate-950 p-2 rounded border border-slate-800">
            {upstreamStatus}
          </p>
        )}
      </div>

      {/* Manual Add Custom Model Modal Card */}
      {isAdding && (
        <div className="p-5 rounded-xl bg-slate-900 border border-purple-800/60 space-y-4 shadow-xl">
          <h3 className="text-sm font-bold text-purple-300">手动配置模型高级参数与能力标志</h3>
          <div className="grid grid-cols-4 gap-3 text-xs">
            <div>
              <label className="block text-slate-400 mb-1">模型 ID (如 deepseek-v3 / gpt-4o)</label>
              <input
                type="text"
                placeholder="deepseek-v3"
                value={newModel.id}
                onChange={(e) => setNewModel({ ...newModel, id: e.target.value })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">显示名称</label>
              <input
                type="text"
                placeholder="DeepSeek V3"
                value={newModel.display_name}
                onChange={(e) => setNewModel({ ...newModel, display_name: e.target.value })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">上下文窗口 (Context Tokens)</label>
              <input
                type="number"
                value={newModel.context_window}
                onChange={(e) => setNewModel({ ...newModel, context_window: parseInt(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">最大输出 Token (Max Output)</label>
              <input
                type="number"
                value={newModel.max_output_tokens}
                onChange={(e) => setNewModel({ ...newModel, max_output_tokens: parseInt(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">输入单价 ($/1M Tokens)</label>
              <input
                type="number"
                step="0.01"
                value={newModel.input_price_per_1m}
                onChange={(e) => setNewModel({ ...newModel, input_price_per_1m: parseFloat(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">输出单价 ($/1M Tokens)</label>
              <input
                type="number"
                step="0.01"
                value={newModel.output_price_per_1m}
                onChange={(e) => setNewModel({ ...newModel, output_price_per_1m: parseFloat(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">Cached 单价 ($/1M)</label>
              <input
                type="number"
                step="0.001"
                value={newModel.cached_input_price_per_1m}
                onChange={(e) => setNewModel({ ...newModel, cached_input_price_per_1m: parseFloat(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">Reasoning 单价 ($/1M)</label>
              <input
                type="number"
                step="0.01"
                value={newModel.reasoning_price_per_1m}
                onChange={(e) => setNewModel({ ...newModel, reasoning_price_per_1m: parseFloat(e.target.value) })}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-slate-200 font-mono"
              />
            </div>
          </div>

          {/* Capability Switches */}
          <div className="flex items-center space-x-6 text-xs border-t border-slate-800 pt-3">
            <label className="flex items-center space-x-1.5 cursor-pointer">
              <input
                type="checkbox"
                checked={newModel.supports_vision}
                onChange={(e) => setNewModel({ ...newModel, supports_vision: e.target.checked })}
                className="rounded bg-slate-950 border-slate-800 text-purple-600 focus:ring-0 w-4 h-4 cursor-pointer"
              />
              <ImageIcon className="w-3.5 h-3.5 text-purple-400" />
              <span>支持多模态识图 (Vision)</span>
            </label>

            <label className="flex items-center space-x-1.5 cursor-pointer">
              <input
                type="checkbox"
                checked={newModel.supports_reasoning}
                onChange={(e) => setNewModel({ ...newModel, supports_reasoning: e.target.checked })}
                className="rounded bg-slate-950 border-slate-800 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
              />
              <Brain className="w-3.5 h-3.5 text-indigo-400" />
              <span>支持推理思考 (Reasoning)</span>
            </label>

            <label className="flex items-center space-x-1.5 cursor-pointer">
              <input
                type="checkbox"
                checked={newModel.supports_tools}
                onChange={(e) => setNewModel({ ...newModel, supports_tools: e.target.checked })}
                className="rounded bg-slate-950 border-slate-800 text-amber-600 focus:ring-0 w-4 h-4 cursor-pointer"
              />
              <Wrench className="w-3.5 h-3.5 text-amber-400" />
              <span>支持工具调用 (Tool Calling)</span>
            </label>
          </div>

          <div className="flex justify-end space-x-2 pt-2">
            <button
              onClick={() => setIsAdding(false)}
              className="px-3 py-1.5 rounded bg-slate-800 text-slate-400 text-xs"
            >
              取消
            </button>
            <button
              onClick={handleAddModel}
              className="px-4 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium"
            >
              保存至 Model Registry
            </button>
          </div>
        </div>
      )}

      {/* Model Registry Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900 overflow-hidden">
        <table className="w-full text-left border-collapse text-xs">
          <thead>
            <tr className="bg-slate-950 border-b border-slate-800 text-slate-400">
              <th className="p-3 font-semibold">模型标识 (Model ID)</th>
              <th className="p-3 font-semibold">上下文 / 最大输出</th>
              <th className="p-3 font-semibold">输入单价 (/1M)</th>
              <th className="p-3 font-semibold">输出单价 (/1M)</th>
              <th className="p-3 font-semibold">Cached 单价</th>
              <th className="p-3 font-semibold">能力特性标志</th>
              <th className="p-3 font-semibold text-right">操作</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60">
            {models.map((m) => (
              <tr key={m.id} className="hover:bg-slate-800/40 transition-colors">
                <td className="p-3 font-mono font-bold text-slate-200">
                  {m.display_name}
                  <span className="block text-[10px] text-slate-500 font-normal">{m.id}</span>
                </td>
                <td className="p-3 font-mono text-slate-300">
                  {m.context_window?.toLocaleString()} <span className="text-slate-500">/ {m.max_output_tokens?.toLocaleString()}</span>
                </td>
                <td className="p-3 text-emerald-400 font-mono">${m.input_price_per_1m.toFixed(2)}</td>
                <td className="p-3 text-emerald-400 font-mono">${m.output_price_per_1m.toFixed(2)}</td>
                <td className="p-3 text-slate-400 font-mono">${m.cached_input_price_per_1m.toFixed(3)}</td>
                <td className="p-3 space-x-1">
                  {m.supports_vision && (
                    <span className="px-1.5 py-0.5 rounded bg-purple-900/60 text-purple-300 text-[10px] inline-flex items-center space-x-1">
                      <ImageIcon className="w-3 h-3 mr-0.5" />
                      <span>识图</span>
                    </span>
                  )}
                  {m.supports_reasoning && (
                    <span className="px-1.5 py-0.5 rounded bg-indigo-900/60 text-indigo-300 text-[10px] inline-flex items-center space-x-1">
                      <Brain className="w-3 h-3 mr-0.5" />
                      <span>推理</span>
                    </span>
                  )}
                  {m.supports_tools && (
                    <span className="px-1.5 py-0.5 rounded bg-amber-900/60 text-amber-300 text-[10px]">
                      Tools
                    </span>
                  )}
                </td>
                <td className="p-3 text-right space-x-2">
                  <button
                    onClick={() => handleTestModel(m.id, m.supports_vision)}
                    className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-indigo-300 text-[10px] font-mono border border-slate-700"
                  >
                    测试
                  </button>
                  <button
                    onClick={() => handleDeleteModel(m.id)}
                    className="p-1 text-slate-500 hover:text-red-400"
                    title="删除模型"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
