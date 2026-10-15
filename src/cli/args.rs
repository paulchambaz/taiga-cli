//! The parsed command line: one argument struct per command.

/// Arguments of `login`.
#[derive(Debug)]
pub struct LoginArgs {
    pub address: Option<String>,
}

/// Arguments of `new`.
#[derive(Debug)]
pub struct NewTaskArgs {
    pub project: String,
    pub status: Option<String>,
    pub name: String,
    pub assign: Vec<String>,
    pub due_date: Option<String>,
    pub team: bool,
    pub client: bool,
    pub block: bool,
    pub sprint: Option<String>,
}

/// Arguments of `move`.
#[derive(Debug)]
pub struct MoveTaskArgs {
    pub project: String,
    pub id: usize,
    pub status: String,
}

/// Arguments of the commands that only name a task, `done` and `delete`.
#[derive(Debug)]
pub struct TaskArgs {
    pub project: String,
    pub id: usize,
}

/// Arguments of `rename`.
#[derive(Debug)]
pub struct RenameTaskArgs {
    pub project: String,
    pub id: usize,
    pub name: String,
}

/// Arguments of `assign`.
#[derive(Debug)]
pub struct AssignTaskArgs {
    pub project: String,
    pub id: usize,
    pub username: String,
    pub remove: bool,
}

/// Arguments of `due`.
#[derive(Debug)]
pub struct DueTaskArgs {
    pub project: String,
    pub id: usize,
    pub due_date: Option<String>,
}

/// Arguments of `sprint`.
#[derive(Debug)]
pub struct SprintTaskArgs {
    pub project: String,
    pub id: usize,
    pub sprint: Option<String>,
}

/// What `subtask` should do with the tasks under a user story.
#[derive(Debug)]
pub enum SubtaskAction {
    New {
        name: String,
        description: String,
        assign: Option<String>,
    },
    List,
    Describe {
        index: usize,
        description: String,
    },
    Delete {
        index: usize,
    },
}

/// Arguments of `subtask`.
#[derive(Debug)]
pub struct SubtaskArgs {
    pub project: String,
    pub story_id: usize,
    pub action: SubtaskAction,
}

/// Arguments of the commands that switch a task requirement, `team`, `client` and `block`.
#[derive(Debug)]
pub struct ToggleTaskArgs {
    pub project: String,
    pub id: usize,
    pub remove: bool,
}

/// Arguments of `modify`; `None` leaves the field untouched.
#[derive(Debug)]
pub struct ModifyTaskArgs {
    pub project: String,
    pub id: usize,
    pub status: Option<String>,
    pub rename: Option<String>,
    pub assign: Option<Vec<String>>,
    pub due_date: Option<String>,
    pub team: Option<bool>,
    pub client: Option<bool>,
    pub block: Option<bool>,
    pub sprint: Option<Option<String>>,
}

/// Arguments of `search`.
#[derive(Debug)]
pub struct SearchTaskArgs {
    pub project: String,
    pub include_statuses: Vec<String>,
    pub exclude_statuses: Vec<String>,
    pub include_assigned: Vec<String>,
    pub exclude_assigned: Vec<String>,
    pub include_sprints: Vec<String>,
    pub exclude_sprints: Vec<String>,
    pub due_date: Option<String>,
    pub team: Option<bool>,
    pub client: Option<bool>,
    pub block: Option<bool>,
    pub query: Vec<String>,
}

/// Arguments of `users`.
#[derive(Debug)]
pub struct ProjectUserArgs {
    pub project: String,
}

/// Arguments of `burndown`.
#[derive(Debug)]
pub struct ProjectBurndownArgs {
    pub project: String,
    pub sprint: Option<String>,
}

/// A fully parsed command.
#[derive(Debug)]
pub enum TaigaCmd {
    Default,
    Login(LoginArgs),
    Projects,
    NewTask(NewTaskArgs),
    MoveTask(MoveTaskArgs),
    DoneTask(TaskArgs),
    RenameTask(RenameTaskArgs),
    AssignTask(AssignTaskArgs),
    DueTask(DueTaskArgs),
    SprintTask(SprintTaskArgs),
    Subtask(SubtaskArgs),
    TeamTask(ToggleTaskArgs),
    ClientTask(ToggleTaskArgs),
    BlockTask(ToggleTaskArgs),
    ModifyTask(ModifyTaskArgs),
    SearchTask(SearchTaskArgs),
    DeleteTask(TaskArgs),
    ProjectUsers(ProjectUserArgs),
    ProjectBurndown(ProjectBurndownArgs),
}
