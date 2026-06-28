use std::{path::PathBuf, time::SystemTime};

use crate::scanner::DirNode;

#[derive(Clone, Copy, PartialEq)]
pub enum SortMode {
    Size,
    Name,
    Count,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Size => SortMode::Name,
            SortMode::Name => SortMode::Count,
            SortMode::Count => SortMode::Size,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Size => "size",
            SortMode::Name => "name",
            SortMode::Count => "count",
        }
    }
}

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
    pub sort_mode: SortMode,
}

impl App {
    pub fn new(root: DirNode, scanned_at: SystemTime, _root_path: PathBuf) -> Self {
        Self {
            root,
            nav_stack: vec![],
            selected: 0,
            scanned_at,
            confirm: None,
            sort_mode: SortMode::Size,
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

    pub fn cycle_sort(&mut self) {
        // Save ancestor paths and current selection path before mutating the tree.
        let ancestor_paths: Vec<PathBuf> = {
            let mut paths = Vec::new();
            let mut node = &self.root;
            for &idx in &self.nav_stack {
                paths.push(node.children[idx].path.clone());
                node = &node.children[idx];
            }
            paths
        };
        let selected_path = self
            .current_children()
            .get(self.selected)
            .map(|c| c.path.clone());

        self.sort_mode = self.sort_mode.next();
        sort_subtree(&mut self.root, self.sort_mode);

        // Rebuild nav_stack indices to match the new order.
        self.nav_stack.clear();
        let mut node = &self.root;
        for path in &ancestor_paths {
            let idx = node
                .children
                .iter()
                .position(|c| &c.path == path)
                .unwrap_or(0);
            self.nav_stack.push(idx);
            node = &node.children[idx];
        }

        self.selected = selected_path
            .and_then(|p| self.current_children().iter().position(|c| c.path == p))
            .unwrap_or(0);
    }
}

fn sort_subtree(node: &mut DirNode, mode: SortMode) {
    match mode {
        SortMode::Size => node.children.sort_by(|a, b| b.size.cmp(&a.size)),
        SortMode::Name => node.children.sort_by(|a, b| a.name.cmp(&b.name)),
        SortMode::Count => node.children.sort_by(|a, b| b.file_count.cmp(&a.file_count)),
    }
    for child in &mut node.children {
        sort_subtree(child, mode);
    }
}
