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

pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn analyze(
        &mut self,
        modules: Arc<HashMap<String, Arc<Module>>>,
    ) -> Result<HashMap<String, MiddleIR>, ()> {
        let tasks = modules.len();
        let task_counter = Arc::new(AtomicUsize::new(tasks));

        let found_errors = Arc::new(AtomicBool::new(false));

        // TODO: Create common error manager inface with context about where the error should be.
        let modules = modules.clone();

        // let done = task_counter.load(SeqCst) - tasks;
        println!(
            "\x1b[1m\x1b[32mStarting analysis of {} modules...\x1b[0m\n",
            tasks
        );

        let task_counter_c = task_counter.clone();
        let found_errors_c = found_errors.clone();
        let mdir_modules = thread::spawn(move || {
            let mdir_modules = Arc::new(Mutex::new(HashMap::new()));

            let mdir_modules = mdir_modules.clone();
            for (name, module) in modules.iter() {
                let module_c = module.clone();
                let name = name.clone();

                let modules_c = modules.clone();
                let mdir_modules_c = mdir_modules.clone();
                let task_counter_c = task_counter_c.clone();
                let found_errors = found_errors_c.clone();

                thread::spawn(move || {
                    let module = module_c.clone();

                    let mut checker = Checker::new(&module, modules_c);

                    let mdir_module = checker.types();

                    if checker.errors().len() != 0 {
                        checker.print_interrupts();
                        found_errors.store(true, SeqCst);
                    }

                    mdir_modules_c
                        .lock()
                        .unwrap()
                        .insert(name.clone(), mdir_module);

                    task_counter_c.fetch_sub(1, SeqCst);
                });
            }

            mdir_modules
        });

        loop {
            if task_counter.load(SeqCst) == 0 {
                break;
            }
        }

        if found_errors.load(SeqCst) {
            Err(())
        } else {
            let mdir_modules = mdir_modules.join().unwrap();
            Ok(Arc::try_unwrap(mdir_modules).unwrap().into_inner().unwrap())
        }
    }
}
