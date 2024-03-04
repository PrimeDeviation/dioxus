use crate::ScopeId;
use crate::Task;
use std::borrow::Borrow;
use std::cell::Cell;
use std::cell::RefCell;
use std::hash::Hash;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Eq)]
pub struct ScopeOrder {
    pub(crate) height: u32,
    pub(crate) id: ScopeId,
}

impl ScopeOrder {
    pub fn new(height: u32, id: ScopeId) -> Self {
        Self { height, id }
    }
}

impl PartialEq for ScopeOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for ScopeOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScopeOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.height.cmp(&other.height).then(self.id.cmp(&other.id))
    }
}

impl Hash for ScopeOrder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Default)]
pub struct DirtyScopes {
    pub(crate) scopes: BTreeSet<ScopeOrder>,
    pub(crate) tasks: BTreeSet<DirtyTasks>,
}

impl DirtyScopes {
    pub fn queue_task(&mut self, task: Task, order: ScopeOrder) {
        match self.tasks.get(&order) {
            Some(scope) => scope.queue_task(task),
            None => {
                let mut scope = DirtyTasks::from(order);
                scope.queue_task(task);
                self.tasks.insert(scope);
            }
        }
    }

    pub fn queue_scope(&mut self, order: ScopeOrder) {
        self.scopes.insert(order);
    }

    pub fn has_dirty_scopes(&self) -> bool {
        !self.scopes.is_empty()
    }

    pub fn pop_task(&mut self) -> Option<DirtyTasks> {
        self.tasks.pop_first()
    }

    pub fn pop_scope(&mut self) -> Option<ScopeOrder> {
        self.scopes.pop_first()
    }

    pub fn pop_work(&mut self) -> Option<Work> {
        let dirty_scope = self.scopes.first();
        let dirty_task = self.tasks.first();
        match (dirty_scope, dirty_task) {
            (Some(scope), Some(task)) => {
                let tasks_order = task.borrow();
                if scope > tasks_order{
                    let scope = self.scopes.pop_first().unwrap();
                    Some(Work{
                        scope: scope,
                        rerun_scope: true,
                        tasks: Vec::new(),
                    })
                } else if tasks_order> scope {
                    let task = self.tasks.pop_first().unwrap();
                    Some(Work{
                        scope: task.order,
                        rerun_scope: false,
                        tasks: task.tasks_queued.into_inner(),
                    })
                }
                else {
                    let scope = self.scopes.pop_first().unwrap();
                    let task = self.tasks.pop_first().unwrap();
                    Some(Work{
                        scope: scope,
                        rerun_scope: true,
                        tasks: task.tasks_queued.into_inner(),
                    })
                }
            }
            (Some(scope), None) => {
                let scope = self.scopes.pop_first().unwrap();
                Some(Work{
                    scope: scope,
                    rerun_scope: true,
                    tasks: Vec::new(),
                })
            }
            (None, Some(task)) => {
                let task = self.tasks.pop_first().unwrap();
                Some(Work{
                    scope: task.order,
                    rerun_scope: false,
                    tasks: task.tasks_queued.into_inner(),
                })
            }
            (None, None) => None
        }
    }

    pub fn remove(&mut self, scope: &ScopeOrder) {
        self.scopes.remove(scope);
    }
}

pub struct Work {
    pub scope: ScopeOrder,
    pub rerun_scope: bool,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Eq)]
pub(crate) struct DirtyTasks {
    pub order: ScopeOrder,
    pub tasks_queued: RefCell<Vec<Task>>,
}

impl From<ScopeOrder> for DirtyTasks {
    fn from(order: ScopeOrder) -> Self {
        Self {
            order,
            tasks_queued: Vec::new().into(),
        }
    }
}

impl DirtyTasks {

    pub fn queue_task(&self, task: Task) {
        self.tasks_queued.borrow_mut().push(task);
    }
}

impl Borrow<ScopeOrder> for DirtyTasks {
    fn borrow(&self) -> &ScopeOrder {
        &self.order
    }
}

impl PartialOrd for DirtyTasks {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.order.cmp(&other.order))
    }
}

impl Ord for DirtyTasks {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialEq for DirtyTasks {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Hash for DirtyTasks {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.order.hash(state);
    }
}
