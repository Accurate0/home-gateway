use std::path::Path;
use std::sync::Arc;

use home_gateway::lua::testing::{LuaTestRun, LuaTestRunner};
use libtest_mimic::{Arguments, Failed, Trial};

fn outcome(run: LuaTestRun, nocapture: bool) -> Result<(), Failed> {
    if nocapture {
        for line in &run.logs {
            println!("lua: {line}");
        }
    }

    let Err(e) = run.result else {
        return Ok(());
    };

    if nocapture || run.logs.is_empty() {
        return Err(Failed::from(e.to_string()));
    }

    let logs = run
        .logs
        .iter()
        .map(|line| format!("lua: {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    Err(Failed::from(format!("{e}\n\n---- log ----\n{logs}")))
}

fn main() {
    let args = Arguments::from_args();
    let nocapture = args.nocapture;
    let runner = LuaTestRunner::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("config/lua"))
        .expect("expected config/lua to load");

    let mut trials = Vec::new();

    for file in runner
        .files()
        .expect("expected config/lua/tests to be readable")
    {
        let file = Arc::new(file);

        match runner.cases(&file) {
            Ok(cases) => {
                for case in cases {
                    let runner = runner.clone();
                    let file = file.clone();

                    trials.push(Trial::test(format!("{}::{case}", file.name), move || {
                        outcome(runner.run(&file, &case), nocapture)
                    }));
                }
            }
            Err(e) => {
                let message = e.to_string();

                trials.push(Trial::test(file.name.clone(), move || {
                    Err(Failed::from(message))
                }));
            }
        }
    }

    libtest_mimic::run(&args, trials).exit();
}
