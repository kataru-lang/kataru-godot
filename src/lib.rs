use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

// Forces lifetime extension.
// WARNING: this requires manual validation of lifetimes!
unsafe fn extend_lifetime<'a>(r: Runner<'a>) -> Runner<'static> {
    std::mem::transmute::<Runner<'a>, Runner<'static>>(r)
}

type DebugLevel = u8;
const DEBUG_NONE: u8 = 0;
const DEBUG_INFO: u8 = 1;
const DEBUG_VERBOSE: u8 = 2;

struct Engine {
    pub story: Story,
    pub bookmark: Bookmark,
    runner: Option<Runner<'static>>,
}
impl Engine {
    pub fn new(story_path: &str, bookmark_path: &str, default_passage: String) -> Result<Self> {
        godot_print!("new engine");
        let story = Story::load(story_path)?;
        let mut result = Self {
            story,
            bookmark: Bookmark::default(),
            runner: None,
        };

        godot_print!("init bookmark");

        result.bookmark = Bookmark::load_or_default(bookmark_path, &result.story, default_passage)?;

        godot_print!("init runner");
        // Runner holds references to the story and bookmark. We ensure that whenever their references
        // are invalidated, we will reconstruct the runner.
        unsafe {
            result.runner = Some(extend_lifetime(Runner::new(
                &mut result.bookmark,
                &result.story,
            )?));
        }
        godot_print!("Engine built! Story: {:?}", result.story);
        Ok(result)
    }

    pub fn goto_passage(&mut self, passage: String) -> Result<()> {
        self.runner.as_mut().unwrap().bookmark.set_passage(passage);
        self.runner.as_mut().unwrap().bookmark.set_line(0);
        self.runner.as_mut().unwrap().bookmark.stack.clear();
        self.runner.as_mut().unwrap().goto()
    }

    pub fn run_passage(&mut self, passage: String) -> Result<Line> {
        self.goto_passage(passage)?;
        self.next("")
    }

    pub fn next(&mut self, input: &str) -> Result<Line> {
        self.runner.as_mut().unwrap().next(input)
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct KataruInterface {
    pub story_path: String,
    pub bookmark_path: String,

    default_passage: String,
    debug_level: u8,
    engine: Option<Engine>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for KataruInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            story_path: "".to_string(),
            bookmark_path: "".to_string(),
            default_passage: "".to_string(),
            engine: None,
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
    #[func]
    pub fn init(
        &mut self,
        story_path: GodotString,
        bookmark_path: GodotString,
        default_passage: GodotString,
        debug_level: DebugLevel,
    ) {
        self.story_path = story_path.to_string();
        self.bookmark_path = bookmark_path.to_string();
        self.default_passage = default_passage.to_string();
        self.debug_level = debug_level;

        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru initialized.")
        }
    }

    fn engine(&mut self) -> Result<&mut Engine> {
        if let Some(engine) = self.engine.as_mut() {
            Ok(engine)
        } else {
            let err = error!("Kataru was not initialized. Call Kataru.load().");
            Err(err)
        }
    }

    #[func]
    pub fn load(&mut self) {
        godot_print!("Load from rust...");
        match Engine::new(
            &self.story_path,
            &self.bookmark_path,
            self.default_passage.clone(),
        ) {
            Ok(engine) => self.engine = Some(engine),
            Err(err) => self.fatal_error(err),
        }
        self.base.emit_signal(Self::LOADED.into(), &[]);
    }

    #[func]
    pub fn compile(&mut self, story_path: GodotString) {
        if let Err(err) = self.try_compile(&story_path.to_string()) {
            self.fatal_error(err)
        }
    }
    pub fn try_compile(&mut self, story_path: &str) -> Result<()> {
        let story = Story::load(story_path)?;
        let mut bookmark =
            Bookmark::load_or_default(&self.bookmark_path, &story, self.default_passage.clone())?;
        Validator::new(&story, &mut bookmark).validate()?;
        story.save(&self.story_path)?;
        Ok(())
    }

    #[func]
    pub fn next(&mut self, input: GodotString) {
        godot_print!("Next in rust...");
        if let Err(err) = self.try_next(input.to_string()) {
            self.fatal_error(err)
        }
    }
    fn try_next(&mut self, input: String) -> Result<()> {
        godot_print!("Try next...");
        let line = self.engine()?.next(&input)?;
        godot_print!("emit_line_signal...");
        self.emit_line_signal(&line);
        Ok(())
    }

    fn emit_line_signal(&mut self, line: &Line) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("{:#?}", line);
        }
        if self.debug_level >= DEBUG_VERBOSE {
            godot_print!("{:#?}", self.engine().unwrap().bookmark);
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

    #[func]
    pub fn goto_passage(&mut self, passage: GodotString) {
        if let Err(err) = self.try_goto_passage(passage.to_string()) {
            self.fatal_error(err)
        }
    }
    fn try_goto_passage(&mut self, passage: String) -> Result<()> {
        self.engine()?.goto_passage(passage)
    }

    #[func]
    pub fn run_passage(&mut self, passage: GodotString) {
        if let Err(err) = self.try_run_passage(passage.to_string()) {
            self.fatal_error(err)
        }
    }
    fn try_run_passage(&mut self, passage: String) -> Result<()> {
        self.engine()?.run_passage(passage)?;
        Ok(())
    }

    #[func]
    pub fn run_passage_until_choice(&mut self, passage: GodotString) {
        if let Err(err) = self.try_run_passage_until_choice(passage.to_string()) {
            self.fatal_error(err)
        }
    }
    fn try_run_passage_until_choice(&mut self, passage: String) -> Result<()> {
        self.try_goto_passage(passage)?;
        loop {
            match self.engine()?.next("")? {
                Line::Choices(_) | Line::End => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    #[func]
    pub fn exit(&mut self) {
        self.base.emit_signal(Self::END.into(), &[]);
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

    #[signal]
    fn fatal(message: GodotString);
    const FATAL: &str = "fatal";

    fn fatal_error(&mut self, err: Error) {
        godot_error!("Error: {}", err);
        self.base
            .emit_signal(Self::FATAL.into(), &[Variant::from(err.to_string())]);
    }
}
