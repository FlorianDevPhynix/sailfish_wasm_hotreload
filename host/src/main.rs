use core::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
};

use extism::*;
use notify_debouncer_full::{DebounceEventHandler, DebounceEventResult};
use shared::Postcard;

//static WASM_INSTANCE: Mutex<Option<HashMap<String, Plugin>>> = Mutex::new(None);

/* lazy_static::lazy_static! {
    pub static ref WASM_INSTANCE: Mutex<HashMap<String, Plugin>> = {
        Mutex::new(HashMap::new())
    };
} */

#[derive(Debug, Clone)]
struct Crate {
    name: String,
    target_dir: String,
}

impl Crate {
    fn new(name: String, target_dir: String) -> Self {
        Self { name, target_dir }
    }

    fn build(&self) {
        println!("building guest...");
        std::process::Command::new("cargo")
            .args([
                "build",
                "-p",
                &self.name,
                "--target",
                "wasm32-unknown-unknown",
            ])
            .status()
            .unwrap();
    }

    pub fn load(&self) {
        let module_path = format!(
            "{}/wasm32-unknown-unknown/debug/{}.wasm",
            self.target_dir, self.name
        );
        println!("module: {}", module_path);
        let manifest = Manifest::new([Wasm::file(module_path)]);

        let plugin = Plugin::new(manifest, [], false).unwrap();
        shared::WASM_INSTANCE
            .lock()
            .unwrap()
            .insert(self.name.clone(), plugin);
    }

    pub fn call(&self) {
        let mut lock = shared::WASM_INSTANCE.lock().unwrap();
        let instance = lock.get_mut(&self.name);

        if let Some(instance) = instance {
            println!("calling guest...");
            let result = instance.call::<Postcard<guest::TestTemplate>, String>(
                "render_test_template",
                Postcard(guest::TestTemplate::new("Builder".into(), "Bob".into())),
            );
            println!("{:?}", result);
        } else {
            println!("instance not found");
        }
    }
}

impl DebounceEventHandler for Crate {
    fn handle_event(&mut self, result: DebounceEventResult) {
        //println!("event: {:?}", result);

        self.build();
        self.load();
    }
}

fn main() {
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
    // extism doesn't seem to care about windows paths
    let target_dir = metadata.target_directory.as_str().replace("\\", "/");

    let guest_package = metadata
        .packages
        .into_iter()
        .find(|p| p.name == "guest")
        .unwrap();
    let manifest_path = guest_package.manifest_path;
    println!("manifest: {}", manifest_path.parent().unwrap());

    let dependencys: Vec<_> = guest_package
        .dependencies
        .iter()
        .filter(|dep| dep.source.is_none())
        .collect();
    println!("dependencys: {:?}", dependencys);
    println!(
        "dependencys: {:?}",
        dependencys
            .iter()
            .map(|dep| dep.name.as_str())
            .collect::<Vec<_>>()
    );

    let guest_crate = Crate::new("guest".to_string(), target_dir);
    guest_crate.build();
    guest_crate.load();

    let mut watcher =
        notify_debouncer_full::new_debouncer(Duration::from_secs(1), None, guest_crate.clone())
            .unwrap();
    // add main package to watcher
    watcher
        .watch(
            manifest_path.parent().unwrap(),
            notify_debouncer_full::notify::RecursiveMode::Recursive,
        )
        .unwrap();
    // add all dependencies to the watcher
    for dep in dependencys {
        if let Some(path) = dep.path.as_ref() {
            watcher
                .watch(
                    path,
                    notify_debouncer_full::notify::RecursiveMode::Recursive,
                )
                .unwrap();
        } else {
            println!("could not watch package {}", dep.name);
        }
    }

    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    let handler_pair = Arc::clone(&pair);
    ctrlc::set_handler(move || {
        let (lock, cvar) = &*handler_pair;
        let mut exit = lock.lock().unwrap();
        *exit = true;
        // We notify the condvar that the value has changed.
        cvar.notify_one();
    })
    .unwrap();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        //guest_crate.call();
        guest::TestTemplate::new("Builder".into(), "Bob".into()).render_once();

        if *pair.0.lock().unwrap() {
            break;
        }
    }

    // wait for ctrl-c
    /* let (lock, cvar) = &*pair;
    let mut exit = lock.lock().unwrap();
    while !*exit {
        exit = cvar.wait(exit).unwrap();
    } */
}
