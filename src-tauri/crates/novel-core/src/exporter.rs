use crate::models::{Project, Volume};

pub struct NovelExporter;

impl NovelExporter {
    pub fn export_to_markdown(project: &Project, volumes: &[Volume]) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", project.title));
        out.push_str(&format!("> 类型：{} | 标签：{}\n\n", project.genre, project.target_audience));
        out.push_str(&format!("{}\n\n---\n\n", project.description));

        for vol in volumes {
            out.push_str(&format!("## {}\n\n", vol.title));
            if let Some(ref sum) = vol.summary {
                out.push_str(&format!("*卷摘要：{}*\n\n", sum));
            }

            for chap in &vol.chapters {
                out.push_str(&format!("### {}\n\n", chap.title));
                out.push_str(&chap.content);
                out.push_str("\n\n---\n\n");
            }
        }
        out
    }

    pub fn export_to_txt(project: &Project, volumes: &[Volume]) -> String {
        let mut out = String::new();
        out.push_str(&format!("《{}》\n\n", project.title));
        out.push_str(&format!("简介：{}\n\n", project.description));

        for vol in volumes {
            out.push_str(&format!("============== {} ==============\n\n", vol.title));

            for chap in &vol.chapters {
                out.push_str(&format!("{}\n\n", chap.title));
                out.push_str(&chap.content);
                out.push_str("\n\n\n");
            }
        }
        out
    }

    pub fn export_to_html(project: &Project, volumes: &[Volume]) -> String {
        let mut out = String::new();
        out.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'><title>");
        out.push_str(&project.title);
        out.push_str("</title><style>body{font-family:SimSun,Georgia,serif;line-height:1.8;padding:2em;}h1{text-align:center;}h2{border-bottom:1px solid #ccc;padding-bottom:0.3em;}</style></head><body>");
        out.push_str(&format!("<h1>{}</h1>", project.title));
        out.push_str(&format!("<p><em>{}</em></p><hr>", project.description));

        for vol in volumes {
            out.push_str(&format!("<h2>{}</h2>", vol.title));
            for chap in &vol.chapters {
                out.push_str(&format!("<h3>{}</h3>", chap.title));
                let html_content = chap.content.replace('\n', "<br>");
                out.push_str(&format!("<p>{}</p>", html_content));
            }
        }
        out.push_str("</body></html>");
        out
    }
}
