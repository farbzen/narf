//! Application state and key handling.

use std::time::Duration;

use anyhow::{Context, Result};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ListState,
};

use crate::{
    EXEC_FLAG,
    runner::Runner,
    status::{self, Hosts, Status},
    ui,
};

pub enum Value {
    Toggle(bool),
    Text(String),
}

/// One NARF option.
pub struct Field {
    pub flag: char,
    pub label: &'static str,
    pub help: &'static str,
    pub value: Value,
}

impl Field {
    fn toggle(flag: char, label: &'static str, help: &'static str) -> Self {
        Self {
            flag,
            label,
            help,
            value: Value::Toggle(false),
        }
    }

    fn text(flag: char, label: &'static str, help: &'static str) -> Self {
        Self {
            flag,
            label,
            help,
            value: Value::Text(String::new()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Options,
    Inputs,
}

pub enum Mode {
    Form,
    /// Editing a text field; holds the value to restore on Esc.
    Edit {
        backup: String,
    },
    /// Choosing the host from the flake's `nixosConfigurations`.
    PickHost(ListState),
    Run(Runner),
}

/// One entry of the host picker.
pub enum HostChoice {
    /// No `-H`, NARF uses the hostname.
    Default,
    Host(String),
    /// Type the host by hand.
    Custom,
}

pub struct Input {
    pub name: String,
    pub selected: bool,
}

pub struct App {
    pub fields: Vec<Field>,
    pub inputs: Vec<Input>,
    pub focus: Focus,
    pub options_state: ListState,
    pub inputs_state: ListState,
    pub mode: Mode,
    pub status: Status,
    pub hosts: Hosts,
    pub command: String,
    /// Shown in the footer until the next key press.
    pub error: Option<String>,
    quit: bool,
}

fn default_fields() -> Vec<Field> {
    vec![
        Field::toggle('p', "Pull", "Pull new commits from origin before running"),
        Field::toggle(
            'u',
            "Update all inputs",
            "Update all flake inputs before running",
        ),
        Field::toggle(
            'b',
            "Build only",
            "Build the new generation without switching",
        ),
        Field::toggle('d', "Dry run", "Only evaluate and state what WOULD be done"),
        Field::toggle('t', "Show trace", "Show trace of the rebuild"),
        Field::toggle(
            'n',
            "No rebuild",
            "Skip the rebuild and only push existing commits",
        ),
        Field::toggle(
            'y',
            "Yes to all",
            "Answer all prompts with yes (non-interactive)",
        ),
        Field::toggle('P', "Push", "Push new commits to origin"),
        Field::toggle('S', "Don't squash", "Don't squash commits before pushing"),
        Field::text('c', "Comment", "Add a comment to the resulting commit"),
        Field::text(
            'f',
            "Flake",
            "Path to the flake (default: ~/.config/narf/flake)",
        ),
        Field::text(
            'H',
            "Host",
            "NixOS host configuration of the flake (default: hostname)",
        ),
    ]
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            fields: default_fields(),
            inputs: Vec::new(),
            focus: Focus::Options,
            options_state: ListState::default().with_selected(Some(0)),
            inputs_state: ListState::default(),
            mode: Mode::Form,
            status: status::load(""),
            hosts: Hosts::default(),
            command: String::new(),
            error: None,
            quit: false,
        };
        app.hosts.load(app.status.flake.as_deref().ok());
        app.sync_inputs();
        app.update_command();
        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            if let Mode::Run(runner) = &mut self.mode {
                runner.poll();
            }
            if self.hosts.loading() {
                self.hosts.poll();
                self.clamp_selection();
            }

            terminal.draw(|frame| ui::draw(frame, self))?;

            if !event::poll(Duration::from_millis(30))? {
                continue;
            }
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    self.on_key(key, terminal)?
                }
                Event::Resize(cols, rows) => {
                    if let Mode::Run(runner) = &mut self.mode {
                        let area = ui::terminal_area(ratatui::layout::Rect::new(0, 0, cols, rows));
                        runner.resize(area.height, area.width);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn field(&self, flag: char) -> &Field {
        self.fields
            .iter()
            .find(|f| f.flag == flag)
            .expect("unknown flag")
    }

    pub fn toggled(&self, flag: char) -> bool {
        matches!(self.field(flag).value, Value::Toggle(true))
    }

    pub fn text(&self, flag: char) -> &str {
        match &self.field(flag).value {
            Value::Text(text) => text.trim(),
            Value::Toggle(_) => "",
        }
    }

    pub fn selected_field(&self) -> Option<&Field> {
        self.options_state
            .selected()
            .and_then(|i| self.fields.get(i))
    }

    /// The arguments NARF will be called with.
    pub fn args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for field in &self.fields {
            match &field.value {
                Value::Toggle(true) => args.push(format!("-{}", field.flag)),
                Value::Text(text) if !text.trim().is_empty() => {
                    args.push(format!("-{}", field.flag));
                    let text = text.trim();
                    args.push(if field.flag == 'f' {
                        narf::flake::expand_tilde(text)
                    } else {
                        text.to_owned()
                    });
                }
                _ => {}
            }
        }
        if !self.toggled('u') {
            for input in self.inputs.iter().filter(|i| i.selected) {
                args.push("-U".to_owned());
                args.push(input.name.clone());
            }
        }
        args
    }

    fn update_command(&mut self) {
        let mut parts = vec!["narf".to_owned()];
        parts.extend(self.args().iter().map(|arg| shell_quote(arg)));
        self.command = parts.join(" ");
    }

    /// Combinations NARF accepts but silently ignores, and an unknown host.
    pub fn hints(&self) -> Vec<String> {
        let mut hints = Vec::new();
        if self.toggled('S') && !self.toggled('P') {
            hints.push("Don't squash has no effect without Push".to_owned());
        }
        if self.toggled('P') && (self.toggled('d') || self.toggled('b')) {
            hints.push("Push is skipped in dry/build-only mode".to_owned());
        }
        if !self.text('c').is_empty() && (self.toggled('d') || self.toggled('b')) {
            hints.push("No commit (and no comment) in dry/build-only mode".to_owned());
        }
        if self.toggled('n')
            && (self.toggled('u')
                || self.inputs.iter().any(|i| i.selected)
                || !self.text('c').is_empty()
                || self.toggled('b')
                || self.toggled('t'))
        {
            hints.push(
                "No rebuild can't be combined with update, comment, build only or trace".to_owned(),
            );
        }
        if self.toggled('u') && self.inputs.iter().any(|i| i.selected) {
            hints.push("Update all inputs overrides the input selection".to_owned());
        }
        let host = self.host();
        if !self.hosts.names.is_empty() && !self.hosts.names.iter().any(|name| name == host) {
            hints.push(format!(
                "Host '{host}' is not in the flake's nixosConfigurations"
            ));
        }
        hints
    }

    /// The host NARF will build for.
    pub fn host(&self) -> &str {
        match self.text('H') {
            "" => &self.status.default_host,
            host => host,
        }
    }

    /// Entries of the host picker: the hostname default, the flake's hosts
    /// and typing one by hand.
    pub fn host_choices(&self) -> Vec<HostChoice> {
        std::iter::once(HostChoice::Default)
            .chain(self.hosts.names.iter().cloned().map(HostChoice::Host))
            .chain(std::iter::once(HostChoice::Custom))
            .collect()
    }

    fn reload_status(&mut self) {
        self.status = status::load(self.text('f'));
        self.hosts.load(self.status.flake.as_deref().ok());
        self.sync_inputs();
    }

    fn host_field(&mut self) -> &mut String {
        match &mut self
            .fields
            .iter_mut()
            .find(|f| f.flag == 'H')
            .expect("unknown flag")
            .value
        {
            Value::Text(text) => text,
            Value::Toggle(_) => unreachable!("host is a text field"),
        }
    }

    fn open_host_picker(&mut self) {
        let current = self.text('H');
        let choices = self.host_choices();
        let index = choices
            .iter()
            .position(|choice| match choice {
                HostChoice::Default => current.is_empty(),
                HostChoice::Host(name) => name == current,
                HostChoice::Custom => true,
            })
            .unwrap_or(0);
        self.mode = Mode::PickHost(ListState::default().with_selected(Some(index)));
    }

    fn on_host_picker_key(&mut self, key: KeyEvent) {
        let Mode::PickHost(state) = &mut self.mode else {
            return;
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Form,
            KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => state.select_next(),
            KeyCode::Home | KeyCode::Char('g') => state.select_first(),
            KeyCode::End | KeyCode::Char('G') => state.select_last(),
            KeyCode::Char(' ') | KeyCode::Enter => {
                let selected = state.selected().unwrap_or(0);
                let choice = self.host_choices().into_iter().nth(selected);
                let text = self.host_field();
                self.mode = match choice {
                    Some(HostChoice::Default) => {
                        text.clear();
                        Mode::Form
                    }
                    Some(HostChoice::Host(name)) => {
                        *text = name;
                        Mode::Form
                    }
                    Some(HostChoice::Custom) => Mode::Edit {
                        backup: text.clone(),
                    },
                    None => Mode::Form,
                };
            }
            _ => {}
        }
        self.clamp_selection();
        self.update_command();
    }

    /// Rebuilds the input list from the lock file, keeping the selection.
    fn sync_inputs(&mut self) {
        let previous = std::mem::take(&mut self.inputs);
        self.inputs = self
            .status
            .inputs
            .iter()
            .map(|name| Input {
                name: name.clone(),
                selected: previous.iter().any(|i| i.selected && &i.name == name),
            })
            .collect();
        let selected = self.inputs_state.selected().unwrap_or(0);
        self.inputs_state
            .select((!self.inputs.is_empty()).then(|| selected.min(self.inputs.len() - 1)));
    }

    fn on_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        match &mut self.mode {
            Mode::Run(runner) => {
                if runner.running() {
                    match key.code {
                        KeyCode::PageUp => runner.scroll(1),
                        KeyCode::PageDown => runner.scroll(-1),
                        _ => runner.send_key(key),
                    }
                } else {
                    match key.code {
                        KeyCode::PageUp => runner.scroll(1),
                        KeyCode::PageDown => runner.scroll(-1),
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                            self.mode = Mode::Form;
                            self.reload_status();
                        }
                        _ => {}
                    }
                }
            }
            Mode::Edit { backup } => {
                let backup = backup.clone();
                let Some(Field {
                    value: Value::Text(text),
                    flag,
                    ..
                }) = self
                    .options_state
                    .selected()
                    .and_then(|i| self.fields.get_mut(i))
                else {
                    self.mode = Mode::Form;
                    return Ok(());
                };
                let flag = *flag;
                match key.code {
                    KeyCode::Enter => {
                        self.mode = Mode::Form;
                        if flag == 'f' {
                            self.reload_status();
                        }
                    }
                    KeyCode::Esc => {
                        *text = backup;
                        self.mode = Mode::Form;
                    }
                    KeyCode::Backspace if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let keep = text.trim_end().rfind(' ').map_or(0, |i| i + 1);
                        text.truncate(keep);
                    }
                    KeyCode::Backspace => {
                        text.pop();
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        text.clear()
                    }
                    KeyCode::Char(c) => text.push(c),
                    _ => {}
                }
                self.update_command();
            }
            Mode::PickHost(_) => self.on_host_picker_key(key),
            Mode::Form => {
                self.error = None;
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.quit = true
                    }
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Char('l') => {
                        if !self.inputs.is_empty() {
                            self.focus = match self.focus {
                                Focus::Options => Focus::Inputs,
                                Focus::Inputs => Focus::Options,
                            };
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => self.list_state().select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => self.list_state().select_next(),
                    KeyCode::Home | KeyCode::Char('g') => self.list_state().select_first(),
                    KeyCode::End | KeyCode::Char('G') => self.list_state().select_last(),
                    KeyCode::Char(' ') | KeyCode::Enter => self.activate(),
                    KeyCode::Char('R') => self.reload_status(),
                    KeyCode::Char('r') => self.start(terminal)?,
                    KeyCode::Char(flag) => self.activate_flag(flag),
                    _ => {}
                }
                self.clamp_selection();
                self.update_command();
            }
        }
        Ok(())
    }

    fn list_state(&mut self) -> &mut ListState {
        match self.focus {
            Focus::Options => &mut self.options_state,
            Focus::Inputs => &mut self.inputs_state,
        }
    }

    /// `select_next`/`select_last` may point past the end until the next render.
    fn clamp_selection(&mut self) {
        let clamp = |state: &mut ListState, len: usize| {
            if let Some(i) = state.selected() {
                state.select((len > 0).then(|| i.min(len - 1)));
            }
        };
        clamp(&mut self.options_state, self.fields.len());
        clamp(&mut self.inputs_state, self.inputs.len());
        let host_choices = self.host_choices().len();
        if let Mode::PickHost(state) = &mut self.mode {
            clamp(state, host_choices);
        }
    }

    fn activate(&mut self) {
        match self.focus {
            Focus::Options => {
                if self.selected_field().is_some_and(|f| f.flag == 'H') {
                    self.open_host_picker();
                    return;
                }
                let Some(field) = self
                    .options_state
                    .selected()
                    .and_then(|i| self.fields.get_mut(i))
                else {
                    return;
                };
                match &mut field.value {
                    Value::Toggle(on) => *on = !*on,
                    Value::Text(text) => {
                        self.mode = Mode::Edit {
                            backup: text.clone(),
                        }
                    }
                }
            }
            Focus::Inputs => {
                if let Some(input) = self
                    .inputs_state
                    .selected()
                    .and_then(|i| self.inputs.get_mut(i))
                {
                    input.selected = !input.selected;
                }
            }
        }
    }

    /// Selects the option of `flag` and toggles or edits it.
    fn activate_flag(&mut self, flag: char) {
        let Some(index) = self.fields.iter().position(|f| f.flag == flag) else {
            return;
        };
        self.focus = Focus::Options;
        self.options_state.select(Some(index));
        self.activate();
    }

    fn start(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let area = ui::terminal_area(terminal.size()?.into());
        let exe = std::env::current_exe().context("could not locate the narf-tui binary")?;
        let mut args = vec![EXEC_FLAG.to_owned()];
        args.extend(self.args());
        match Runner::spawn(&exe, &args, area.height, area.width) {
            Ok(runner) => self.mode = Mode::Run(runner),
            Err(err) => self.error = Some(format!("{err:#}")),
        }
        Ok(())
    }
}

/// Quotes an argument for display, only where the shell would need it.
fn shell_quote(arg: &str) -> String {
    let safe = |c: char| c.is_ascii_alphanumeric() || "-_./=:@+%,".contains(c);
    if !arg.is_empty() && arg.chars().all(safe) {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        crossterm::event::{KeyCode, KeyEvent},
        widgets::ListState,
    };

    use super::{App, Focus, HostChoice, Input, Mode, Value, default_fields, shell_quote};
    use crate::status::{Hosts, Status};

    fn app_with_inputs(inputs: &[&str]) -> App {
        let inputs: Vec<String> = inputs.iter().map(|name| (*name).to_owned()).collect();
        let mut app = App {
            fields: default_fields(),
            inputs: Vec::new(),
            focus: Focus::Options,
            options_state: ListState::default().with_selected(Some(0)),
            inputs_state: ListState::default(),
            mode: Mode::Form,
            status: Status {
                flake: Err("no flake configured".to_owned()),
                default_host: "laptop".to_owned(),
                upstream: None,
                unpushed: None,
                head: None,
                changes: Vec::new(),
                inputs,
            },
            hosts: Hosts::default(),
            command: String::new(),
            error: None,
            quit: false,
        };
        app.sync_inputs();
        app
    }

    fn app() -> App {
        app_with_inputs(&[])
    }

    fn value_mut(app: &mut App, flag: char) -> &mut Value {
        &mut app
            .fields
            .iter_mut()
            .find(|field| field.flag == flag)
            .expect("known flag")
            .value
    }

    fn switch_on(app: &mut App, flag: char) {
        *value_mut(app, flag) = Value::Toggle(true);
    }

    fn type_text(app: &mut App, flag: char, text: &str) {
        *value_mut(app, flag) = Value::Text(text.to_owned());
    }

    fn select_input(app: &mut App, name: &str) {
        app.inputs
            .iter_mut()
            .find(|input| input.name == name)
            .expect("known input")
            .selected = true;
    }

    fn selected_inputs(app: &App) -> Vec<&str> {
        app.inputs
            .iter()
            .filter(|input| input.selected)
            .map(|input| input.name.as_str())
            .collect()
    }

    #[test]
    fn no_arguments_by_default() {
        assert!(app().args().is_empty());
    }

    #[test]
    fn switched_on_toggles_become_flags_in_field_order() {
        let mut app = app();
        switch_on(&mut app, 'P');
        switch_on(&mut app, 'p');
        assert_eq!(app.args(), ["-p", "-P"]);
    }

    #[test]
    fn text_values_are_trimmed_and_blank_ones_skipped() {
        let mut app = app();
        type_text(&mut app, 'c', "  new kernel  ");
        type_text(&mut app, 'H', "   ");
        assert_eq!(app.args(), ["-c", "new kernel"]);
    }

    #[test]
    fn tilde_is_expanded_only_in_the_flake_path() {
        let mut app = app();
        type_text(&mut app, 'c', "~/comment");
        type_text(&mut app, 'f', "~/nixos");
        assert_eq!(
            app.args(),
            [
                "-c".to_owned(),
                "~/comment".to_owned(),
                "-f".to_owned(),
                narf::flake::expand_tilde("~/nixos"),
            ]
        );
    }

    #[test]
    fn selected_inputs_are_updated_individually() {
        let mut app = app_with_inputs(&["home-manager", "nixpkgs"]);
        select_input(&mut app, "nixpkgs");
        assert_eq!(app.args(), ["-U", "nixpkgs"]);
    }

    #[test]
    fn updating_all_inputs_drops_the_selection() {
        let mut app = app_with_inputs(&["home-manager", "nixpkgs"]);
        select_input(&mut app, "nixpkgs");
        switch_on(&mut app, 'u');
        assert_eq!(app.args(), ["-u"]);
    }

    #[test]
    fn command_quotes_arguments_for_the_shell() {
        let mut app = app();
        switch_on(&mut app, 'y');
        type_text(&mut app, 'c', "it's done");
        app.update_command();
        assert_eq!(app.command, r"narf -y -c 'it'\''s done'");
    }

    #[test]
    fn shell_quote_keeps_safe_arguments() {
        assert_eq!(shell_quote("/etc/nixos"), "/etc/nixos");
        assert_eq!(shell_quote("-U"), "-U");
        assert_eq!(shell_quote("user@host:1+2%,3=4"), "user@host:1+2%,3=4");
    }

    #[test]
    fn shell_quote_wraps_unsafe_and_empty_arguments() {
        assert_eq!(shell_quote("new kernel"), "'new kernel'");
        assert_eq!(shell_quote("~/nixos"), "'~/nixos'");
        assert_eq!(shell_quote(""), "''");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn no_hints_by_default() {
        assert!(app().hints().is_empty());
    }

    #[test]
    fn hints_name_ignored_combinations() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        switch_on(&mut app, 'S');
        switch_on(&mut app, 'd');
        switch_on(&mut app, 'u');
        type_text(&mut app, 'c', "comment");
        select_input(&mut app, "nixpkgs");
        assert_eq!(
            app.hints(),
            [
                "Don't squash has no effect without Push",
                "No commit (and no comment) in dry/build-only mode",
                "Update all inputs overrides the input selection",
            ]
        );
    }

    #[test]
    fn hints_that_push_is_skipped_when_building_only() {
        let mut app = app();
        switch_on(&mut app, 'P');
        switch_on(&mut app, 'b');
        assert_eq!(app.hints(), ["Push is skipped in dry/build-only mode"]);
    }

    #[test]
    fn hints_that_no_rebuild_conflicts_with_rebuild_options() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        switch_on(&mut app, 'n');
        assert!(app.hints().is_empty());
        select_input(&mut app, "nixpkgs");
        assert_eq!(
            app.hints(),
            ["No rebuild can't be combined with update, comment, build only or trace"]
        );
    }

    #[test]
    fn hints_at_a_host_missing_from_the_flake() {
        let mut app = app();
        app.hosts.names = vec!["desktop".to_owned()];
        assert_eq!(
            app.hints(),
            ["Host 'laptop' is not in the flake's nixosConfigurations"]
        );
        type_text(&mut app, 'H', "desktop");
        assert!(app.hints().is_empty());
    }

    #[test]
    fn host_falls_back_to_the_hostname() {
        let mut app = app();
        assert_eq!(app.host(), "laptop");
        type_text(&mut app, 'H', " desktop ");
        assert_eq!(app.host(), "desktop");
    }

    #[test]
    fn host_choices_list_default_hosts_and_custom() {
        let mut app = app();
        app.hosts.names = vec!["desktop".to_owned(), "laptop".to_owned()];
        let choices = app.host_choices();
        assert!(matches!(
            choices.as_slice(),
            [
                HostChoice::Default,
                HostChoice::Host(first),
                HostChoice::Host(second),
                HostChoice::Custom,
            ] if first == "desktop" && second == "laptop"
        ));
    }

    #[test]
    fn picking_a_host_fills_the_host_field() {
        let mut app = app();
        app.hosts.names = vec!["desktop".to_owned()];
        app.mode = Mode::PickHost(ListState::default().with_selected(Some(1)));
        app.on_host_picker_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::Form));
        assert_eq!(app.args(), ["-H", "desktop"]);
    }

    #[test]
    fn picking_the_default_clears_the_host_field() {
        let mut app = app();
        type_text(&mut app, 'H', "desktop");
        app.mode = Mode::PickHost(ListState::default().with_selected(Some(0)));
        app.on_host_picker_key(KeyEvent::from(KeyCode::Enter));
        assert!(app.args().is_empty());
    }

    #[test]
    fn picking_custom_edits_the_host_and_keeps_a_backup() {
        let mut app = app();
        type_text(&mut app, 'H', "desktop");
        app.mode = Mode::PickHost(ListState::default().with_selected(Some(1)));
        app.on_host_picker_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(&app.mode, Mode::Edit { backup } if backup == "desktop"));
    }

    #[test]
    fn host_picker_opens_on_the_current_host() {
        let mut app = app();
        app.hosts.names = vec!["desktop".to_owned(), "laptop".to_owned()];
        type_text(&mut app, 'H', "laptop");
        app.open_host_picker();
        assert!(matches!(&app.mode, Mode::PickHost(state) if state.selected() == Some(2)));
    }

    #[test]
    fn syncing_inputs_keeps_the_selection_of_remaining_inputs() {
        let mut app = app_with_inputs(&["home-manager", "nixpkgs", "sops-nix"]);
        select_input(&mut app, "nixpkgs");
        select_input(&mut app, "sops-nix");
        app.inputs_state.select(Some(2));
        app.status.inputs = vec!["disko".to_owned(), "nixpkgs".to_owned()];
        app.sync_inputs();
        assert_eq!(selected_inputs(&app), ["nixpkgs"]);
        assert_eq!(app.inputs_state.selected(), Some(1));
    }

    #[test]
    fn syncing_without_inputs_clears_the_cursor() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        app.status.inputs.clear();
        app.sync_inputs();
        assert!(app.inputs.is_empty());
        assert_eq!(app.inputs_state.selected(), None);
    }

    #[test]
    fn selection_is_clamped_to_the_lists() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        app.options_state.select(Some(99));
        app.inputs_state.select(Some(99));
        app.clamp_selection();
        assert_eq!(app.options_state.selected(), Some(app.fields.len() - 1));
        assert_eq!(app.inputs_state.selected(), Some(0));
    }

    #[test]
    fn activating_toggles_options_and_inputs() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        app.activate();
        assert_eq!(app.args(), ["-p"]);
        app.focus = Focus::Inputs;
        app.inputs_state.select(Some(0));
        app.activate();
        assert_eq!(app.args(), ["-p", "-U", "nixpkgs"]);
    }

    #[test]
    fn flag_keys_toggle_their_option_from_the_inputs() {
        let mut app = app_with_inputs(&["nixpkgs"]);
        app.focus = Focus::Inputs;
        app.activate_flag('P');
        assert_eq!(app.args(), ["-P"]);
        assert!(app.focus == Focus::Options);
        assert_eq!(app.selected_field().map(|f| f.flag), Some('P'));
        app.activate_flag('P');
        assert!(app.args().is_empty());
    }

    #[test]
    fn flag_keys_edit_text_options_and_pick_the_host() {
        let mut app = app();
        type_text(&mut app, 'c', "kernel");
        app.activate_flag('c');
        assert!(matches!(&app.mode, Mode::Edit { backup } if backup == "kernel"));
        app.mode = Mode::Form;
        app.activate_flag('H');
        assert!(matches!(app.mode, Mode::PickHost(_)));
    }

    #[test]
    fn unknown_flag_keys_change_nothing() {
        let mut app = app();
        app.activate_flag('x');
        assert!(app.args().is_empty());
        assert!(matches!(app.mode, Mode::Form));
    }

    #[test]
    fn flag_keys_do_not_collide_with_form_keys() {
        for field in default_fields() {
            assert!(!"qjkhlgGrR".contains(field.flag), "-{}", field.flag);
        }
    }

    #[test]
    fn inputs_start_unselected() {
        let app = app_with_inputs(&["nixpkgs"]);
        assert!(matches!(
            app.inputs.as_slice(),
            [Input { name, selected: false }] if name == "nixpkgs"
        ));
    }
}
