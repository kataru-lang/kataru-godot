use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

// Debug levels.
const DEBUG_INFO: u8 = 1;
const DEBUG_VERBOSE: u8 = 2;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct KataruInterface {
    pub story_path: String,
    pub bookmark_path: String,

    story: Story,
    bookmark: Bookmark,
    runner: Option<Runner<'static>>,
    default_passage: String,
    debug_level: u8,
    line: Line,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for KataruInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            story_path: "".to_string(),
            bookmark_path: "".to_string(),
            story: Story::default(),
            bookmark: Bookmark::default(),
            runner: None,
            default_passage: "".to_string(),
            debug_level: 0,
            line: Line::End,
            base,
        }
    }
}

// Forces lifetime extension.
// WARNING: this requires manual validation of lifetimes!
unsafe fn extend_lifetime<'a>(r: Runner<'a>) -> Runner<'static> {
    std::mem::transmute::<Runner<'a>, Runner<'static>>(r)
}

fn serde_to_variant<T: serde::Serialize>(value: &T) -> Variant {
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
        debug_level: u8,
    ) {
        self.story_path = story_path.to_string();
        self.bookmark_path = bookmark_path.to_string();
        self.default_passage = default_passage.to_string();
        self.debug_level = debug_level;

        if self.debug_level >= DEBUG_INFO {
            godot_print!("Kataru initialized.")
        }
    }

    fn try_load_bookmark(&mut self) -> Result<Bookmark> {
        match Bookmark::load(&self.bookmark_path) {
            Ok(bookmark) => Ok(bookmark),
            Err(_err) => {
                if self.debug_level >= DEBUG_INFO {
                    godot_print!(
                        "Bookmark did not exist or was malformed. Createing a new bookmark at {}",
                        self.bookmark_path
                    );
                }
                let mut bookmark = Bookmark::default();
                bookmark.init_state(&self.story);
                bookmark.set_passage(self.default_passage.clone());
                bookmark.save(&self.bookmark_path)?;
                Ok(bookmark)
            }
        }
    }

    fn try_load(&mut self) -> Result<()> {
        self.story = Story::load(&self.story_path)?;
        self.bookmark = self.try_load_bookmark()?;
        // Runner holds references to the story and bookmark. We ensure that whenveer their references
        // are invalidated, we will reconstruct the runner.
        unsafe {
            self.runner = Some(extend_lifetime(Runner::new(
                &mut self.bookmark,
                &self.story,
            )?));
        }
        Ok(())
    }

    #[func]
    fn load(&mut self) {
        if self.debug_level >= DEBUG_INFO {
            godot_print!("Loading Kataru story and initializing runner.");
        }
        if let Err(err) = self.try_load() {
            godot_error!("{}", err);
        }
        self.base.emit_signal("loaded".into(), &[]);
    }

    pub fn validate(&mut self) -> Result<()> {
        Validator::new(&self.story, &mut self.bookmark).validate()
    }

    fn try_compile(&mut self, story_path: String) -> Result<()> {
        self.story = Story::load(story_path)?;
        self.bookmark = self.try_load_bookmark()?;
        self.validate()?;
        self.story.save(&self.story_path)?;
        Ok(())
    }
    #[func] pub fn compile(&mut self, 
        story_path: GodotString) {
        godot_print!("Kataru Compile:");
        if let Err(err) = self.try_compile(story_path.to_string()) {
            godot_error!("Compile error: {}", err)
         }
    }

    #[func]
    pub fn next(&mut self, input: GodotString) {
        if let Some(runner) = &mut self.runner {
            if self.debug_level > DEBUG_VERBOSE {
                godot_print!("{:#?}", runner.bookmark);
            }
            self.line = runner.next(&input.to_string()).unwrap().clone();
            if self.debug_level > DEBUG_INFO {
                godot_print!("{:#?}", self.line);
            }
            match &self.line {
                Line::Dialogue(dialogue) => self
                    .base
                    .emit_signal("dialogue_json".into(), &[serde_to_variant(dialogue)]),
                Line::Choices(choices) => self
                    .base
                    .emit_signal("choices_json".into(), &[serde_to_variant(choices)]),
                Line::Command(command) => self
                    .base
                    .emit_signal("command_json".into(), &[serde_to_variant(command)]),
                Line::Input(input_cmd) => self
                    .base
                    .emit_signal("input_json".into(), &[serde_to_variant(input_cmd)]),
                Line::InvalidChoice => self.base.emit_signal("invalid_choice".into(), &[]),
                Line::End => self.base.emit_signal("end".into(), &[]),
            };
        } else {
            godot_error!("Attempted to call Kataru.next() before Kataru.load().")
        }
    }

    #[func]
    pub fn run_passage(&mut self, passage: GodotString) {
        if let Some(runner) = &mut self.runner {
            runner.bookmark.set_passage(passage.to_string());
            runner.bookmark.set_line(0);
            runner.bookmark.stack.clear();
            if let Err(err) = runner.goto() {
                godot_error!("{}", err);
            }
        }
        self.next("".into())
    }

    #[func]
    pub fn run_passage_until_choice(&mut self, passage: GodotString) {
        self.run_passage(passage);
        loop {
            match self.line {
                Line::Choices(_) | Line::End => break,
                _ => self.next("".into()),
            }
        }
    }

    #[func]
    pub fn exit(&mut self) {
        self.base.emit_signal("end".into(), &[]);
    }

    #[signal]
    fn loaded();

    #[signal]
    fn dialogue_json(dialogue: GodotString);

    #[signal]
    fn choices_json(choices: GodotString);

    #[signal]
    fn command_json(command: GodotString);

    #[signal]
    fn input_json(input: GodotString);

    #[signal]
    fn invalid_choice();

    #[signal]
    fn end();
}
