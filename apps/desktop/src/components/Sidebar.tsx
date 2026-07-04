import React from 'react';
import { useNovelStore } from '../stores/useNovelStore';
import {
  BookOpen,
  ChevronDown,
  Plus,
  Cpu,
  Compass,
  Download,
  DollarSign,
  History,
  Feather,
  Layers
} from 'lucide-react';

interface SidebarProps {
  activeTab: 'editor' | 'gateway' | 'timeline' | 'models' | 'audit' | 'workflow';
  setActiveTab: (tab: 'editor' | 'gateway' | 'timeline' | 'models' | 'audit' | 'workflow') => void;
  onOpenExport: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeTab, setActiveTab, onOpenExport }) => {
  const {
    currentProject,
    volumes,
    activeChapter,
    setActiveChapter
  } = useNovelStore();

  return (
    <div className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col h-full text-slate-300 select-none">
      {/* App Title Banner */}
      <div className="p-4 border-b border-slate-800 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 rounded-lg bg-indigo-600 flex items-center justify-center text-white font-bold shadow-md shadow-indigo-900/50">
            <BookOpen className="w-4 h-4" />
          </div>
          <div>
            <h1 className="font-bold text-sm text-slate-100 leading-tight">AI Novel Studio</h1>
            <p className="text-[10px] text-slate-500 font-mono">Gateway & Agent v0.3.0</p>
          </div>
        </div>
      </div>

      {/* Main Tab Switches */}
      <div className="p-3 border-b border-slate-800 space-y-1">
        <button
          onClick={() => setActiveTab('editor')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'editor'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <Feather className="w-4 h-4" />
          <span>小说正文写作</span>
        </button>

        <button
          onClick={() => setActiveTab('workflow')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'workflow'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <Layers className="w-4 h-4 text-purple-400" />
          <span>Agent 工作流编排</span>
        </button>

        <button
          onClick={() => setActiveTab('gateway')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'gateway'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <Cpu className="w-4 h-4" />
          <span>AI 网关设置</span>
        </button>

        <button
          onClick={() => setActiveTab('models')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'models'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <DollarSign className="w-4 h-4 text-emerald-400" />
          <span>模型注册与价格表</span>
        </button>

        <button
          onClick={() => setActiveTab('audit')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'audit'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <History className="w-4 h-4 text-amber-400" />
          <span>API 消费与审计日志</span>
        </button>

        <button
          onClick={() => setActiveTab('timeline')}
          className={`w-full px-3 py-2 rounded-lg text-xs font-medium flex items-center space-x-2 transition-all ${activeTab === 'timeline'
              ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
              : 'hover:bg-slate-800 text-slate-400'
            }`}
        >
          <Compass className="w-4 h-4" />
          <span>时间线与伏笔追踪</span>
        </button>
      </div>

      {/* Chapters Tree List */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {currentProject ? (
          <div>
            <div className="flex items-center justify-between text-xs text-slate-400 font-medium mb-2 px-1">
              <span className="truncate">{currentProject.title}</span>
              <button
                className="hover:bg-slate-800 p-1 rounded text-slate-300"
                title="新增章节"
              >
                <Plus className="w-3.5 h-3.5" />
              </button>
            </div>

            {volumes.map((volume) => (
              <div key={volume.id} className="space-y-1 mb-3">
                <div className="text-[11px] font-semibold text-slate-500 px-1 py-1 flex items-center space-x-1">
                  <ChevronDown className="w-3 h-3 text-slate-600" />
                  <span>{volume.title}</span>
                </div>
                <div className="pl-2 space-y-0.5">
                  {volume.chapters?.map((chap) => (
                    <button
                      key={chap.id}
                      onClick={() => {
                        setActiveChapter(chap);
                        setActiveTab('editor');
                      }}
                      className={`w-full px-2 py-1.5 rounded text-left text-xs flex items-center justify-between transition-all ${activeChapter?.id === chap.id
                          ? 'bg-slate-800 text-indigo-300 font-medium'
                          : 'hover:bg-slate-800/60 text-slate-400'
                        }`}
                    >
                      <span className="truncate">{chap.title}</span>
                      <span className="text-[10px] text-slate-600 font-mono">{chap.wordCount}字</span>
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-xs text-slate-500">
            暂无小说项目，请点击上方创建
          </div>
        )}
      </div>

      {/* Bottom Export Action */}
      <div className="p-3 border-t border-slate-800">
        <button
          onClick={onOpenExport}
          className="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium flex items-center justify-center space-x-2 transition-all"
        >
          <Download className="w-3.5 h-3.5 text-indigo-400" />
          <span>导出全书 (TXT/MD/DOCX/EPUB)</span>
        </button>
      </div>
    </div>
  );
};
