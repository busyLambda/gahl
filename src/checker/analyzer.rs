use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
        Arc, Mutex,
    },
    thread,
};

use crate::ast::Module;

use super::{mdir::MiddleIR, CheckError, Checker};

pub struct Analyzer {
    modules: Arc<HashMap<String, Arc<Module>>>,
    mdir_modules: Arc<Mutex<HashMap<String, MiddleIR>>>,
}

impl Analyzer {
    pub fn new(modules: Arc<HashMap<String, Arc<Module>>>) -> Self {
        Self {
            modules,
            mdir_modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn analyze(&mut self) -> Result<(), ()> {
        let tasks = self.modules.len();
        let task_counter = Arc::new(AtomicUsize::new(tasks));

        let found_errors = Arc::new(AtomicBool::new(false));

        // TODO: Create common error manager inface with context about where the error should be.
        for (name, module) in self.modules.iter() {
            let module_c = module.clone();
            let modules_c = self.modules.clone();
            let task_counter_c = task_counter.clone();

            let module = module_c.as_ref();
            let mut checker = Checker::new(module, modules_c);

            let mdir_module = checker.types();

            if checker.errors().len() != 0 {
                checker.print_interrupts();
                found_errors.store(true, SeqCst);
            }

            self.mdir_modules
                .lock()
                .unwrap()
                .insert(name.clone(), mdir_module);

            thread::spawn(move || {
                task_counter_c.fetch_sub(1, SeqCst);
            });
        }

        loop {
            if task_counter.load(SeqCst) == 0 {
                break;
            }
        }

        if found_errors.load(SeqCst) {
            Err(())
        } else {
            Ok(())
        }
    }
}
