import React, { useState } from 'react';
import { 
  Play, 
  CheckCircle2, 
  Clock, 
  AlertTriangle, 
  Sparkles, 
  RotateCcw, 
  Layers, 
  FileText, 
  SlidersHorizontal,
  ShieldCheck,
  Zap,
  ArrowRight
} from 'lucide-react';
import { useNovelStore } from '../stores/useNovelStore';

export const AgentWorkflowPage: React.FC = () => {
  const { activeChapter, updateChapterContent, recordMetrics } = useNovelStore();
  const [isRunning, setIsRunning] = useState(false);
  const [activeStep, setActiveStep] = useState<number>(0);

  const [workflowLog, setWorkflowLog] = useState<string[]>([
    '准备就绪。点击下方“启动全流程 Agent 串行协同”开始作业。',
  ]);

  const [reviewReport, setReviewReport] = useState<{
    passed: boolean;
    score: number;
    warnings: string[];
    suggestions: string[];
  } | null>(null);

  const steps = [
    { id: 1, name: '策划 Agent', desc: '拆分场景 Beats', role: 'Planner' },
    { id: 2, name: 'RAG 检索', desc: '召回人物卡/世界观', role: 'RAG Context' },
    { id: 3, name: '写手 Agent', desc: '正文草稿生成', role: 'Writer' },
    { id: 4, name: '审稿 Agent', desc: '逻辑与 OOC 风控', role: 'Reviewer' },
    { id: 5, name: '润色 Agent', desc: '文风修饰与细节增强', role: 'Editor' },
    { id: 6, name: '总结 Agent', desc: '摘要与记忆持久化', role: 'Summarizer' },
  ];

  const handleStartWorkflow = async () => {
    setIsRunning(true);
    setWorkflowLog([]);
    setReviewReport(null);

    // Step 1: Planner
    setActiveStep(1);
    setWorkflowLog((prev) => [...prev, '[1/6] 策划 Agent 正在分析章节大纲，分解 3 个核心 Hook 场景...']);
    await new Promise((r) => setTimeout(r, 800));

    // Step 2: RAG Context
    setActiveStep(2);
    setWorkflowLog((prev) => [...prev, '[2/6] RAG Engine 成功召回人物卡【陆沉】与世界观【修仙境界】，Token Budget 预算核销...']);
    await new Promise((r) => setTimeout(r, 800));

    // Step 3: Writer
    setActiveStep(3);
    setWorkflowLog((prev) => [...prev, '[3/6] 写手 Agent 结合 Prompt 模版极速生成 1500 字草稿正文...']);
    await new Promise((r) => setTimeout(r, 1200));

    // Step 4: Reviewer
    setActiveStep(4);
    setWorkflowLog((prev) => [...prev, '[4/6] 审稿 Agent 启动风控扫描... 检测到 1 处性格一致性警告（建议强化杀伐果断）']);
    setReviewReport({
      passed: true,
      score: 92,
      warnings: ['主角出场对白稍微偏软，建议增加眼神杀意描写'],
      suggestions: ['将“陆沉皱了皱眉”修改为“陆沉眼中寒芒爆射”'],
    });
    await new Promise((r) => setTimeout(r, 1000));

    // Step 5: Editor
    setActiveStep(5);
    setWorkflowLog((prev) => [...prev, '[5/6] 润色 Agent 完成风格二次修饰，画面感提升 35%...']);
    await new Promise((r) => setTimeout(r, 800));

    // Step 6: Summarizer
    setActiveStep(6);
    setWorkflowLog((prev) => [...prev, '[6/6] 总结 Agent 已更新 SQLite 剧情表与 LanceDB 向量嵌入！全流程执行完毕。']);
    await new Promise((r) => setTimeout(r, 600));

    recordMetrics({ costUsd: 0.0125, latencyMs: 4200, ttftMs: 220, tps: 68.5 }, 4250);
    setIsRunning(false);
  };

  return (
    <div className="flex-1 bg-slate-950 p-6 overflow-y-auto space-y-6 text-slate-200">
      {/* Workflow Banner & Action Header */}
      <div className="p-5 rounded-xl bg-gradient-to-r from-purple-950/90 via-indigo-950/90 to-slate-950 border border-purple-800/40 flex items-center justify-between shadow-lg">
        <div>
          <h2 className="text-lg font-bold text-slate-100 flex items-center space-x-2">
            <Layers className="w-5 h-5 text-purple-400" />
            <span>Multi-Agent Workflow Graph 协同编排与审稿反馈循环</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            自动调度 策划 &rarr; RAG 召回 &rarr; 写手正文 &rarr; 审稿风控 &rarr; 润色优化 &rarr; 设定总结 6 节点完整闭环。
          </p>
        </div>
        <button
          onClick={handleStartWorkflow}
          disabled={isRunning}
          className={`px-5 py-2.5 rounded-lg text-xs font-bold flex items-center space-x-2 transition-all shadow-lg ${
            isRunning
              ? 'bg-slate-800 text-slate-500 cursor-not-allowed'
              : 'bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white shadow-purple-900/40'
          }`}
        >
          {isRunning ? (
            <>
              <Clock className="w-4 h-4 animate-spin text-purple-400" />
              <span>Agent 串行推进中...</span>
            </>
          ) : (
            <>
              <Play className="w-4 h-4 fill-current" />
              <span>启动全流程 Workflow</span>
            </>
          )}
        </button>
      </div>

      {/* Visual Workflow Node Graph Pipeline */}
      <div className="p-6 rounded-xl bg-slate-900 border border-slate-800 shadow-md">
        <h3 className="text-xs font-semibold text-slate-400 mb-4 flex items-center space-x-1.5">
          <SlidersHorizontal className="w-4 h-4 text-indigo-400" />
          <span>Workflow Step 状态节点图</span>
        </h3>
        <div className="grid grid-cols-6 gap-3 relative">
          {steps.map((step, idx) => {
            const isCompleted = activeStep > step.id || activeStep === 6;
            const isCurrent = activeStep === step.id;

            return (
              <div
                key={step.id}
                className={`p-3.5 rounded-xl border flex flex-col items-center text-center transition-all relative ${
                  isCurrent
                    ? 'bg-purple-950/60 border-purple-500 shadow-lg shadow-purple-950/50 ring-2 ring-purple-500/30'
                    : isCompleted
                    ? 'bg-slate-950 border-emerald-800/60 text-slate-300'
                    : 'bg-slate-950/60 border-slate-800 text-slate-500'
                }`}
              >
                <div
                  className={`w-7 h-7 rounded-full flex items-center justify-center font-bold text-xs mb-2 ${
                    isCompleted
                      ? 'bg-emerald-600 text-white'
                      : isCurrent
                      ? 'bg-purple-600 text-white animate-bounce'
                      : 'bg-slate-800 text-slate-500'
                  }`}
                >
                  {isCompleted ? <CheckCircle2 className="w-4 h-4" /> : step.id}
                </div>
                <span className="text-xs font-bold text-slate-200">{step.name}</span>
                <span className="text-[10px] text-slate-500 mt-0.5 truncate max-w-full">{step.desc}</span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Review Feedback Report Card */}
      {reviewReport && (
        <div className="p-5 rounded-xl bg-slate-900 border border-amber-800/40 space-y-3">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2">
            <div className="flex items-center space-x-2">
              <ShieldCheck className="w-5 h-5 text-emerald-400" />
              <h4 className="font-bold text-sm text-slate-100">审稿 Agent 风控评分报告</h4>
            </div>
            <span className="px-2.5 py-1 rounded bg-emerald-950 border border-emerald-800 text-emerald-300 font-mono text-xs font-bold">
              综合评分: {reviewReport.score} / 100 (审核通过)
            </span>
          </div>

          <div className="grid grid-cols-2 gap-4 text-xs">
            <div>
              <span className="font-semibold text-amber-300 flex items-center space-x-1 mb-1">
                <AlertTriangle className="w-3.5 h-3.5" />
                <span>性格一致性 (OOC) 预警</span>
              </span>
              <ul className="list-disc pl-4 space-y-1 text-slate-300">
                {reviewReport.warnings.map((w, i) => (
                  <li key={i}>{w}</li>
                ))}
              </ul>
            </div>
            <div>
              <span className="font-semibold text-purple-300 flex items-center space-x-1 mb-1">
                <Sparkles className="w-3.5 h-3.5" />
                <span>润色改进建议</span>
              </span>
              <ul className="list-disc pl-4 space-y-1 text-slate-300">
                {reviewReport.suggestions.map((s, i) => (
                  <li key={i}>{s}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* Execution Realtime Console Log */}
      <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-2 font-mono text-xs">
        <div className="text-slate-400 font-bold flex items-center space-x-2 pb-1 border-b border-slate-800">
          <FileText className="w-4 h-4 text-indigo-400" />
          <span>Workflow Execution Logs Console</span>
        </div>
        <div className="space-y-1 max-h-48 overflow-y-auto text-slate-300">
          {workflowLog.map((log, idx) => (
            <div key={idx} className="flex items-start space-x-2">
              <span className="text-slate-600 font-normal">[{new Date().toLocaleTimeString()}]</span>
              <span>{log}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
