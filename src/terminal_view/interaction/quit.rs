use super::*;
use gpui::PromptLevel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuitRequestTarget {
    Application,
    WindowClose,
}

impl TerminalView {
    pub(in super::super) fn execute_quit_command_action(
        &mut self,
        action: CommandAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match action {
            CommandAction::Quit => {
                self.request_application_quit(window, cx);
                true
            }
            CommandAction::RestartApp => {
                match self.restart_application() {
                    Ok(()) => {
                        self.allow_quit_without_prompt = true;
                        cx.quit();
                    }
                    Err(error) => {
                        termy_toast::error(format!("Restart failed: {}", error));
                        cx.notify();
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub(in super::super) fn restart_application(&self) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| format!("current_exe failed: {}", e))?;

        #[cfg(target_os = "macos")]
        {
            let app_bundle = exe
                .ancestors()
                .find(|path| {
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("app"))
                        .unwrap_or(false)
                })
                .map(PathBuf::from);

            if let Some(app_bundle) = app_bundle {
                let status = Command::new("open")
                    .arg("-n")
                    .arg(&app_bundle)
                    .status()
                    .map_err(|e| format!("failed to launch app bundle: {}", e))?;
                if status.success() {
                    return Ok(());
                }
                return Err(format!("open returned non-success status: {}", status));
            }
        }

        Command::new(&exe)
            .spawn()
            .map_err(|e| format!("failed to spawn executable: {}", e))?;
        Ok(())
    }

    fn busy_tab_titles_for_quit(&self) -> Vec<String> {
        let fallback_title = self.fallback_title();
        self.tabs
            .iter()
            .enumerate()
            .filter(|(_, tab)| tab.running_process || tab.terminal.alternate_screen_mode())
            .map(|(index, tab)| {
                let title = tab.title.trim();
                if title.is_empty() {
                    format!("{fallback_title} {}", index + 1)
                } else {
                    title.to_string()
                }
            })
            .collect()
    }

    fn quit_warning_detail(&self, busy_titles: &[String]) -> String {
        let count = busy_titles.len();
        let mut detail = format!(
            "{} tab{} {} running a command or fullscreen terminal app:\n",
            count,
            if count == 1 { "" } else { "s" },
            if count == 1 { "has" } else { "have" },
        );

        for title in busy_titles {
            detail.push_str("- ");
            detail.push_str(title);
            detail.push('\n');
        }

        detail.push_str("\nQuit anyway?");
        detail
    }

    fn request_quit(
        &mut self,
        target: QuitRequestTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.quit_prompt_in_flight {
            return false;
        }

        let busy_titles = self.busy_tab_titles_for_quit();
        if !self.warn_on_quit_with_running_process || busy_titles.is_empty() {
            if target == QuitRequestTarget::Application {
                self.allow_quit_without_prompt = true;
                cx.quit();
                return false;
            }
            return true;
        }

        self.quit_prompt_in_flight = true;
        let detail = self.quit_warning_detail(&busy_titles);
        let prompt = window.prompt(
            PromptLevel::Warning,
            "Quit Termy?",
            Some(&detail),
            &["Quit", "Cancel"],
            cx,
        );
        let window_handle = window.window_handle();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let confirmed = matches!(prompt.await, Ok(0));
            let _ = cx.update(|cx| {
                let mut follow_through = false;
                if this
                    .update(cx, |view, _| {
                        view.quit_prompt_in_flight = false;
                        if confirmed {
                            view.allow_quit_without_prompt = true;
                            follow_through = true;
                        }
                    })
                    .is_err()
                {
                    return;
                }

                if !follow_through {
                    return;
                }

                match target {
                    QuitRequestTarget::Application => cx.quit(),
                    QuitRequestTarget::WindowClose => {
                        let _ = window_handle.update(cx, |_, window, _| window.remove_window());
                    }
                }
            });
        })
        .detach();

        false
    }

    pub(crate) fn handle_window_should_close_request(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.allow_quit_without_prompt {
            self.allow_quit_without_prompt = false;
            return true;
        }

        self.request_quit(QuitRequestTarget::WindowClose, window, cx)
    }

    pub(in super::super) fn request_application_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.request_quit(QuitRequestTarget::Application, window, cx);
    }
}
