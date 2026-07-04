import React, { useState } from 'react';
import { Clock, Eye, ShieldAlert, Plus, CheckCircle, AlertTriangle, FileText, Download } from 'lucide-react';
import { useNovelStore } from '../stores/useNovelStore';

export const TimelineForeshadowView: React.FC = () => {
  const [activeSubTab, setActiveSubTab] = useState<'timeline' | 'foreshadow'>('foreshadow');
  
  const [foreshadows, setForeshadows] = useState([
    {
      id: 'fs_1',
      content: '陆沉捡到的九天玄铁断剑内部，隐蔽刻有神仙姐姐的残魂真名。',
      plantedChapter: '第一章：被撕毁的婚书与废宝堆',
      resolvedChapter: '未回收 (预计第25章回收)',
      status: 'pending',
      importance: '高',
    },
    {
      id: 'fs_2',
      content: '灵剑宗外门大长老袖口隐藏的黑死病印记，暗中联络魔宗。',
      plantedChapter: '第二章：天道垃圾桶',
      resolvedChapter: '未回收',
      status: 'pending',
      importance: '中',
    },
  ]);

  const [timelineEvents, setTimelineEvents] = useState([
    {
      id: 'tl_1',
      time: '天元历 3200 年初冬',
      name: '陆沉被逐出灵剑宗外门',
      chapter: '第一章',
      impact: '导致陆沉获得天道回收系统，命运发生转折。',
    },
    {
      id: 'tl_2',
      time: '天元历 3200 年初冬夜',
      name: '废宝堆首次激活系统回收玄铁剑',
      chapter: '第二章',
      impact: '陆沉领悟乾坤剑意，迈入练气三重。',
    },
  ]);

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Subtab Navigation Header */}
      <div className="flex items-center justify-between border-b border-slate-800 pb-4">
        <div className="flex space-x-4">
          <button
            onClick={() => setActiveSubTab('foreshadow')}
            className={`flex items-center space-x-2 font-bold text-sm pb-2 border-b-2 transition-colors ${
              activeSubTab === 'foreshadow'
                ? 'border-purple-500 text-purple-300'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <ShieldAlert className="w-4 h-4 text-purple-400" />
            <span>伏笔追踪卡片 (Foreshadow Tracking)</span>
          </button>
          <button
            onClick={() => setActiveSubTab('timeline')}
            className={`flex items-center space-x-2 font-bold text-sm pb-2 border-b-2 transition-colors ${
              activeSubTab === 'timeline'
                ? 'border-indigo-500 text-indigo-300'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Clock className="w-4 h-4 text-indigo-400" />
            <span>剧情时间线图表 (Timeline Chain)</span>
          </button>
        </div>

        <button className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium flex items-center space-x-1">
          <Plus className="w-3.5 h-3.5" />
          <span>添加新{activeSubTab === 'foreshadow' ? '伏笔' : '时间线节点'}</span>
        </button>
      </div>

      {/* Foreshadow View */}
      {activeSubTab === 'foreshadow' && (
        <div className="grid grid-cols-2 gap-4">
          {foreshadows.map((item) => (
            <div key={item.id} className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold px-2 py-0.5 rounded bg-purple-500/20 text-purple-300 border border-purple-500/30">
                  重要程度: {item.importance}
                </span>
                <span className="text-[11px] text-amber-400 flex items-center space-x-1">
                  <AlertTriangle className="w-3.5 h-3.5" />
                  <span>状态: 待回收</span>
                </span>
              </div>
              <p className="text-sm font-medium text-slate-100">{item.content}</p>
              <div className="text-xs text-slate-400 space-y-1 pt-2 border-t border-slate-800">
                <div>📍 埋设章节: {item.plantedChapter}</div>
                <div>🎯 计划回收: {item.resolvedChapter}</div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Timeline View */}
      {activeSubTab === 'timeline' && (
        <div className="space-y-4 relative pl-4 border-l-2 border-indigo-900/60 ml-2">
          {timelineEvents.map((event) => (
            <div key={event.id} className="relative pl-6">
              <div className="absolute -left-[29px] top-1 w-3.5 h-3.5 rounded-full bg-indigo-500 ring-4 ring-slate-950" />
              <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-mono text-indigo-400">{event.time}</span>
                  <span className="text-xs text-slate-500">所属章节: {event.chapter}</span>
                </div>
                <h3 className="font-bold text-slate-100 text-sm">{event.name}</h3>
                <p className="text-xs text-slate-400">{event.impact}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
