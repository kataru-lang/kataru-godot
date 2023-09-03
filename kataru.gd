# ------------------------------------------------------------------------------
# Kataru - the simple YAML-based dialogue engine.
# ------------------------------------------------------------------------------
#
# This script is intended to be used as an Autoload:
# ```toml
# [autoload]
#
# Kataru="*res://kataru-godot/kataru.gd"
# ```
@tool
extends Node

const STORY_PATH = "res://kataru-story"
const COMPILED_STORY_PATH = "res://kataru-story.bin"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"
const DEBUG_LEVEL = 2

# Interface with Rust.
var ffi = KataruInterface.new()

# ------------------------------------------------------------------------------
# Signals - subscribe to these signals to listen to dialogue events.
# ------------------------------------------------------------------------------

# Signals a character saying a line of dialogue.
signal dialogue(char_name: String, text: String, attributes: Array)

# Signals an array of choices that the player can make.
signal choices(choices: Array, timeout: float)

# Signals a command issued by Kataru, which should trigger a function call.
signal command(cmd_name: String, params: Dictionary)

# Signals a command asking for input from the user to be stored in Kataru state.
signal input_command(input: Dictionary, timeout: float)


# Runs the first line in a given passage.
func run(passage: String):
	self.ffi.run(passage)


# Runs lines in the given passage until a choice is encountered.
func run_until_choice(passage: String):
	self.ffi.run_until_choice(passage)


# Runs the next line of dialogue in the current passage.
func next(input: String):
	self.ffi.next(input)


# Called when the node enters the scene tree for the first time.
func _ready():
	# Provide a story source path to Rust to compile the story to bytecode, but only if we're in the editor.
	var story_src_path = ""
	if !OS.has_feature("standalone"):
		story_src_path = ProjectSettings.globalize_path(STORY_PATH)

	self.ffi.init(
		story_src_path,
		ProjectSettings.globalize_path(COMPILED_STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		DEFAULT_PASSAGE,
		DEBUG_LEVEL
	)

	# Connect callbacks.
	self.ffi.dialogue.connect(
		func(char_name: String, text: String, attributes: String): self.dialogue.emit(
			char_name, text, JSON.parse_string(attributes)
		)
	)
	self.ffi.choices.connect(
		func(choice_list: Array, timeout: float): self.choices.emit(choice_list, timeout)
	)
	self.ffi.command.connect(
		func(cmd_name: String, params: String): self.command.emit(
			cmd_name, JSON.parse_string(params)
		)
	)
	self.ffi.input_command.connect(
		func(inputs: Dictionary, timeout: float): self.input.emit(inputs, timeout)
	)
	self.ffi.fatal.connect(func(message: String): assert(message != "", message))


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass
