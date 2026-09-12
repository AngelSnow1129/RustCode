use crate::model::ProjectModel;

/// Render a Mermaid `graph TD` of the module dependency graph.
pub fn render_mermaid(model: &ProjectModel) -> String {
    let mut out = String::from("graph TD\n");
    let mut ids: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for m in &model.modules {
        let id = sanitize(&m.name);
        ids.insert(m.name.clone(), id.clone());
        out.push_str(&format!("    {id}[\"{}\"]\n", escape(&m.name)));
    }
    for m in &model.modules {
        if let Some(from) = ids.get(&m.name) {
            for dep in &m.depends_on {
                if let Some(to) = ids.get(dep) {
                    out.push_str(&format!("    {from} --> {to}\n"));
                }
            }
        }
    }
    out
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn escape(s: &str) -> String {
    s.replace(['"', '\\'], "")
}
