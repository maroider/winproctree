//! Prints a tree representing the parent-child relationship between currently running processes.
//!
//! The current implementation suffers from the fact that Windows recycles PIDs, and that Windows is fond of
//! usig small PIDs. This has already resulted in this program printing a bogus tree.

use std::collections::HashMap;

use winproc::Process;

fn main() {
    let mut processes: HashMap<u32, (Process, String)> = HashMap::new();
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for process in Process::all().unwrap() {
        if let Ok(process_name) = process.name() {
            let parent_id = process.parent_id();
            let process_id = process.id();

            processes.insert(process_id, (process, process_name));

            if let Some(parent_id) = parent_id {
                children.entry(parent_id).or_default().push(process_id);
            }
        }
    }

    let mut top_level_processes: Vec<_> = processes
        .iter()
        .filter(|(_, (process, _))| {
            process.parent_id().is_none()
                || process
                    .parent_id()
                    .map(|parent_id| !processes.contains_key(&parent_id))
                    .unwrap_or(false)
        })
        .map(|(key, _)| key)
        .copied()
        .collect();

    top_level_processes.sort_by(|lhs, rhs| {
        let lhs = processes.get(lhs).unwrap();
        let lhs = (&lhs.1, lhs.0.id());
        let rhs = &processes.get(rhs).unwrap();
        let rhs = (&rhs.1, rhs.0.id());
        lhs.cmp(&rhs)
    });
    for ch in children.values_mut() {
        ch.sort_by(|lhs, rhs| {
            let lhs = processes.get(lhs).unwrap();
            let lhs = (&lhs.1, lhs.0.id());
            let rhs = &processes.get(rhs).unwrap();
            let rhs = (&rhs.1, rhs.0.id());
            lhs.cmp(&rhs)
        });
    }

    for top_process_id in top_level_processes {
        let (_, process_name) = processes.get(&top_process_id).unwrap();
        println!("{:<8}{}", top_process_id, process_name);

        if let Some(ch) = children.get(&top_process_id) {
            let mut stack = vec![(ch, 0, false)];
            while !stack.is_empty() {
                let mut stack_top = stack.pop().unwrap();
                let (ch, idx, done) = &mut stack_top;
                if *done {
                    continue;
                }

                let child = ch[*idx];
                let (_, process_name) = processes.get(&child).unwrap();
                println!(
                    "{}{:<8}{}",
                    (0..stack.len() + 1)
                        .map(|i| if i == stack.len() {
                            if ch.len() - 1 > *idx {
                                "├── "
                            } else {
                                "└── "
                            }
                        } else {
                            if stack[i].2 {
                                "    "
                            } else {
                                "│   "
                            }
                        })
                        .collect::<String>(),
                    child,
                    process_name
                );
                if *idx < ch.len() - 1 {
                    *idx += 1;
                } else {
                    *done = true;
                }
                stack.push(stack_top);
                if let Some(ch) = children.get(&child) {
                    stack.push((ch, 0, false));
                }
            }
        }
    }
}
