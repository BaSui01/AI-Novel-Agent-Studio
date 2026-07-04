export interface NovelProject {
  id: string;
  title: string;
  genre: string;
  targetAudience: string;
  writingStyle: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

export interface Volume {
  id: string;
  projectId: string;
  title: string;
  sortOrder: number;
  summary?: string;
  chapters?: Chapter[];
}

export interface Chapter {
  id: string;
  volumeId: string;
  title: string;
  content: string;
  outline?: string;
  summary?: string;
  wordCount: number;
  status: 'draft' | 'revised' | 'finalized';
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface CharacterCard {
  id: string;
  projectId: string;
  name: string;
  aliases: string[];
  gender?: string;
  age?: string;
  roleType: 'protagonist' | 'antagonist' | 'supporting';
  appearance: string;
  personality: string;
  goals: string;
  catchphrases: string[];
  forbiddenRules: string[];
}

export interface WorldSetting {
  id: string;
  projectId: string;
  category: 'rule' | 'location' | 'power_system' | 'faction' | 'item';
  name: string;
  content: string;
  tags: string[];
}

export interface UnifiedUsage {
  inputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
  cachedTokens: number;
  totalTokens: number;
}

export interface CostMetrics {
  costUsd: number;
  latencyMs: number;
  ttftMs: number;
  tps: number;
}

export interface AgentTask {
  id: string;
  agentName: 'Planner' | 'Outline' | 'Writer' | 'Editor' | 'Reviewer' | 'Summarizer';
  status: 'idle' | 'running' | 'completed' | 'failed';
  prompt: string;
  result?: string;
  metrics?: CostMetrics;
}
