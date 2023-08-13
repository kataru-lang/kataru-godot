use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

// Debug levels.
const DEBUG_INFO: u8 = 1;
const DEBUG_VERBOSE: u8 = 2;

#[derive(GodotClass)]
pub struct KataruDialogue {
    pub name: String,
    pub text: String,
}
impl From<&Dialogue> for KataruDialogue {
    fn from(value: &Dialogue) -> Self {
        Self {
            name: value.name.clone(),
            text: value.text.clone(),
        }
    }
}

#[derive(GodotClass)]
pub struct KataruChoices {
    pub choices: Vec<String>,
    pub timeout: f64,
}
impl From<&Choices> for KataruChoices {
    fn from(value: &Choices) -> Self {
        Self {
            choices: value.choices.clone(),
            timeout: value.timeout,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct KataruInterface {
    pub story_path: String,
    pub bookmark_path: String,

    story: Story,
    bookmark: Bookmark,
    runner: Option<Runner<'static>>,
    debug_level: u8,

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
            debug_level: 0,
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
    fn try_load(&mut self, default_passage: String) -> Result<()> {
        self.story = Story::load(&self.story_path)?;
        self.bookmark = match Bookmark::load(&self.bookmark_path) {
            Ok(bookmark) => bookmark,
            Err(_err) => {
                let mut bookmark = Bookmark::default();
                bookmark.init_state(&self.story);
                bookmark.set_passage(default_passage);
                bookmark.save(&self.bookmark_path)?;
                bookmark
            }
        };
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
    pub fn load(
        &mut self,
        story_path: GodotString,
        bookmark_path: GodotString,
        default_passage: GodotString,
        debug_level: u8,
    ) {
        self.story_path = story_path.to_string();
        self.bookmark_path = bookmark_path.to_string();
        self.debug_level = debug_level;

        if self.debug_level >= DEBUG_INFO {
            godot_print!("Loading Kataru...")
        }

        if let Err(err) = self.try_load(default_passage.to_string()) {
            godot_error!("{}", err)
        }

        self.base.emit_signal("loaded".into(), &[]);
    }

    #[func]
    pub fn validate(&mut self) {
        match Validator::new(&self.story, &mut self.bookmark).validate() {
            Err(e) => {
                godot_error!("{}", format!("{}", e));
            }
            Ok(_) => {
                godot_print!("{}", "Validated story successfully.");
            }
        }
    }

    #[func]
    pub fn next(&mut self, input: GodotString) {
        if let Some(runner) = &mut self.runner {
            if self.debug_level > DEBUG_VERBOSE {
                godot_print!("{:#?}", runner.bookmark);
            }
            let line = runner.next(&input.to_string()).unwrap().clone();
            if self.debug_level > DEBUG_INFO {
                godot_print!("{:#?}", line);
            }
            match &line {
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
        }
    }

    #[func]
    pub fn goto_passage(&mut self, passage: GodotString) {
        if let Some(runner) = &mut self.runner {
            runner.bookmark.set_passage(passage.to_string());
            runner.bookmark.set_line(0);
            runner.bookmark.stack.clear();
            if let Err(err) = runner.goto() {
                godot_error!("{}", err)
            }
        }
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
