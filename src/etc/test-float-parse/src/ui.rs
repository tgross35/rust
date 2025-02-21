//! Progress bars and such.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{Completed, Config, EarlyExit, FinishedAll, TestInfo};

/// Templates for progress bars.
const PB_TEMPLATE: &str = "[{elapsed:3} {percent:3}%] {bar:20.cyan/blue} NAME ({pos}/{len}, {msg} f, {per_sec}, eta {eta})";
const PB_TEMPLATE_FINAL: &str =
    "[{elapsed:3} {percent:3}%] NAME ({pos}/{len}, {msg:.COLOR}, {per_sec}, {elapsed_precise})";

/// Create a new progress bar within a multiprogress bar.
pub fn create_pb(
    total_tests: u64,
    short_name_padded: &str,
    all_bars: &mut Vec<ProgressBar>,
) -> ProgressBar {
    let pb = ProgressBar::new(total_tests);
    let pb_style = ProgressStyle::with_template(&PB_TEMPLATE.replace("NAME", short_name_padded))
        .unwrap()
        .progress_chars("##-");

    pb.set_style(pb_style.clone());
    pb.set_message("0");
    all_bars.push(pb.clone());
    pb
}

/// Removes the status bar and replace it with a message.
pub fn finalize_pb(mp: &MultiProgress, pb: &ProgressBar, short_name_padded: &str, c: &Completed) {
    let f = c.failures;

    // Use a tuple so we can use colors
    let (color, msg, finish_pb): (&str, String, fn(&ProgressBar)) = match &c.result {
        Ok(FinishedAll) if f > 0 => {
            ("red", format!("{f} f (completed with errors)",), ProgressBar::finish)
        }
        Ok(FinishedAll) => {
            ("green", format!("{f} f (completed successfully)",), ProgressBar::finish)
        }
        Err(EarlyExit::Timeout) => ("red", format!("{f} f (timed out)"), ProgressBar::abandon),
        Err(EarlyExit::MaxFailures) => {
            ("red", format!("{f} f (failure limit)"), ProgressBar::abandon)
        }
    };

    let pb_style = ProgressStyle::with_template(
        &PB_TEMPLATE_FINAL.replace("NAME", short_name_padded).replace("COLOR", color),
    )
    .unwrap();

    pb.set_style(pb_style);
    finish_pb(pb);
    mp.println(msg).unwrap();
}

/// Print final messages after all tests are complete.
pub fn finish(tests: &[TestInfo], total_elapsed: Duration, cfg: &Config) -> ExitCode {
    println!("\n\nResults:");

    let mut failed_generators = 0;
    let mut stopped_generators = 0;

    for t in tests {
        let Completed { executed, failures, elapsed, warning, result } = t.completed.get().unwrap();

        let stat = if result.is_err() {
            stopped_generators += 1;
            "STOPPED"
        } else if *failures > 0 {
            failed_generators += 1;
            "FAILURE"
        } else {
            "SUCCESS"
        };

        println!(
            "    {stat} for generator '{name}'. {passed}/{executed} passed in {elapsed:?}",
            name = t.name,
            passed = executed - failures,
        );

        if let Some(warning) = warning {
            println!("      warning: {warning}");
        }

        match result {
            Ok(FinishedAll) => (),
            Err(EarlyExit::Timeout) => {
                println!("      exited early; exceded {:?} timeout", cfg.timeout)
            }
            Err(EarlyExit::MaxFailures) => {
                println!("      exited early; exceeded {:?} max failures", cfg.max_failures)
            }
        }
    }

    println!(
        "{passed}/{} tests succeeded in {total_elapsed:?} ({passed} passed, {} failed, {} stopped)",
        tests.len(),
        failed_generators,
        stopped_generators,
        passed = tests.len() - failed_generators - stopped_generators,
    );

    if failed_generators > 0 || stopped_generators > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// indicatif likes to eat panic messages. This workaround isn't ideal, but it improves things.
/// <https://github.com/console-rs/indicatif/issues/121>.
pub fn set_panic_hook(drop_bars: &[ProgressBar]) {
    let hook = std::panic::take_hook();
    let drop_bars = drop_bars.to_owned();
    std::panic::set_hook(Box::new(move |info| {
        for bar in &drop_bars {
            bar.abandon();
            println!();
            io::stdout().flush().unwrap();
            io::stderr().flush().unwrap();
        }
        hook(info);
    }));
}
