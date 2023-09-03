use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

pub type DebugLevel = u8;
pub const DEBUG_NONE: u8 = 0;
pub const DEBUG_INFO: u8 = 1;
pub const DEBUG_VERBOSE: u8 = 2;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct KataruInterface {
    bookmark_path: String,
    default_passage: String,
    debug_level: u8,
    runner: Option<Runner<'static>>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for KataruInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            bookmark_path: "".to_string(),
            default_passage: "".to_string(),
            runner: None,
            debug_level: DEBUG_NONE,
            base,
        }
    }
}

fn serde_to_json<T: serde::Serialize>(value: &T) -> Variant {
    Variant::from(serde_json::to_string(value).unwrap())
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
        default_passage: GodotString,
        debug_level: DebugLevel,
    ) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru.init()");
        }
        if let Err(err) = self.try_init(
            story_src_path.to_string(),
            story_path.to_string(),
            bookmark_path.to_string(),
            default_passage.to_string(),
            debug_level,
        ) {
            godot_error!("Kataru.init(): {}", err);
        }
    }
    fn try_init(
        &mut self,
        story_src_path: String,
        story_path: String,
        bookmark_path: String,
        default_passage: String,
        debug_level: DebugLevel,
    ) -> Result<()> {
        self.bookmark_path = bookmark_path.to_string();
        self.default_passage = default_passage.to_string();
        self.debug_level = debug_level;

        // Validate and compile if a source path is specified.
        if !story_src_path.is_empty() {
            let story = Story::load(story_src_path)?;
            let mut bookmark = Bookmark::load_or_default(
                &self.bookmark_path,
                &story,
                self.default_passage.clone(),
            )?;
            Validator::new(&story, &mut bookmark).validate()?;
            story.save(&story_path)?;
            if self.debug_level >= DEBUG_INFO {
                godot_print!("Kataru.init(): story compiled to {}", story_path)
            }
        }

        // Load runner from compiled story.
        let story = Story::load(&story_path)?;
        let bookmark =
            Bookmark::load_or_default(&self.bookmark_path, &story, self.default_passage.clone())?;
        self.runner = Some(Runner::new(bookmark, story, false)?);
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
            godot_error!("Kataru.next({}): {}", input, err);
        }
    }
    fn try_next(&mut self, input: String) -> Result<()> {
        if let Some(runner) = &mut self.runner {
            let line = runner.next(&input)?;

            if self.debug_level >= DEBUG_VERBOSE {
                godot_print!("Kataru.next({}): Bookmark {:#?}", input, runner.bookmark);
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
            Err(error!("Kataru uninitialized."))
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
                    Variant::from(Array::from(choices.choices.as_slice())),
                    Variant::from(choices.timeout),
                ],
            ),
            Line::Command(command) => self.base.emit_signal(
                Self::COMMAND.into(),
                &[
                    Variant::from(command.name.to_string()),
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


    #[signal]
    fn loaded();
    const LOADED: &str = "dialogue";

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
}
