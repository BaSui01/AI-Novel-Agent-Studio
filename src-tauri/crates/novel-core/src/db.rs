use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::models::{CharacterCard, Chapter, Project, Volume, WorldSetting};

pub type DbPool = Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: DbPool,
}

impl DatabaseManager {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::new(manager)?;
        
        let db = Self { pool };
        db.init_tables()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager)?;
        
        let db = Self { pool };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                genre TEXT NOT NULL,
                target_audience TEXT,
                writing_style TEXT,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS volumes (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                summary TEXT,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS chapters (
                id TEXT PRIMARY KEY NOT NULL,
                volume_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                outline TEXT,
                summary TEXT,
                word_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'draft',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(volume_id) REFERENCES volumes(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS characters (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                aliases TEXT,
                gender TEXT,
                age TEXT,
                role_type TEXT NOT NULL,
                appearance TEXT NOT NULL,
                personality TEXT NOT NULL,
                goals TEXT NOT NULL,
                catchphrases TEXT,
                forbidden_rules TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS world_settings (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS timeline_events (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL,
                event_name TEXT NOT NULL,
                occurred_time TEXT,
                chapter_id TEXT,
                description TEXT,
                impact TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS foreshadows (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL,
                content TEXT NOT NULL,
                planted_chapter_id TEXT,
                resolved_chapter_id TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                importance INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS model_registry (
                id TEXT PRIMARY KEY NOT NULL,
                provider TEXT NOT NULL,
                display_name TEXT NOT NULL,
                api_model_name TEXT NOT NULL,
                endpoint_type TEXT NOT NULL DEFAULT 'chat_completions',
                context_window INTEGER NOT NULL DEFAULT 128000,
                max_output_tokens INTEGER NOT NULL DEFAULT 4096,
                supports_streaming BOOLEAN NOT NULL DEFAULT 1,
                supports_tools BOOLEAN NOT NULL DEFAULT 1,
                supports_vision BOOLEAN NOT NULL DEFAULT 0,
                supports_reasoning BOOLEAN NOT NULL DEFAULT 0,
                input_price_per_1m REAL NOT NULL DEFAULT 3.0,
                output_price_per_1m REAL NOT NULL DEFAULT 15.0,
                cached_input_price_per_1m REAL NOT NULL DEFAULT 0.75,
                reasoning_price_per_1m REAL NOT NULL DEFAULT 15.0,
                currency TEXT NOT NULL DEFAULT 'USD',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                is_custom BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS request_logs (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT,
                agent_name TEXT,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                reasoning_tokens INTEGER NOT NULL DEFAULT 0,
                cached_tokens INTEGER NOT NULL DEFAULT 0,
                cost_usd REAL NOT NULL DEFAULT 0.0,
                latency_ms INTEGER NOT NULL DEFAULT 0,
                ttft_ms INTEGER NOT NULL DEFAULT 0,
                tps REAL NOT NULL DEFAULT 0.0,
                status_code INTEGER NOT NULL DEFAULT 200,
                created_at TEXT NOT NULL
            );

            -- Phase 3: Agent 自定义 Prompt 模板表
            CREATE TABLE IF NOT EXISTS agent_prompts (
                id TEXT PRIMARY KEY NOT NULL,
                role_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                user_template TEXT NOT NULL,
                is_default BOOLEAN NOT NULL DEFAULT 1,
                updated_at TEXT NOT NULL
            );
            "
        )?;

        self.seed_default_models_if_empty()?;
        self.seed_default_prompts_if_empty()?;
        Ok(())
    }

    fn seed_default_models_if_empty(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM model_registry", [], |r| r.get(0))?;
        if count == 0 {
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute_batch(&format!("
                INSERT INTO model_registry (id, provider, display_name, api_model_name, endpoint_type, context_window, max_output_tokens, supports_streaming, supports_tools, supports_vision, supports_reasoning, input_price_per_1m, output_price_per_1m, cached_input_price_per_1m, reasoning_price_per_1m, currency, enabled, is_custom, created_at)
                VALUES 
                ('claude-3-5-sonnet-20241022', 'anthropic', 'Claude 3.5 Sonnet', 'claude-3-5-sonnet-20241022', 'chat_completions', 200000, 8192, 1, 1, 1, 0, 3.0, 15.0, 0.75, 15.0, 'USD', 1, 0, '{now}'),
                ('gpt-4o', 'openai', 'OpenAI GPT-4o', 'gpt-4o', 'chat_completions', 128000, 16384, 1, 1, 1, 0, 2.5, 10.0, 1.25, 10.0, 'USD', 1, 0, '{now}'),
                ('o3-mini', 'openai', 'OpenAI o3-mini (Reasoning)', 'o3-mini', 'responses', 200000, 100000, 1, 1, 0, 1, 1.1, 4.4, 0.55, 4.4, 'USD', 1, 0, '{now}'),
                ('gemini-2.0-flash', 'gemini', 'Google Gemini 2.0 Flash', 'gemini-2.0-flash', 'chat_completions', 1048576, 8192, 1, 1, 1, 0, 0.1, 0.4, 0.025, 0.4, 'USD', 1, 0, '{now}'),
                ('qwen2.5:32b', 'ollama', 'Ollama Qwen 2.5 32B (本地离线)', 'qwen2.5:32b', 'chat_completions', 32768, 4096, 1, 1, 0, 0, 0.0, 0.0, 0.0, 0.0, 'USD', 1, 0, '{now}');
            "))?;
        }
        Ok(())
    }

    fn seed_default_prompts_if_empty(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM agent_prompts", [], |r| r.get(0))?;
        if count == 0 {
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute_batch(&format!("
                INSERT INTO agent_prompts (id, role_name, display_name, system_prompt, user_template, is_default, updated_at)
                VALUES
                ('writer', 'Writer', '主笔写手 Agent', '你是一位通俗小说资深主笔，擅长制造冲突与画面感。', '根据细纲续写，控制在 1500 字。', 1, '{now}'),
                ('editor', 'Editor', '编辑润色 Agent', '你是一位资深文学编辑，擅长精简废话与强化感官描写。', '优化段落画面感。', 1, '{now}'),
                ('reviewer', 'Reviewer', '审稿风控 Agent', '你是一位严格的网文总编，负责检查剧情 OOC 与逻辑漏项。', '检查逻辑漏洞并给出评分。', 1, '{now}'),
                ('summarizer', 'Summarizer', '设定总结 Agent', '你是一位小说资料库架构师，负责提取新人物与最新剧情摘要。', '生成章节摘要。', 1, '{now}');
            "))?;
        }
        Ok(())
    }

    pub fn create_project(&self, p: &Project) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO projects (id, title, genre, target_audience, writing_style, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![p.id, p.title, p.genre, p.target_audience, p.writing_style, p.description, p.created_at, p.updated_at],
        )?;
        Ok(())
    }

    pub fn get_projects(&self) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, title, genre, target_audience, writing_style, description, created_at, updated_at FROM projects")?;
        let proj_iter = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                genre: row.get(2)?,
                target_audience: row.get(3)?,
                writing_style: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut projects = Vec::new();
        for p in proj_iter {
            projects.push(p?);
        }
        Ok(projects)
    }

    pub fn update_chapter_content(&self, chapter_id: &str, content: &str, word_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chapters SET content = ?1, word_count = ?2, updated_at = ?3 WHERE id = ?4",
            params![content, word_count as i64, now, chapter_id],
        )?;
        Ok(())
    }
}
