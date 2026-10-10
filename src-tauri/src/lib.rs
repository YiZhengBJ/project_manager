use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyRecord {
    todo: String,
    note: String,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: String,
    title: String,
    owner: String,
    #[serde(default)]
    goal: String,
    #[serde(default)]
    risk: String,
    progress: u8,
    parent: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    logs: BTreeMap<String, DailyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    items: Vec<Item>,
    members: Vec<String>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum StoredProject {
    Current(Project),
    Legacy(Vec<Item>),
}
fn validate_project(project: &Project) -> Result<(), String> {
    validate(&project.items)?;
    let members: HashSet<&str> = project.members.iter().map(|name| name.as_str()).collect();
    if members.is_empty()
        || members.len() != project.members.len()
        || project
            .members
            .iter()
            .any(|name| name.trim().is_empty() || name.trim() != name || name.chars().count() > 20)
    {
        return Err("负责人列表为空、重复或姓名无效".into());
    }
    if project
        .items
        .iter()
        .any(|item| !members.contains(item.owner.as_str()))
    {
        return Err("事项负责人不在负责人列表中".into());
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceProject {
    id: String,
    #[serde(flatten)]
    project: Project,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Workspace {
    projects: Vec<WorkspaceProject>,
    active_project_id: String,
}
fn validate_workspace(workspace: &Workspace) -> Result<(), String> {
    let ids: HashSet<&str> = workspace.projects.iter().map(|p| p.id.as_str()).collect();
    if ids.len() != workspace.projects.len() || workspace.projects.iter().any(|p| p.id.is_empty()) {
        return Err("项目 ID 为空或重复".into());
    }
    if workspace.projects.is_empty() {
        if !workspace.active_project_id.is_empty() {
            return Err("空工作空间不能有当前项目".into());
        }
    } else if !ids.contains(workspace.active_project_id.as_str()) {
        return Err("当前项目不存在".into());
    }
    for p in &workspace.projects {
        validate_project(&p.project)?;
    }
    Ok(())
}
fn read_workspace(path: &Path) -> Result<Option<Workspace>, String> {
    let content = match fs::read(path) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let value: serde_json::Value = serde_json::from_slice(&content).map_err(|e| e.to_string())?;
    let workspace = if value.get("projects").is_some() {
        serde_json::from_value(value).map_err(|e| e.to_string())?
    } else {
        Workspace {
            projects: vec![WorkspaceProject {
                id: "default".into(),
                project: read_project(path)?.ok_or("项目文件为空")?,
            }],
            active_project_id: "default".into(),
        }
    };
    validate_workspace(&workspace)?;
    Ok(Some(workspace))
}
fn write_workspace(path: &Path, workspace: &Workspace) -> Result<(), String> {
    validate_workspace(workspace)?;
    let content = serde_json::to_vec_pretty(workspace).map_err(|e| e.to_string())?;
    write_document(path, &content)
}
struct ProjectStore {
    path: PathBuf,
    lock: Mutex<()>,
}

fn validate(items: &[Item]) -> Result<(), String> {
    if items.is_empty() || items.len() > 10_000 {
        return Err("事项数量必须在 1 到 10000 之间".into());
    }
    let ids: HashSet<&str> = items.iter().map(|n| n.id.as_str()).collect();
    if ids.len() != items.len() || items.iter().any(|n| n.id.is_empty()) {
        return Err("事项 ID 为空或重复".into());
    }
    if items.iter().filter(|n| n.parent.is_none()).count() != 1 {
        return Err("项目必须有且仅有一个根节点".into());
    }
    let by_id: BTreeMap<&str, &Item> = items.iter().map(|n| (n.id.as_str(), n)).collect();
    for item in items {
        if item.title.trim().is_empty() || item.owner.trim().is_empty() || item.progress > 100 {
            return Err("事项名称、负责人或完成度无效".into());
        }
        if !matches!(
            item.status.as_deref(),
            None | Some("not_started") | Some("in_progress")
        ) {
            return Err("事项状态无效".into());
        }
        let mut seen = HashSet::new();
        let mut current = item;
        while let Some(parent_id) = current.parent.as_deref() {
            if !seen.insert(current.id.as_str()) {
                return Err("事项层级不能形成循环".into());
            }
            let parent = by_id.get(parent_id).ok_or("上级事项不存在")?;
            if parent.status.as_deref() == Some("not_started")
                && current.status.as_deref() != Some("not_started")
            {
                return Err("未进行事项的下级也必须为未进行".into());
            }
            current = parent;
        }
    }
    Ok(())
}

fn read_project(path: &Path) -> Result<Option<Project>, String> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("读取项目失败：{error}")),
    };
    let stored: StoredProject = serde_json::from_slice(&content)
        .map_err(|error| format!("项目文件格式错误（原文件未修改）：{error}"))?;
    let project = match stored {
        StoredProject::Current(project) => project,
        StoredProject::Legacy(items) => {
            let mut members: Vec<String> = ["郑一", "林晓", "陈默", "苏宁"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            for item in &items {
                if !members.contains(&item.owner) {
                    members.push(item.owner.clone());
                }
            }
            Project { items, members }
        }
    };
    validate_project(&project)?;
    Ok(Some(project))
}

#[cfg(test)]
fn write_project(path: &Path, project: &Project) -> Result<(), String> {
    validate_project(project)?;
    let content = serde_json::to_vec_pretty(project).map_err(|e| e.to_string())?;
    write_document(path, &content)
}
fn write_document(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("数据目录无效")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp_path = path.with_extension("json.tmp");
    let backup_path = path.with_extension("json.bak");
    if path.exists() {
        fs::copy(path, backup_path).map_err(|e| format!("备份项目失败：{e}"))?;
    }
    use std::io::Write;
    let mut file = fs::File::create(&temp_path).map_err(|e| e.to_string())?;
    file.write_all(content).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temp_path, path).map_err(|e| format!("保存项目失败：{e}"))?;
    Ok(())
}

#[tauri::command]
fn load_project(store: tauri::State<ProjectStore>) -> Result<Option<Workspace>, String> {
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    read_workspace(&store.path)
}

#[tauri::command]
fn save_project(workspace: Workspace, store: tauri::State<ProjectStore>) -> Result<(), String> {
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    write_workspace(&store.path, &workspace)
}

#[tauri::command]
fn copy_text(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            app.manage(ProjectStore {
                path: app.path().app_data_dir()?.join("project.json"),
                lock: Mutex::new(()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_project,
            save_project,
            copy_text
        ])
        .run(tauri::generate_context!())
        .expect("无法启动项目管理应用");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Vec<Item> {
        serde_json::from_value(serde_json::json!([
            {"id":"root","title":"示例项目","owner":"郑一","progress":0,"parent":null},
            {"id":"child","title":"事项","owner":"林晓","progress":50,"parent":"root"}
        ]))
        .unwrap()
    }
    #[test]
    fn optional_goal_and_risk_round_trip() {
        let mut items = sample();
        assert!(items[0].goal.is_empty() && items[0].risk.is_empty());
        items[0].goal = "完成设计\n明确验收标准".into();
        items[0].risk = "需求变动".into();
        let document = Workspace {
            projects: vec![WorkspaceProject {
                id: "example".into(),
                project: Project {
                    items,
                    members: vec!["郑一".into(), "林晓".into()],
                },
            }],
            active_project_id: "example".into(),
        };
        let folder =
            std::env::temp_dir().join(format!("project-manager-notes-test-{}", std::process::id()));
        let path = folder.join("project.json");
        write_workspace(&path, &document).unwrap();
        let loaded = read_workspace(&path).unwrap().unwrap();
        assert_eq!(
            loaded.projects[0].project.items[0].goal,
            "完成设计\n明确验收标准"
        );
        assert_eq!(loaded.projects[0].project.items[0].risk, "需求变动");
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn hierarchy_and_status_validation() {
        let mut items = sample();
        assert!(validate(&items).is_ok());
        items[0].parent = Some("child".into());
        assert!(validate(&items).is_err());
        items = sample();
        items[1].parent = Some("missing".into());
        assert!(validate(&items).is_err());
        items = sample();
        items[0].status = Some("not_started".into());
        assert!(validate(&items).is_err());
        items[1].status = Some("not_started".into());
        assert!(validate(&items).is_ok());
    }
    #[test]
    fn saves_backup_and_does_not_overwrite_valid_data_with_invalid_data() {
        let folder =
            std::env::temp_dir().join(format!("project-manager-test-{}", std::process::id()));
        let path = folder.join("project.json");
        let mut project = Project {
            items: sample(),
            members: vec!["郑一".into(), "林晓".into()],
        };
        write_project(&path, &project).unwrap();
        project.items[1].title = "更新事项".into();
        write_project(&path, &project).unwrap();
        assert_eq!(
            read_project(&path).unwrap().unwrap().items[1].title,
            "更新事项"
        );
        assert_eq!(
            read_project(&path.with_extension("json.bak"))
                .unwrap()
                .unwrap()
                .items[1]
                .title,
            "事项"
        );
        project.items[1].parent = Some("missing".into());
        assert!(write_project(&path, &project).is_err());
        assert_eq!(
            read_project(&path).unwrap().unwrap().items[1].title,
            "更新事项"
        );
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn legacy_data_migrates_and_members_persist() {
        let folder = std::env::temp_dir().join(format!(
            "project-manager-members-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("project.json");
        fs::write(&path, serde_json::to_vec(&sample()).unwrap()).unwrap();
        let mut project = read_project(&path).unwrap().unwrap();
        assert_eq!(project.items.len(), 2);
        project.members.push("新负责人".into());
        write_project(&path, &project).unwrap();
        assert!(read_project(&path)
            .unwrap()
            .unwrap()
            .members
            .contains(&"新负责人".into()));
        project.members.retain(|name| name != "林晓");
        assert!(validate_project(&project).is_err());
        project.items[1].owner = "新负责人".into();
        assert!(validate_project(&project).is_ok());
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn workspace_migration_multiple_projects_and_empty_workspace() {
        let folder = std::env::temp_dir().join(format!(
            "project-manager-workspace-test-{}",
            std::process::id()
        ));
        let path = folder.join("project.json");
        let project = Project {
            items: sample(),
            members: vec!["郑一".into(), "林晓".into()],
        };
        write_project(&path, &project).unwrap();
        let mut workspace = read_workspace(&path).unwrap().unwrap();
        assert_eq!(workspace.projects.len(), 1);
        workspace.projects.push(WorkspaceProject {
            id: "second".into(),
            project,
        });
        workspace.active_project_id = "second".into();
        write_workspace(&path, &workspace).unwrap();
        let loaded = read_workspace(&path).unwrap().unwrap();
        assert_eq!(loaded.active_project_id, "second");
        assert_eq!(loaded.projects.len(), 2);
        workspace.projects.clear();
        workspace.active_project_id.clear();
        write_workspace(&path, &workspace).unwrap();
        assert!(read_workspace(&path).unwrap().unwrap().projects.is_empty());
        workspace.active_project_id = "deleted".into();
        assert!(validate_workspace(&workspace).is_err());
        fs::remove_dir_all(folder).unwrap();
    }
}
