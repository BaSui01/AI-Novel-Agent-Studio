import React, { useState, useEffect } from 'react';
import { useNovelStore } from '../stores/useNovelStore';
import { 
  Bot, 
  Sparkles, 
  Play, 
  Square, 
  RotateCcw, 
  Check, 
  ChevronDown,
  Edit3,
  AlertCircle
} from 'lucide-react';

interface ModelRegistryItem {
  id: string;
  display_name: string;
}

export const AgentPanel: React.FC = () => {
  const { 
    activeChapter, 
    updateChapterContent, 
    isGenerating, 
    setIsGenerating, 
    recordMetrics 
  } = useNovelStore();

  const [selectedAgent, setSelectedAgent] = useState<'Writer' | 'Editor' | 'Reviewer' | 'Outline' | 'Summarizer'>('Writer');
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [availableModels, setAvailableModels] = useState<ModelRegistryItem[]>([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  
  // Custom Prompts Input (Strictly from user input or backend presets)
  const [systemPrompt, setSystemPrompt] = useState<string>('');
  const [instruction, setInstruction] = useState<string>('');
  const [isEditingPrompt, setIsEditingPrompt] = useState(false);

  const [streamResult, setStreamResult] = useState<string>('');

  useEffect(() => {
    fetchModels();
  }, []);

  const fetchModels = async () => {
    try {
      const res = await fetch('http://127.0.0.1:8080/v1/models/registry');
      if (res.ok) {
        const data: ModelRegistryItem[] = await res.json();
        setAvailableModels(data);
        if (data.length > 0) {
          setSelectedModel(data[0].id);
        } else {
          setSelectedModel('');
        }
      } else {
        setAvailableModels([]);
        setSelectedModel('');
      }
    } catch {
      setAvailableModels([]);
      setSelectedModel('');
    }
  };

  const handleRunAgent = async () => {
    if (!selectedModel) {
      setErrorMessage('后端网关未配置或未启用任何模型，请先前往【设置 - 模型配置】添加并启用模型。');
      return;
    }

    setIsGenerating(true);
    setStreamResult('');
    setErrorMessage(null);

    const startTime = Date.now();

    try {
      const response = await fetch('http://127.0.0.1:8080/v1/chat/completions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: selectedModel,
          stream: true,
          messages: [
            { role: 'system', content: systemPrompt },
            { role: 'user', content: instruction }
          ]
        })
      });

      if (!response.ok) {
        const errJson = await response.json().catch(() => null);
        const errorText = errJson?.error?.message || `HTTP ${response.status} 网关响应异常`;
        setErrorMessage(errorText);
        setIsGenerating(false);
        return;
      }

      if (response.body) {
        const reader = response.body.getReader();
        const decoder = new TextDecoder('utf-8');
        let done = false;
        let accumulated = '';

        while (!done) {
          const { value, done: streamDone } = await reader.read();
          done = streamDone;
          if (value) {
            const chunk = decoder.decode(value, { stream: true });
            const lines = chunk.split('\n');

            for (const line of lines) {
              if (line.startsWith('data: ')) {
                const dataStr = line.replace('data: ', '').trim();
                if (dataStr === '[DONE]') {
                  done = true;
                  break;
                }
                try {
                  const json = JSON.parse(dataStr);
                  const content = json.choices?.[0]?.delta?.content || json.choices?.[0]?.delta?.reasoning_content || '';
                  accumulated += content;
                  setStreamResult(accumulated);
                } catch {
                  // Ignore JSON parse chunk error
                }
              }
            }
          }
        }

        const latency = Date.now() - startTime;
        recordMetrics(
          { costUsd: 0.001, latencyMs: latency, ttftMs: 120, tps: (accumulated.length / (latency / 1000 || 1)) },
          accumulated.length
        );
      }
    } catch (err: any) {
      setErrorMessage(`网络连接错误或 AI 网关服务无法访问: ${err?.message || err}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleApplyToChapter = () => {
    if (activeChapter && streamResult) {
      const newContent = activeChapter.content + '\n\n' + streamResult;
      updateChapterContent(activeChapter.id, newContent, newContent.length);
      setStreamResult('');
    }
  };

  return (
    <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col h-full text-slate-300">
      {/* Agent Panel Header */}
      <div className="p-4 border-b border-slate-800 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <div className="w-7 h-7 rounded-lg bg-purple-600/30 border border-purple-500/40 flex items-center justify-center text-purple-400">
            <Bot className="w-4 h-4" />
          </div>
          <span className="font-semibold text-sm text-slate-100">AI Agent 智能体协同中心</span>
        </div>
        <button
          onClick={() => setIsEditingPrompt(!isEditingPrompt)}
          className={`p-1.5 rounded text-xs flex items-center space-x-1 ${
            isEditingPrompt ? 'bg-purple-950 text-purple-300 border border-purple-800' : 'hover:bg-slate-800 text-slate-400'
          }`}
          title="配置 Agent 提示词模板"
        >
          <Edit3 className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Agent Role Selection */}
        <div>
          <label className="block text-xs font-medium text-slate-400 mb-1.5">Agent 智能体角色</label>
          <div className="grid grid-cols-2 gap-2">
            {[
              { id: 'Writer', name: '主笔写手', desc: '根据细纲写正文' },
              { id: 'Editor', name: '编辑润色', desc: '提升画面感与节奏' },
              { id: 'Reviewer', name: '审稿风控', desc: '检查 OOC 与剧情漏洞' },
              { id: 'Summarizer', name: '设定提取', desc: '自动总结与提取设定' },
            ].map((role) => (
              <button
                key={role.id}
                onClick={() => setSelectedAgent(role.id as any)}
                className={`p-2 rounded-lg text-left border transition-all ${
                  selectedAgent === role.id
                    ? 'bg-purple-950/50 border-purple-600/60 text-purple-200'
                    : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'
                }`}
              >
                <div className="text-xs font-semibold">{role.name}</div>
                <div className="text-[10px] text-slate-500 truncate">{role.desc}</div>
              </button>
            ))}
          </div>
        </div>

        {/* Dynamic Model Picker (Strictly from Backend Gateway) */}
        <div>
          <div className="flex items-center justify-between mb-1.5">
            <label className="block text-xs font-medium text-slate-400">选择调度模型 (来自后端网关)</label>
            <button 
              onClick={fetchModels} 
              className="text-[10px] text-purple-400 hover:underline"
            >
              刷新
            </button>
          </div>
          <div className="relative">
            <select
              value={selectedModel}
              onChange={(e) => setSelectedModel(e.target.value)}
              disabled={availableModels.length === 0}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg py-1.5 px-3 text-xs text-slate-200 focus:outline-none focus:border-purple-500 appearance-none font-mono disabled:opacity-50"
            >
              {availableModels.length > 0 ? (
                availableModels.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.display_name} ({m.id})
                  </option>
                ))
              ) : (
                <option value="">-- 暂无注册模型，请前往模型设置配置 --</option>
              )}
            </select>
            <ChevronDown className="w-3.5 h-3.5 text-slate-500 absolute right-2.5 top-2.5 pointer-events-none" />
          </div>
        </div>

        {/* Error Alert Display */}
        {errorMessage && (
          <div className="p-3 rounded-lg bg-rose-950/50 border border-rose-800/60 text-rose-300 text-xs flex items-start space-x-2">
            <AlertCircle className="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
            <div className="leading-relaxed">{errorMessage}</div>
          </div>
        )}

        {/* Custom System Prompt Editing */}
        {isEditingPrompt && (
          <div className="p-3 rounded-lg bg-slate-950 border border-purple-800/60 space-y-2">
            <label className="block text-[11px] font-semibold text-purple-300">自定义 System Prompt 系统提示词</label>
            <textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              rows={3}
              className="w-full bg-slate-900 border border-slate-800 rounded p-2 text-xs text-slate-200 resize-none focus:outline-none focus:border-purple-500"
            />
          </div>
        )}

        {/* Prompt Input Instruction */}
        <div>
          <label className="block text-xs font-medium text-slate-400 mb-1.5">写作诉求指令 (User Instruction)</label>
          <textarea
            value={instruction}
            onChange={(e) => setInstruction(e.target.value)}
            rows={4}
            className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-xs text-slate-200 focus:outline-none focus:border-purple-500 resize-none font-mono"
            placeholder="自定义写手 Agent 的细节诉求..."
          />
        </div>

        {/* Action Button */}
        <button
          onClick={handleRunAgent}
          disabled={isGenerating || availableModels.length === 0}
          className={`w-full py-2.5 px-4 rounded-lg font-medium text-xs flex items-center justify-center space-x-2 transition-all ${
            isGenerating || availableModels.length === 0
              ? 'bg-slate-800 text-slate-500 cursor-not-allowed'
              : 'bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white shadow-lg shadow-purple-900/30'
          }`}
        >
          {isGenerating ? (
            <>
              <Square className="w-4 h-4 animate-spin text-purple-400" />
              <span>SSE 真实 API 请求推演中...</span>
            </>
          ) : (
            <>
              <Play className="w-4 h-4 fill-current" />
              <span>唤醒 {selectedAgent} Agent 生成</span>
            </>
          )}
        </button>

        {/* Stream Result Display */}
        {streamResult && (
          <div className="mt-4 border border-purple-800/40 rounded-lg bg-purple-950/20 p-3 space-y-3">
            <div className="flex items-center justify-between border-b border-purple-800/30 pb-2">
              <span className="text-xs font-semibold text-purple-300 flex items-center space-x-1">
                <Sparkles className="w-3.5 h-3.5" />
                <span>生成结果预览</span>
              </span>
              <span className="text-[10px] text-emerald-400">后端真实 SSE Stream 传输</span>
            </div>
            <p className="text-xs text-slate-300 leading-relaxed whitespace-pre-wrap font-mono">
              {streamResult}
            </p>
            <div className="flex space-x-2 pt-2">
              <button
                onClick={handleApplyToChapter}
                className="flex-1 py-1.5 rounded bg-emerald-600/30 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-600/50 text-xs font-medium flex items-center justify-center space-x-1"
              >
                <Check className="w-3.5 h-3.5" />
                <span>应用至正文</span>
              </button>
              <button
                onClick={handleRunAgent}
                className="p-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-400"
                title="重新生成"
              >
                <RotateCcw className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
