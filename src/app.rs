use std::{path::PathBuf, time::SystemTime};

use crate::scanner::DirNode;

#[derive(Clone)]
pub enum ConfirmAction {
    Delete,
    Clean,
}

pub struct App {
    pub root: DirNode,
    pub nav_stack: Vec<usize>,
    pub selected: usize,
    pub scanned_at: SystemTime,
    pub confirm: Option<(ConfirmAction, PathBuf)>,
}

impl App {
    pub fn new(root: DirNode, scanned_at: SystemTime, _root_path: PathBuf) -> Self {
        Self {
            root,
            nav_stack: vec![],
            selected: 0,
            scanned_at,
            confirm: None,
        }
    }

    pub fn current_node(&self) -> &DirNode {
        let mut node = &self.root;
        for &idx in &self.nav_stack {
            node = &node.children[idx];
        }
        node
    }

    pub fn current_children(&self) -> &[DirNode] {
        &self.current_node().children
    }

    pub fn move_down(&mut self) {
        let len = self.current_children().len();
        if len > 0 && self.selected + 1 < len {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn enter(&mut self) {
        let can_enter = self
            .current_children()
            .get(self.selected)
            .map(|c| !c.children.is_empty())
            .unwrap_or(false);

        if can_enter {
            self.nav_stack.push(self.selected);
            self.selected = 0;
        }
    }

    pub fn go_up(&mut self) {
        if let Some(prev) = self.nav_stack.pop() {
            self.selected = prev;
        }
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.current_children()
            .get(self.selected)
            .map(|c| c.path.clone())
    }

    pub fn selected_size(&self) -> u64 {
        self.current_children()
            .get(self.selected)
            .map(|c| c.size)
            .unwrap_or(0)
    }

    pub fn start_confirm(&mut self, action: ConfirmAction) {
        if let Some(path) = self.selected_path() {
            self.confirm = Some((action, path));
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
    }
}
