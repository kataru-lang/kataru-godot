use std::default;

use godot::prelude::*;
use kataru::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

#[derive(GodotClass)]
pub struct KataruDialogue {
    pub name: String,
    pub text: String,
}
impl From<Dialogue> for KataruDialogue {
    fn from(value: Dialogue) -> Self {
        Self {
            name: value.name,
            text: value.text,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Kataru {
    pub story_path: String,
    pub bookmark_path: String,

    story: Story,
    bookmark: Bookmark,
    runner: Option<Runner<'static>>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for Kataru {
    fn init(base: Base<Node>) -> Self {
        godot_print!("Welcome to Kataru!"); // Prints to the Godot console

        Self {
            story_path: "".to_string(),
            bookmark_path: "".to_string(),
            story: Story::default(),
            bookmark: Bookmark::default(),
            runner: None,
            base,
        }
    }
}

// Forces lifetime extension.
// WARNING: this requires manual validation of lifetimes!
unsafe fn extend_lifetime<'a>(r: Runner<'a>) -> Runner<'static> {
    std::mem::transmute::<Runner<'a>, Runner<'static>>(r)
}

#[godot_api]
impl Kataru {
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
    ) {
        godot_print!("Load story!"); // Prints to the Godot console

        self.story_path = story_path.to_string();
        self.bookmark_path = bookmark_path.to_string();
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
            godot_print!("{:#?}", runner.bookmark);
            let line = runner.next(&input.to_string()).unwrap().clone();
            godot_print!("{:#?}", line);
            match &line {
                Line::Dialogue(dialogue) => {
                    self.base.emit_signal(
                        "dialogue".into(),
                        &[
                            Variant::from(dialogue.name.clone()),
                            Variant::from(dialogue.text.clone()),
                        ],
                    );
                }
                Line::Choices(_choices) => {}
                Line::Command(_command) => {}
                Line::Input(_input_cmd) => {}
                Line::InvalidChoice => {}
                Line::End => {}
            }
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
    fn dialogue(name: GodotString, text: GodotString);

    #[signal]
    fn choices();

    #[signal]
    fn input();
}
