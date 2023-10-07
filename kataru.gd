# ------------------------------------------------------------------------------
# Kataru - the simple YAML-based dialogue engine.
# ------------------------------------------------------------------------------
#
# This script is intended to be used as an Autoload:
# ```toml
# [autoload]
#
# Kataru="*res://addons/kataru/kataru.gd"
# ```
@tool
extends Node

enum DebugLevel { NONE, INFO, VERBOSE }

const STORY_PATH = "res://kataru-story"
const COMPILED_STORY_PATH = "res://kataru-story.bin"
const CODEGEN_PATH = "res://addons/kataru/consts"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"
const DEBUG_LEVEL = DebugLevel.INFO
const WATCH_POLL_INTERVAL = 0.5

const Characters = preload("res://addons/kataru/consts/characters.gd")
const Passages = preload("res://addons/kataru/consts/passages.gd")
const Namespaces = preload("res://addons/kataru/consts/namespaces.gd")
var Commands = preload("res://addons/kataru/consts/commands.gd").new()

# Interface with Rust.
var ffi = KataruInterface.new()

# ------------------------------------------------------------------------------
# Signals - subscribe to these signals to listen to dialogue events.
# ------------------------------------------------------------------------------

# Signals a character saying a line of dialogue.
signal dialogue(character: String, text: String, attributes: Array)

# Signals an array of choices that the player can make.
signal choices(choices: Array, timeout: float)

# Signals a command issued by Kataru, which should trigger a function call.
signal command(cmd_name: String, normalized_name: String, params: Dictionary)

# Signals a command asking for input from the user to be stored in Kataru state.
signal input_command(input: Dictionary, timeout: float)

# Signals that Kataru has loaded. Other autoload scripts can wait for this signal before running.
signal loaded


# Runs the first line in a given passage.
func run(passage: String):
	self.ffi.run(passage)


# Runs lines in the given passage until a choice is encountered.
func run_until_choice(passage: String):
	self.ffi.run_until_choice(passage)


# Runs the next line of dialogue in the current passage.
func next(input: String):
	self.ffi.next(input)


# Register a function.
# If registering for a specific character, specify the char_name.
func register(f: Callable, cmd_name: String, char_name: String = ""):
	if char_name != "":
		print("register for character: ", char_name)
		cmd_name = cmd_name.replace("$character.", char_name + ".")
	if DEBUG_LEVEL > DebugLevel.NONE:
		print("Kataru.register(): Registering ", cmd_name)
	Commands.registry[cmd_name] = f


# Called when the node enters the scene tree for the first time.
func _ready():
	# Connect callbacks used in init.
	self.ffi.loaded.connect(func(): self.loaded.emit())
	self.ffi.fatal.connect(func(message: String): assert(false, message))

	# Provide a story source path to Rust to compile the story to bytecode, but only if we're in the editor.
	var story_src_path = ""
	var codegen_path = ""
	if !OS.has_feature("standalone"):
		story_src_path = ProjectSettings.globalize_path(STORY_PATH)
		codegen_path = ProjectSettings.globalize_path(CODEGEN_PATH)

	self.ffi.init(
		story_src_path,
		ProjectSettings.globalize_path(COMPILED_STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		codegen_path,
		DEFAULT_PASSAGE,
		DEBUG_LEVEL,
		WATCH_POLL_INTERVAL
	)

	# Connect remaining callbacks.
	self.ffi.dialogue.connect(
		func(char_name: String, text: String, attributes: String): self.dialogue.emit(
			char_name, text, JSON.parse_string(attributes)
		)
	)
	self.ffi.choices.connect(
		func(choice_list: Array, timeout: float): self.choices.emit(choice_list, timeout)
	)
	self.ffi.command.connect(self.Commands.call_command)
	self.ffi.input_command.connect(
		func(inputs: Dictionary, timeout: float): self.input.emit(inputs, timeout)
	)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	self.ffi.watch_story_dir(delta)
