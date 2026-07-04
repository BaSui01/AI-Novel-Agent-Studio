import React, { useState } from 'react';
import { Search, Database, Layers, Sparkles, Sliders, CheckCircle2, Cpu, Tag, FileText } from 'lucide-react';

export const RagDebugPage: React.FC = () => {
  const [query, setQuery] = useState('陆沉 灵剑宗 废宝堆 九天玄铁剑');
  const [budgetTokens, setBudgetTokens] = useState<number>(2000);
  const [alpha, setAlpha] = useState<number>(0.5); // BM25 vs Vector weight
  const [isSearching, setIsSearching] = useState(false);

  const [recalledItems, setRecalledItems] = useState([
    {
      id: 'ws_1',
      title: '【设定】修仙境界体系',
      type: 'world_setting',
      content: '练气、筑基、金丹、元婴、化神、炼虚、合体、大乘、渡劫。每个大境界分为一至九重。废宝堆属于后山禁地。',
      bm25Score: 0.92,
      vectorScore: 0.88,
      hybridScore: 0.90,
      tokenCount: 180,
    },
    {
      id: 'char_1',
      title: '【人物卡】主角 - 陆沉',
      type: 'character',
      content: '身穿破旧青衫，身姿挺拔如松，眼神坚毅如铁，眉宇间有一道隐秘的剑纹。果决断辣，绝不圣母。',
      bm25Score: 0.85,
      vectorScore: 0.94,
      hybridScore: 0.895,
      tokenCount: 220,
    },
    {
      id: 'chap_summary_1',
      title: '【前章摘要】第一章：废宝堆激活系统',
      type: 'chapter_summary',
      content: '陆沉被宗门大弟子撕毁婚书驱逐，来到废宝堆后成功唤醒天道垃圾回收系统，回收了残破九天玄铁剑。',
      bm25Score: 0.78,
      vectorScore: 0.91,
      hybridScore: 0.845,
      tokenCount: 310,
    },
  ]);

  const handleSearch = () => {
    setIsSearching(true);
    setTimeout(() => {
      setIsSearching(false);
    }, 400);
  };

  const totalRecalledTokens = recalledItems.reduce((acc, item) => acc + item.tokenCount, 0);

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Header Banner */}
      <div className="p-5 rounded-xl bg-gradient-to-r from-indigo-950/80 to-slate-950 border border-indigo-800/40 flex items-center justify-between shadow-lg">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center space-x-2">
            <Database className="w-5 h-5 text-indigo-400" />
            <span>LanceDB 向量索引与 SQLite FTS5 混合检索 (RAG Debugger)</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            测试 Keyword BM25 + Vector Distance 混合召回、加权 Rerank 算法与 Token Budget 预算裁剪。
          </p>
        </div>
      </div>

      {/* Query & Control Bar */}
      <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
        <div className="flex space-x-3">
          <div className="flex-1 relative">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="输入剧情或 Agent 检索 Query..."
              className="w-full bg-slate-950 border border-slate-800 rounded-lg py-2 pl-9 pr-4 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
            />
            <Search className="w-4 h-4 text-slate-500 absolute left-3 top-2.5" />
          </div>
          <button
            onClick={handleSearch}
            className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center space-x-1.5 transition-all shadow-md shadow-indigo-900/40"
          >
            {isSearching ? <Cpu className="w-4 h-4 animate-spin" /> : <Search className="w-4 h-4" />}
            <span>测试混合召回</span>
          </button>
        </div>

        {/* Sliders for Alpha Weight & Token Budget */}
        <div className="grid grid-cols-2 gap-6 text-xs border-t border-slate-800 pt-3">
          <div>
            <div className="flex justify-between text-slate-400 mb-1">
              <span>检索混合加权系数 (Alpha Weight): BM25 ({(alpha * 100).toFixed(0)}%) / Vector ({((1 - alpha) * 100).toFixed(0)}%)</span>
            </div>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={alpha}
              onChange={(e) => setAlpha(parseFloat(e.target.value))}
              className="w-full accent-indigo-500 cursor-pointer"
            />
          </div>

          <div>
            <div className="flex justify-between text-slate-400 mb-1">
              <span>Token Budget 预算上限: {budgetTokens} Tokens (当前占用: {totalRecalledTokens} Tokens)</span>
            </div>
            <input
              type="range"
              min="500"
              max="8000"
              step="500"
              value={budgetTokens}
              onChange={(e) => setBudgetTokens(parseInt(e.target.value))}
              className="w-full accent-indigo-500 cursor-pointer"
            />
          </div>
        </div>
      </div>

      {/* Recalled ContextPack Items */}
      <div className="space-y-3">
        <h3 className="text-xs font-bold text-slate-400 flex items-center space-x-1.5">
          <Sparkles className="w-4 h-4 text-amber-400" />
          <span>ContextPack 命中结果列表 ({recalledItems.length} 个设定 Chunk)</span>
        </h3>

        <div className="space-y-3">
          {recalledItems.map((item) => (
            <div key={item.id} className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-bold text-xs text-indigo-300">{item.title}</span>
                <div className="flex items-center space-x-2 font-mono text-[10px]">
                  <span className="px-2 py-0.5 rounded bg-slate-950 text-slate-400 border border-slate-800">
                    BM25: {item.bm25Score}
                  </span>
                  <span className="px-2 py-0.5 rounded bg-slate-950 text-slate-400 border border-slate-800">
                    Vector: {item.vectorScore}
                  </span>
                  <span className="px-2 py-0.5 rounded bg-indigo-950 text-indigo-300 font-bold border border-indigo-800">
                    Hybrid Score: {item.hybridScore}
                  </span>
                  <span className="px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800">
                    {item.tokenCount} tokens
                  </span>
                </div>
              </div>
              <p className="text-xs text-slate-300 bg-slate-950 p-2.5 rounded border border-slate-800/80 leading-relaxed font-mono">
                {item.content}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
