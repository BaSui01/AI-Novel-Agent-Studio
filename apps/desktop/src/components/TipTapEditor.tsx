import React, { useEffect } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import CharacterCount from '@tiptap/extension-character-count';
import Highlight from '@tiptap/extension-highlight';
import { useNovelStore } from '../stores/useNovelStore';
import { Sparkles, Bold, Italic, List, AlignLeft, RefreshCw, Layers } from 'lucide-react';

export const TipTapEditor: React.FC = () => {
  const { activeChapter, updateChapterContent } = useNovelStore();

  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({
        placeholder: '在此开始撰写正文或唤醒 AI 创作...',
      }),
      CharacterCount,
      Highlight.configure({
        multicolor: true,
      }),
    ],
    content: activeChapter?.content || '',
    onUpdate: ({ editor }) => {
      if (activeChapter) {
        const text = editor.getText();
        updateChapterContent(activeChapter.id, editor.getHTML(), text.length);
      }
    },
  });

  useEffect(() => {
    if (editor && activeChapter && editor.getHTML() !== activeChapter.content) {
      editor.commands.setContent(activeChapter.content);
    }
  }, [activeChapter?.id, editor]);

  if (!activeChapter) {
    return (
      <div className="flex-1 flex items-center justify-center bg-slate-950 text-slate-500">
        请选择或新建一个章节开始写作
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col bg-slate-950 h-full overflow-hidden">
      {/* Editor Toolbar */}
      <div className="h-12 border-b border-slate-800 bg-slate-900/50 px-4 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <h2 className="text-sm font-medium text-slate-200">{activeChapter.title}</h2>
          <span className="text-xs px-2 py-0.5 rounded bg-amber-500/20 text-amber-400 border border-amber-500/30">
            {activeChapter.status === 'draft' ? '草稿' : '已定稿'}
          </span>
        </div>

        <div className="flex items-center space-x-1">
          <button
            onClick={() => editor?.chain().focus().toggleBold().run()}
            className={`p-1.5 rounded hover:bg-slate-800 text-slate-400 ${
              editor?.isActive('bold') ? 'bg-slate-800 text-indigo-400' : ''
            }`}
          >
            <Bold className="w-4 h-4" />
          </button>
          <button
            onClick={() => editor?.chain().focus().toggleItalic().run()}
            className={`p-1.5 rounded hover:bg-slate-800 text-slate-400 ${
              editor?.isActive('italic') ? 'bg-slate-800 text-indigo-400' : ''
            }`}
          >
            <Italic className="w-4 h-4" />
          </button>
          <button
            onClick={() => editor?.chain().focus().toggleBulletList().run()}
            className={`p-1.5 rounded hover:bg-slate-800 text-slate-400 ${
              editor?.isActive('bulletList') ? 'bg-slate-800 text-indigo-400' : ''
            }`}
          >
            <List className="w-4 h-4" />
          </button>
          <div className="w-px h-4 bg-slate-800 mx-1" />
          <button
            className="px-2.5 py-1 text-xs rounded bg-indigo-600/30 text-indigo-300 hover:bg-indigo-600/50 border border-indigo-500/40 flex items-center space-x-1"
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>AI 润色段落</span>
          </button>
        </div>
      </div>

      {/* Main Editor Text Area */}
      <div className="flex-1 overflow-y-auto p-8 max-w-4xl mx-auto w-full">
        <EditorContent editor={editor} className="min-h-full" />
      </div>

      {/* Bottom Bar Info */}
      <div className="h-8 border-t border-slate-800 px-4 bg-slate-900/40 flex items-center justify-between text-xs text-slate-400">
        <div className="flex items-center space-x-4">
          <span>总字数: {editor?.storage.characterCount.characters() || 0} 字</span>
          <span>预计阅读时间: {Math.ceil((editor?.storage.characterCount.characters() || 0) / 400)} 分钟</span>
        </div>
        <div className="flex items-center space-x-2">
          <Layers className="w-3.5 h-3.5 text-indigo-400" />
          <span>TipTap / ProseMirror Engine</span>
        </div>
      </div>
    </div>
  );
};
