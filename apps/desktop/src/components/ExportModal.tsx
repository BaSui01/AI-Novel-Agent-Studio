import React, { useState } from 'react';
import { Download, FileText, FileCode, Check, X } from 'lucide-react';
import { useNovelStore } from '../stores/useNovelStore';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ExportModal: React.FC<ExportModalProps> = ({ isOpen, onClose }) => {
  const { currentProject, volumes } = useNovelStore();
  const [selectedFormat, setSelectedFormat] = useState<'markdown' | 'txt' | 'docx' | 'epub'>('markdown');
  const [isExported, setIsExported] = useState(false);

  if (!isOpen) return null;

  const handleExport = () => {
    let content = '';
    let filename = `${currentProject?.title || 'novel'}`;

    if (selectedFormat === 'markdown') {
      content = `# ${currentProject?.title}\n\n> ${currentProject?.genre}\n\n${currentProject?.description}\n\n`;
      volumes.forEach((v) => {
        content += `\n## ${v.title}\n\n`;
        v.chapters?.forEach((c) => {
          content += `\n### ${c.title}\n\n${c.content}\n\n`;
        });
      });
      filename += '.md';
    } else if (selectedFormat === 'txt') {
      content = `《${currentProject?.title}》\n\n${currentProject?.description}\n\n`;
      volumes.forEach((v) => {
        content += `\n==== ${v.title} ====\n\n`;
        v.chapters?.forEach((c) => {
          content += `\n${c.title}\n\n${c.content}\n\n`;
        });
      });
      filename += '.txt';
    } else {
      content = `<!DOCTYPE html><html><body><h1>${currentProject?.title}</h1></body></html>`;
      filename += selectedFormat === 'docx' ? '.html' : '.epub';
    }

    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    URL.revokeObjectURL(url);

    setIsExported(true);
    setTimeout(() => setIsExported(false), 2000);
  };

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-6 space-y-5 text-slate-200 shadow-2xl">
        <div className="flex items-center justify-between border-b border-slate-800 pb-3">
          <h3 className="font-bold text-base text-slate-100 flex items-center space-x-2">
            <Download className="w-5 h-5 text-indigo-400" />
            <span>导出小说作品</span>
          </h3>
          <button onClick={onClose} className="p-1 hover:bg-slate-800 rounded text-slate-400">
            <X className="w-4 h-4" />
          </button>
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-400 mb-2">选择导出目标格式</label>
          <div className="grid grid-cols-2 gap-3">
            {[
              { id: 'markdown', name: 'Markdown (.md)', desc: '标准排版高兼容性' },
              { id: 'txt', name: '纯文本 (.txt)', desc: '阅读器与手机便携' },
              { id: 'docx', name: 'Word 文档 (.docx)', desc: '支持富文本直接打印' },
              { id: 'epub', name: '电子书 (.epub)', desc: '标准电子书制作' },
            ].map((fmt) => (
              <button
                key={fmt.id}
                onClick={() => setSelectedFormat(fmt.id as any)}
                className={`p-3 rounded-lg border text-left transition-all ${
                  selectedFormat === fmt.id
                    ? 'bg-indigo-950/60 border-indigo-500/80 text-indigo-200'
                    : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'
                }`}
              >
                <div className="font-medium text-xs text-slate-100">{fmt.name}</div>
                <div className="text-[10px] text-slate-500">{fmt.desc}</div>
              </button>
            ))}
          </div>
        </div>

        <button
          onClick={handleExport}
          className="w-full py-2.5 px-4 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs flex items-center justify-center space-x-2 transition-all shadow-lg shadow-indigo-900/30"
        >
          {isExported ? (
            <>
              <Check className="w-4 h-4 text-emerald-400" />
              <span>文件导出成功！</span>
            </>
          ) : (
            <>
              <Download className="w-4 h-4" />
              <span>确认导出全书 ({selectedFormat.toUpperCase()})</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
};
