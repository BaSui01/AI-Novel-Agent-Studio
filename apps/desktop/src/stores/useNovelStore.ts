import { create } from 'zustand';
import { NovelProject, Volume, Chapter, CharacterCard, WorldSetting, CostMetrics } from '../types/novel';

interface NovelState {
  currentProject: NovelProject | null;
  volumes: Volume[];
  activeChapter: Chapter | null;
  characters: CharacterCard[];
  worldSettings: WorldSetting[];
  activeTab: 'editor' | 'outline' | 'characters' | 'world' | 'gateway';
  
  // Realtime Telemetry
  totalCostUsd: number;
  totalTokens: number;
  lastTps: number;
  lastTtftMs: number;
  isGenerating: boolean;

  // Actions
  setCurrentProject: (project: NovelProject | null) => void;
  setVolumes: (volumes: Volume[]) => void;
  setActiveChapter: (chapter: Chapter | null) => void;
  updateChapterContent: (chapterId: string, content: string, wordCount: number) => void;
  setActiveTab: (tab: 'editor' | 'outline' | 'characters' | 'world' | 'gateway') => void;
  recordMetrics: (metrics: CostMetrics, usageTokens: number) => void;
  setIsGenerating: (generating: boolean) => void;
}

export const useNovelStore = create<NovelState>((set) => ({
  currentProject: {
    id: 'proj_demo_01',
    title: '星穹仙途：天道回收站',
    genre: '玄幻修仙 / 系统流',
    targetAudience: '18-35岁网文读者',
    writingStyle: '热血杀伐果断、暗黑设定、扣人心弦',
    description: '一个被废黜的宗门弃徒，无意间获得天道垃圾桶，可以通过回收诸天大佬废弃的法宝与功法逆天改命。',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  volumes: [
    {
      id: 'vol_1',
      projectId: 'proj_demo_01',
      title: '第一卷：宗门弃徒与垃圾天道',
      sortOrder: 1,
      summary: '主角在灵剑宗被诬陷下山，意外激活天道回收系统，回收残破飞剑。',
      chapters: [
        {
          id: 'chap_1',
          volumeId: 'vol_1',
          title: '第一章：被撕毁的婚书与废宝堆',
          content: '大雪纷飞，灵剑宗外门广场上冷冽刺骨。\n\n陆沉抹去角色的血迹，冷眼看着面前高高在上的师兄。手里的外门弟子令牌已被一掌捏成粉碎。\n\n“陆沉，你私吞宗门灵石，罪无可赦！即日起逐出灵剑宗！”',
          outline: '介绍主角落魄现状，引出反派打压，最后落脚于垃圾山意外激活天道回收站。',
          summary: '陆沉被驱逐后来到后山垃圾堆，获得系统。',
          wordCount: 1200,
          status: 'draft',
          sortOrder: 1,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
        {
          id: 'chap_2',
          volumeId: 'vol_1',
          title: '第二章：天道垃圾桶，万物皆可回收！',
          content: '【叮！检测到断裂的九天玄铁剑，是否回收？】\n\n脑海中冰冷的系统音轰然响起。',
          outline: '系统首次功能展示，回收残剑获得极品剑意。',
          wordCount: 1500,
          status: 'draft',
          sortOrder: 2,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ],
    },
  ],
  activeChapter: {
    id: 'chap_1',
    volumeId: 'vol_1',
    title: '第一章：被撕毁的婚书与废宝堆',
    content: '大雪纷飞，灵剑宗外门广场上冷冽刺骨。\n\n陆沉抹去角色的血迹，冷眼看着面前高高在上的师兄。手里的外门弟子令牌已被一掌捏成粉碎。\n\n“陆沉，你私吞宗门灵石，罪无可赦！即日起逐出灵剑宗！”',
    outline: '介绍主角落魄现状，引出反派打压，最后落脚于垃圾山意外激活天道回收站。',
    summary: '陆沉被驱逐后来到后山垃圾堆，获得系统。',
    wordCount: 1200,
    status: 'draft',
    sortOrder: 1,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  characters: [
    {
      id: 'char_1',
      projectId: 'proj_demo_01',
      name: '陆沉',
      aliases: ['陆师弟', '废柴小子'],
      roleType: 'protagonist',
      appearance: '身穿破旧青衫，身姿挺拔如松，眼神坚毅如铁，眉宇间有一道隐秘的剑纹。',
      personality: '隐忍果决、重情重义、对待敌人冷酷无情、绝不圣母。',
      goals: '重回修仙界巅峰，查明当年家族灭门惨案真相。',
      catchphrases: ['在我眼中，天下万物不过是待回收的垃圾。'],
      forbiddenRules: ['绝对不能圣母软心肠', '不能无脑相信陌生人'],
    },
  ],
  worldSettings: [
    {
      id: 'ws_1',
      projectId: 'proj_demo_01',
      category: 'power_system',
      name: '修仙境界',
      content: '练气、筑基、金丹、元婴、化神、炼虚、合体、大乘、渡劫。每个大境界分为一至九重。',
      tags: ['境界', '设定'],
    },
  ],
  activeTab: 'editor',
  totalCostUsd: 0.0425,
  totalTokens: 28450,
  lastTps: 45.2,
  lastTtftMs: 320,
  isGenerating: false,

  setCurrentProject: (project) => set({ currentProject: project }),
  setVolumes: (volumes) => set({ volumes }),
  setActiveChapter: (chapter) => set({ activeChapter: chapter }),
  updateChapterContent: (chapterId, content, wordCount) => set((state) => {
    const updatedActive = state.activeChapter?.id === chapterId 
      ? { ...state.activeChapter, content, wordCount }
      : state.activeChapter;
    return { activeChapter: updatedActive };
  }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  recordMetrics: (metrics, usageTokens) => set((state) => ({
    totalCostUsd: state.totalCostUsd + metrics.costUsd,
    totalTokens: state.totalTokens + usageTokens,
    lastTps: metrics.tps,
    lastTtftMs: metrics.ttftMs,
  })),
  setIsGenerating: (isGenerating) => set({ isGenerating }),
}));
