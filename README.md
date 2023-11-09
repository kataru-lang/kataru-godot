<img src="https://kataru-lang.github.io/_media/logo.svg" alt="Yarn Spinner logo" width="100px;" align="left">

# Kataru 「カタル」Godot Bindings

![rust workflow](https://github.com/katsutoshii/kataru/actions/workflows/rust.yml/badge.svg)

Kataru 「カタル」is a system for interactive dialogue based on YAML, parsed in Rust.

Kataru is similar to [Twine](http://twinery.org/) and [Yarn Spinner](http://yarnspinner.dev) except with more support for organizing passages and sharing common functionality across multiple characters.

```yml
---
# Define what namespace this file is in.
namespace: global

state:
  coffee: 0
  $passage.completed: 0

# Configure the scene. List your characters, commands, etc.
characters:
  May:
  June:

commands:
  Wait:
    duration: 0.3

  $character.SetAnimatorTrigger:
    clip: ""

onExit:
  set:
    $passage.completed +: 1
---
Start:
  - May: Welcome to my story!
  - June: Want a coffee?
  - choices:
      Yes: YesCoffee
      No: NoCoffee

YesCoffee:
  - May: Yeah, thanks!
  - set:
      $coffee +: 1
  - May.SetAnimatorTrigger: ["drinkcoffee"]
  - call: End

NoCoffee:
  - May: No thanks.
  - Wait: { duration: 1 }
  - June: Want to end this story?
  - call: End

End:
  - May: The end!
```

## Features

- Simple and lightweight
- Organize dialogue, state, characters, and commands into local namespaces
- Character-specific commands
- Syntax highlighting and Unity integration

As well as conditionals, variables, and everything else you expect in a dialogue language.

## Getting Started

Godot Asset Library submission in-progress.

Until the addon is available there, install as a git submodule in your Godot project.

1. Run `git submodule add https://github.com/kataru-lang/kataru-godot addons/kataru` to install the plugin.
1. In the Godot Editor, under `Project` -> `Project Settings` -> `Plugins` enable the `Kataru` plugin.
1. By default, the plugin will create a template story in `res://kataru/story`. If you use a different folder or not generate the template, create your own directory and configure the `STORY_PATH` in [`kataru.gd`](kataru.gd).

### Recipes

#### Progress the dialogue

Call `Kataru.run(<passage>)` on the passage you want to run to start the dialogue and run the first line.
Passage name constants are auto-generated in `Kataru.Passages`.

Call `Kataru.next(<input>)` to go to the next line.

#### Listening to dialogue events

To listen to dialogue events, bind to `Kataru` class' signals.

For example:
```py
func _on_choices(choices: Array, _timeout: float):
    pass

func _on_dialogue(char_name: String, text: String, attributes: Array):
    pass

func _ready():
	Kataru.dialogue.connect(self._on_dialogue)
	Kataru.choices.connect(self._on_choices)
```

NOTE: Make sure you connect to the events _before_ you call `Kataru.next()`.

### Commands

In Kataru, arbitrary functions can be called using `Commands`.
To register a global command, use:

```py
func my_custom_command(clip: String):
	pass

func _on_ready():
    Kataru.register(
		self.my_custom_command,
        Kataru.Commands.my_custom_command
	)
```

For a command on a specific `Character` (similar to an instance method), use:

```py
func my_char_command(clip: String):
    pass

func _on_ready():
    Kataru.register(
		self.my_char_command,
        Kataru.Commands.character_my_char_command,
        self.kataru_name
	)
```

Note that the `Kataru.Commands` constant will prefix character specific commands with `character_`.

### Notes and caveats

- Do NOT open the story YAML files in Godot, it will try to autoformat them incorrectly.
- Constant files are generated from reading your story file. These can be used for creating dropdown menus for your scripts (called PROPERTY lists in Godot), but unfortunately these will only be refreshed in the editor has been restarted.
- Typed arrays don't work on callback signatures, e.g. you can only specify `Array` and not `Array[Dictionary]`.

## Getting Help

For bugs or feature requests, file an issue. For other questions, contact kataru-dev@gmail.com.

## License

Kataru is licensed under the [MIT License](LICENSE). Credit is appreciated but not required.

---

Made by [Josiah Putman](https://github.com/Katsutoshii) with help from [Angela He](https://github.com/zephyo).
