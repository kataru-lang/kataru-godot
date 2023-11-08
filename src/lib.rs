use std::path::{Path, PathBuf};

use glob::glob;
use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

pub type DebugLevel = u8;
pub const DEBUG_NONE: u8 = 0;
pub const DEBUG_INFO: u8 = 1;
pub const DEBUG_VERBOSE: u8 = 2;
mod codegen;

fn last_modified_time(path: &PathBuf) -> Option<std::time::SystemTime> {
    glob(path.to_str()?)
        .expect("Couldn't access local directory")
        .flatten() // Remove failed
        .filter(|f| f.metadata().is_ok() && !f.metadata().unwrap().is_dir()) // Filter out directories (only consider files)
        .map(|f| f.metadata().unwrap().modified().unwrap())
        // Get the most recently modified file
        .max()
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct KataruInterface {
    story_src_path: PathBuf,
    story_path: PathBuf,
    bookmark_path: PathBuf,
    codegen_path: PathBuf,
    default_passage: String,
    debug_level: u8,
    runner: Option<Runner>,
    watch_dir: Option<PathBuf>,
    watch_poll_time: f64,
    watch_poll_interval: f64,
    modified_time: Option<std::time::SystemTime>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for KataruInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            story_src_path: "".into(),
            story_path: "".into(),
            bookmark_path: "".into(),
            codegen_path: "".into(),
            default_passage: "".to_string(),
            runner: None,
            watch_dir: None,
            watch_poll_time: 0.0,
            watch_poll_interval: 0.0,
            modified_time: None,
            debug_level: DEBUG_NONE,
            base,
        }
    }
}

fn serde_to_json<T: serde::Serialize>(value: &T) -> Variant {
    Variant::from(serde_json::to_string(value).unwrap())
}

/// Logs a fatal assertion by sending a signal to Godot.
/// Can only be called from a struct that implements NodeVirtual that has a FATAL signal set up.
#[macro_export]
macro_rules! godot_fatal {
    ($self:ident, $fmt:literal $(, $args:expr)* $(,)?) => {
        $self.base.emit_signal(Self::FATAL.into(), &[Variant::from(format!($fmt $(, $args)*))]);
    };
}

#[godot_api]
impl KataruInterface {
    /// Initialize kataru with the given path settings.
    /// This *must* be called before any other methods are called.
    /// If `story_src_path` is specified, compile the story to in `story_src_path` to `story_path`.
    #[func]
    pub fn init(
        &mut self,
        story_src_path: GodotString,
        story_path: GodotString,
        bookmark_path: GodotString,
        codegen_path: GodotString,
        default_passage: GodotString,
        debug_level: DebugLevel,
        watch_poll_interval: f64,
    ) {
        self.story_src_path = story_src_path.to_string().into();
        self.story_path = story_path.to_string().into();
        self.bookmark_path = bookmark_path.to_string().into();
        self.codegen_path = codegen_path.to_string().into();
        self.default_passage = default_passage.into();
        self.debug_level = debug_level;
        self.watch_poll_interval = watch_poll_interval;

        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.init()");
        }
        if let Err(err) = self.try_init() {
            godot_fatal!(self, "Kataru.init(): {}", err);
        }
    }
    fn try_init(&mut self) -> Result<()> {
        // Validate and compile if a source path is specified.
        let story = if !self.story_src_path.as_os_str().is_empty() {
            let story = Story::load(&self.story_src_path)?;
            let mut bookmark = Bookmark::load_or_default(
                &self.bookmark_path,
                &story,
                self.default_passage.clone(),
            )?;
            Validator::new(&story, &mut bookmark).validate()?;
            story.save(&self.story_path)?;
            if self.debug_level >= DEBUG_INFO {
                godot_print!(
                    "Kataru.init(): story compiled to {}",
                    self.story_path.display()
                )
            }

            // Generate constants if enabled.
            if !self.codegen_path.as_os_str().is_empty() {
                codegen::try_codegen_consts(&self.codegen_path, &story)?;

                if self.debug_level >= DEBUG_INFO {
                    godot_print!(
                        "Kataru.init(): constants files generated to {}",
                        self.codegen_path.display()
                    )
                }
            }

            self.watch_dir = Some(Path::new(&self.story_src_path).join("**").join("*"));
            self.modified_time = last_modified_time(self.watch_dir.as_ref().unwrap());

            story
        } else {
            Story::load(&self.story_path)?
        };

        // Load runner from compiled story.
        let bookmark =
            Bookmark::load_or_default(&self.bookmark_path, &story, self.default_passage.clone())?;
        self.runner = Some(Runner::init(bookmark, story, false)?);
        self.base.emit_signal(Self::LOADED.into(), &[]);
        Ok(())
    }

    /// Run the next line of dialogue.
    #[func]
    pub fn next(&mut self, input: GodotString) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.next('{}')", input);
        }
        if let Err(err) = self.try_next(input.to_string()) {
            godot_error!("Kataru.next('{}'): {}", input, err);
        }
    }
    fn try_next(&mut self, input: String) -> Result<()> {
        if let Some(runner) = &mut self.runner {
            let line = runner.next(&input)?;

            if self.debug_level >= DEBUG_VERBOSE {
                godot_print!("Kataru.next('{}'): {:#?}", input, runner.bookmark());
            }
            self.emit_line_signal(&line);
        }
        Ok(())
    }

    /// Go to the given `passage`, but do not run the first line.
    #[func]
    pub fn goto(&mut self, passage: GodotString) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.goto({})", passage);
        }
        if let Err(err) = self.try_goto(passage.to_string()) {
            godot_error!("Kataru.goto({}): {}", passage, err);
        }
    }
    fn try_goto(&mut self, passage: String) -> Result<()> {
        if let Some(runner) = self.runner.as_mut() {
            runner.goto(passage)
        } else {
            godot_fatal!(self, "Kataru was not initialized before call to goto.");
            Err(error!("Kataru uninitialized."))
        }
    }

    /// Run the first line in the given `passage`.
    #[func]
    pub fn run(&mut self, passage: GodotString) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.run({})", passage);
        }
        if let Err(err) = self.try_run(passage.to_string()) {
            godot_error!("Kataru.run(): {}", err)
        }
    }
    fn try_run(&mut self, passage: String) -> Result<()> {
        if let Some(runner) = self.runner.as_mut() {
            runner.run(passage)?;
            Ok(())
        } else {
            godot_fatal!(self, "Kataru was not initialized before call to run.");
            Err(error!("Kataru uninitialized."))
        }
    }

    // Transforms a command name into the normalized version.
    // This is used for lookups to command adapters in GDScript.
    fn get_normalized_command(command: &str) -> String {
        // Get positions of character name.
        let mut colon: usize = 0;
        let mut dot: usize = 0;
        for (i, c) in command.chars().enumerate() {
            match c {
                ':' => colon = i,
                '.' => dot = i,
                _ => {}
            }
        }
        match (colon, dot) {
            (_, 0) => command.to_string(),
            (0, _) => format!("$character.{}", &command[dot + 1..]),
            (_, _) => format!("{}:$character.{}", &command[0..colon], &command[dot + 1..]),
        }
    }

    /// Run the current passage until a choice is encountered.
    #[func]
    pub fn run_until_choice(&mut self, passage: GodotString) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.run_until_choice({})", passage);
        }
        if let Err(err) = self.try_run_until_choice(passage.to_string()) {
            godot_error!("Kataru.run_until_choice({}): {}", passage, err)
        }
    }
    fn try_run_until_choice(&mut self, passage: String) -> Result<()> {
        self.try_goto(passage)?;
        if let Some(runner) = self.runner.as_mut() {
            loop {
                match runner.next("")? {
                    Line::Choices(_) | Line::End => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        } else {
            godot_fatal!(
                self,
                "Kataru was not initialized before call to run_until_choice."
            );
            Err(error!("Kataru uninitialized."))
        }
    }

    /// Exit the current dialogue passage.
    #[func]
    pub fn exit(&mut self) {
        self.base.emit_signal(Self::END.into(), &[]);
    }

    /// Emit a signal for the given line so GDScript can interact with it.
    fn emit_line_signal(&mut self, line: &Line) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.emit_line_signal({:#?})", line);
        }
        match line {
            Line::Dialogue(dialogue) => self.base.emit_signal(
                Self::DIALOGUE.into(),
                &[
                    Variant::from(dialogue.name.to_string()),
                    Variant::from(dialogue.text.to_string()),
                    Variant::from(serde_to_json(&dialogue.attributes)),
                ],
            ),
            Line::Choices(choices) => self.base.emit_signal(
                Self::CHOICES.into(),
                &[
                    Variant::from(Array::<GodotString>::from_iter(
                        choices.choices.iter().map(|e| e.into()),
                    )),
                    Variant::from(choices.timeout),
                ],
            ),
            Line::Command(command) => self.base.emit_signal(
                Self::COMMAND.into(),
                &[
                    Variant::from(command.name.to_string()),
                    Variant::from(Self::get_normalized_command(&command.name)),
                    serde_to_json(&command.params),
                ],
            ),
            Line::Input(input_cmd) => self.base.emit_signal(
                Self::INPUT_COMMAND.into(),
                &[
                    Variant::from(Dictionary::from(&input_cmd.input)),
                    Variant::from(input_cmd.timeout),
                ],
            ),
            Line::InvalidChoice => self.base.emit_signal(Self::INVALID_CHOICE.into(), &[]),
            Line::End => self.base.emit_signal(Self::END.into(), &[]),
        };
    }

    fn watch_dir_changed(&mut self) -> bool {
        if let Some(watch_dir) = &self.watch_dir {
            let old_modified_time = self.modified_time;
            self.modified_time = last_modified_time(watch_dir);
            match (self.modified_time, old_modified_time) {
                (Some(current), Some(previous)) => current != previous,
                (Some(_), None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    #[func]
    fn watch_story_dir(&mut self, delta: f64) {
        // File watcher
        self.watch_poll_time += delta;
        if self.watch_poll_time < self.watch_poll_interval {
            return;
        }
        self.watch_poll_time -= self.watch_poll_interval;
        if self.watch_dir_changed() {
            if self.debug_level >= DEBUG_INFO {
                godot_print!("Kataru story directory changed.")
            }
            if let Err(err) = self.try_init() {
                godot_fatal!(self, "Kataru.init(): {}", err);
            }
        }
    }

    #[signal]
    fn loaded();
    const LOADED: &str = "loaded";

    #[signal]
    fn dialogue(char_name: GodotString, text: GodotString, attributes: GodotString);
    const DIALOGUE: &str = "dialogue";

    #[signal]
    fn choices(choices: Array<GodotString>, timeout: f64);
    const CHOICES: &str = "choices";

    #[signal]
    fn command(cmd_name: GodotString, params: GodotString);
    const COMMAND: &str = "command";

    #[signal]
    fn input_command(inputs: Dictionary, timeout: f64);
    const INPUT_COMMAND: &str = "input_command";

    #[signal]
    fn invalid_choice();
    const INVALID_CHOICE: &str = "invalid_choice";

    #[signal]
    fn end();
    const END: &str = "end";

    #[signal]
    fn fatal(message: GodotString);
    const FATAL: &str = "fatal";
}
