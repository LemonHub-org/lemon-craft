use std::{panic, panic::PanicHookInfo, path::PathBuf};
use tracing::error;

use crate::ui::brand;

pub fn set_panic_hook(log_filename: String, logs_dir: PathBuf) {
    // Set up panic handler to relay swish panic messages to the user
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let panic_info_payload = panic_info.payload();
        let payload_string = panic_info_payload.downcast_ref::<String>();
        let reason = match payload_string {
            Some(s) => s,
            None => {
                let payload_str = panic_info_payload.downcast_ref::<&str>();
                match payload_str {
                    Some(st) => st,
                    None => "Payload is not a string",
                }
            },
        };
        let potential_cause = potential_cause(panic_info);

        let mut dialog_message = format!(
            "A critical error has occurred and {} has been forced to terminate in an unusual \
             manner. Details about the error can be found below.\n\nPanic reason: {}\n\n",
            brand::PRODUCT_NAME,
            reason
        );

        if let Some(potential_cause) = potential_cause {
            // The error is a known error, so don't show the full bug report instructions
            // and instead show a potential fix.
            dialog_message.push_str(format!("Potential causes: {}\n\n", potential_cause).as_str())
        } else {
            dialog_message.push_str(
                format!(
                    "> What should I do?\n\nWe need your help to fix this! You can help by \
                     contacting the {} maintainers and reporting this problem.\n\n",
                    brand::PRODUCT_NAME
                )
                .as_str(),
            );

            if let Some(tracker) = brand::ISSUE_TRACKER_URL {
                dialog_message.push_str(
                    format!("Open an issue on the issue tracker:\n\n{tracker}\n\n").as_str(),
                );
            } else {
                dialog_message.push_str(
                    "No public issue tracker is configured yet. Please contact the maintainers \
                     with the log file contents described below.\n\n",
                );
            }

            if let Some(discord) = brand::DISCORD_URL {
                dialog_message.push_str(
                    format!(
                        "If you're on the community Discord, please also post in the support \
                         channel:\n{discord}\n\n"
                    )
                    .as_str(),
                );
            }

            dialog_message.push_str(
                format!(
                    "> What should I include?\n\nThe error information below will be useful in \
                     finding and fixing the problem. Please include as much information about \
                     your setup and the events that led up to the panic as possible.
            \n{} has logged information about the problem (including this message) to the file {}. \
                     Please include the contents of this file in your bug report.
            \n\n",
                    brand::PRODUCT_NAME,
                    logs_dir.join(&log_filename).display()
                )
                .as_str(),
            );
        }

        dialog_message.push_str(
            format!(
                "> Error information\n\nThe information below is intended for developers and \
                 testers.\n\nPanicHookInfo: {} \nGame version: {}",
                panic_info,
                *common::util::DISPLAY_VERSION,
            )
            .as_str(),
        );

        error!(
            "{} HAS PANICKED\n\n{}\n\nBacktrace:\n{:?}",
            brand::PRODUCT_NAME.to_uppercase(),
            dialog_message,
            backtrace::Backtrace::new(),
        );

        #[cfg(feature = "native-dialog")]
        {
            use native_dialog::{DialogBuilder, MessageLevel};

            let mbox = move || {
                DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title(format!("{} has crashed!", brand::PRODUCT_NAME))
                    //somehow `<` and `>` are invalid characters and cause the msg to get replaced
                    // by some generic text thus i replace them
                    .set_text(dialog_message.replace('<', "[").replace('>', "]"))
                    .alert()
                    .show()
                    .unwrap()
            };

            // On windows we need to spawn a thread as the msg doesn't work otherwise
            #[cfg(target_os = "windows")]
            {
                let builder = std::thread::Builder::new().name("shutdown".into());
                builder
                    .spawn(move || {
                        mbox();
                    })
                    .unwrap()
                    .join()
                    .unwrap();
            }

            #[cfg(not(target_os = "windows"))]
            mbox();
        }

        default_hook(panic_info);
    }));
}

enum PotentialPanicCause {
    GraphicsCardIncompatibleWithRenderingBackend,
}

fn potential_cause(panic_info: &PanicHookInfo) -> Option<String> {
    let location = panic_info
        .location()
        .map_or("".to_string(), |x| x.file().to_string())
        .to_lowercase();

    // Use some basic heuristics to determine the likely cause of the panic. This is
    // deliberately overly simplistic as the vast majority of graphics errors
    // for example are caused by an incompatible GPU rather than a bug.
    let potential_cause = if location.contains("gfx") || location.contains("wgpu") {
        Some(PotentialPanicCause::GraphicsCardIncompatibleWithRenderingBackend)
    } else {
        None
    };

    potential_cause.map(potential_cause_to_string)
}

fn potential_cause_to_string(potential_cause: PotentialPanicCause) -> String {
    match potential_cause {
        PotentialPanicCause::GraphicsCardIncompatibleWithRenderingBackend => {
            "This error occurs when your graphics card is not compatible with the selected \
             graphics mode. This can be changed in the launcher settings window, however it may be \
             the case that your graphics card is not supported by any graphics mode."
                .to_string()
        },
    }
}
