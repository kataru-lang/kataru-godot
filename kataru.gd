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

# Constants to be configured.
@export var root_path = "res://kataru"
@export var story_path = "res://kataru/story"
@export var compiled_story_path = "res://kataru/story.bin"
@export var bookmark_path = "user://kataru-bookmark.yml"
@export var default_passage = ""
@export var debug_level = DebugLevel.INFO
@export var watch_poll_interval = 0.5

const CODEGEN_PATH = "res://addons/kataru/consts"
const TEMPLATE_PATH = "res://addons/kataru/consts/template.yml"

const Directories = preload("res://addons/kataru/directories.gd")
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
signal dialogue(character: String, text: String, attributes: Array[Dictionary])

# Signals an array of choices that the player can make.
signal choices(choices: Array[String], timeout: float)

# Signals a command issued by Kataru, which should trigger a function call.
signal command(cmd_name: String, normalized_name: String, params: Dictionary)

# Signals a command asking for input from the user to be stored in Kataru state.
signal input_command(input: Dictionary, timeout: float)

# Signals that Kataru has loaded. Other autoload scripts can wait for this signal before running.
signal loaded

# Signals that Kataru has reached the end of the current passage.
signal end


# Runs the first line in a given passage.
func run(passage: String):
	self.ffi.run(passage)


# Runs lines in the given passage until a choice is encountered.
func run_until_choice(passage: String):
	self.ffi.run_until_choice(passage)


# Runs the next line of dialogue in the current passage.
func next(input: String = ""):
	self.ffi.next(input)


# Register a function.
# If registering for a specific character, specify the char_name.
func register(f: Callable, cmd_name: String, char_name: String = ""):
	if char_name != "":
		print("register for character: ", char_name)
		cmd_name = cmd_name.replace("$character.", char_name + ".")
	if self.debug_level > DebugLevel.NONE:
		print("Kataru.register(): Registering ", cmd_name)
	Commands.registry[cmd_name] = f


func _connect_callbacks():
	self.ffi.loaded.connect(func(): self.loaded.emit())
	self.ffi.fatal.connect(func(message: String): assert(false, message))

	self.ffi.dialogue.connect(
		func(char_name: String, text: String, attributes: String): self.dialogue.emit(
			char_name, text, JSON.parse_string(attributes)
		)
	)
	self.ffi.choices.connect(
		func(choice_list: Array[String], timeout: float): self.choices.emit(choice_list, timeout)
	)
	self.ffi.command.connect(self.Commands.call_command)
	self.ffi.input_command.connect(
		func(inputs: Dictionary, timeout: float): self.input.emit(inputs, timeout)
	)
	self.ffi.end.connect(func(): self.end.emit())


func init():
	# Provide a story source path to Rust to compile the story to bytecode, but only if we're in the editor.
	var story_src_path = ""
	var codegen_path = ""
	if !OS.has_feature("standalone"):
		Directories.setup()
		story_src_path = ProjectSettings.globalize_path(self.story_path)
		codegen_path = ProjectSettings.globalize_path(CODEGEN_PATH)

	self.ffi.init(
		story_src_path,
		ProjectSettings.globalize_path(self.compiled_story_path),
		ProjectSettings.globalize_path(self.bookmark_path),
		codegen_path,
		self.default_passage,
		self.debug_level,
		self.watch_poll_interval
	)


# Called when the node enters the scene tree for the first time.
func _ready():
	self._connect_callbacks()
	self.init()


func set_state(variable: String, value):
	self.ffi.set_state(variable, value)


func get_state(variable: String):
	self.ffi.get_state(variable)


func save(path: String):
	self.ffi.save(path)


func load(path: String):
	self.ffi.load(path)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float):
	self.ffi.watch_story_dir(delta)
