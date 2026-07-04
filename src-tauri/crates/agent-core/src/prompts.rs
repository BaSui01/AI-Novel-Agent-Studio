use std::fs;
use std::path::Path;

pub const DEFAULT_WRITER_PROMPT: &str = include_str!("../../../../prompts/writer.md");
pub const DEFAULT_EDITOR_PROMPT: &str = include_str!("../../../../prompts/editor.md");
pub const DEFAULT_REVIEWER_PROMPT: &str = include_str!("../../../../prompts/reviewer.md");
pub const DEFAULT_SUMMARIZER_PROMPT: &str = include_str!("../../../../prompts/summarizer.md");

pub fn ensure_prompts_directory<P: AsRef<Path>>(dir: P) -> Result<(), std::io::Error> {
    let dir = dir.as_ref();
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let writer_path = dir.join("writer.md");
    if !writer_path.exists() {
        fs::write(&writer_path, DEFAULT_WRITER_PROMPT)?;
    }

    let editor_path = dir.join("editor.md");
    if !editor_path.exists() {
        fs::write(&editor_path, DEFAULT_EDITOR_PROMPT)?;
    }

    let reviewer_path = dir.join("reviewer.md");
    if !reviewer_path.exists() {
        fs::write(&reviewer_path, DEFAULT_REVIEWER_PROMPT)?;
    }

    let summarizer_path = dir.join("summarizer.md");
    if !summarizer_path.exists() {
        fs::write(&summarizer_path, DEFAULT_SUMMARIZER_PROMPT)?;
    }

    Ok(())
}

pub fn load_prompt_file<P: AsRef<Path>>(file_path: P, default_content: &str) -> String {
    fs::read_to_string(file_path).unwrap_or_else(|_| default_content.to_string())
}
